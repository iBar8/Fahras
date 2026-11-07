// SPDX-License-Identifier: MIT

//! Details view columns as File Explorer writes them: the system's short date
//! and time, sizes in whole kilobytes, "File folder", and no size for folders.

#![cfg_attr(
    windows,
    expect(unsafe_code, reason = "dates and file types come from Win32 APIs")
)]

use std::cell::Cell;

use crate::model::{FileEntry, MetadataValue};

thread_local! {
    static POLISHED: Cell<bool> = const { Cell::new(false) };
}

/// Set by the theme while a polished (not legacy) Explorer look is on.
pub(crate) fn set_polished(polished: bool) {
    POLISHED.with(|current| current.set(polished));
}

pub(crate) fn is_polished() -> bool {
    POLISHED.with(Cell::get)
}

fn active() -> bool {
    is_polished()
}

/// Explorer rounds every file up to whole kilobytes, so a 1-byte file is 1 KB
/// and only an empty one is 0 KB.
pub(crate) fn kilobytes(bytes: u64) -> String {
    rust_i18n::t!(
        "fahras.details.kilobytes",
        size = crate::i18n::integer(bytes.div_ceil(1024))
    )
    .into_owned()
}

/// The Size column with an Explorer look on, or `None` for Fahras's own.
pub(crate) fn size(entry: &FileEntry) -> Option<String> {
    if !active() {
        return None;
    }
    if entry.is_directory() {
        return Some(String::new());
    }
    Some(match entry.size {
        MetadataValue::Known(bytes) => kilobytes(bytes),
        MetadataValue::Unknown | MetadataValue::Unavailable => String::new(),
    })
}

/// The Type column with an Explorer look on: "File folder", and on Windows
/// in the polished looks the type names Windows itself gives files.
pub(crate) fn kind(entry: &FileEntry) -> Option<String> {
    if !active() {
        return None;
    }
    if entry.is_directory() {
        return Some(crate::i18n::tr("fahras.details.file_folder"));
    }
    if !is_polished() {
        return None;
    }
    windows_type_name(&entry.display_name)
}

/// The name Windows gives a file's type, as File Explorer's Type column shows.
#[cfg(windows)]
fn windows_type_name(name: &str) -> Option<String> {
    use windows_sys::Win32::{
        Storage::FileSystem::FILE_ATTRIBUTE_NORMAL,
        UI::Shell::{SHFILEINFOW, SHGFI_TYPENAME, SHGFI_USEFILEATTRIBUTES, SHGetFileInfoW},
    };
    thread_local! {
        static NAMES: std::cell::RefCell<std::collections::HashMap<String, Option<String>>> =
            std::cell::RefCell::new(std::collections::HashMap::new());
    }
    let key = name
        .rsplit_once('.')
        .map_or_else(String::new, |(_, extension)| extension.to_ascii_lowercase());
    if let Some(known) = NAMES.with_borrow(|names| names.get(&key).cloned()) {
        return known;
    }
    let path: Vec<u16> = format!("file.{key}\0").encode_utf16().collect();
    // SAFETY: a zeroed SHFILEINFOW is valid, and its size is passed with it.
    let named = unsafe {
        let mut info: SHFILEINFOW = std::mem::zeroed();
        let found = SHGetFileInfoW(
            path.as_ptr(),
            FILE_ATTRIBUTE_NORMAL,
            &mut info,
            size_of::<SHFILEINFOW>() as u32,
            SHGFI_TYPENAME | SHGFI_USEFILEATTRIBUTES,
        );
        let length = info
            .szTypeName
            .iter()
            .position(|unit| *unit == 0)
            .unwrap_or(info.szTypeName.len());
        (found != 0 && length > 0).then(|| String::from_utf16_lossy(&info.szTypeName[..length]))
    };
    NAMES.with_borrow_mut(|names| names.insert(key, named.clone()));
    named
}

#[cfg(not(windows))]
fn windows_type_name(_name: &str) -> Option<String> {
    None
}

/// The Date modified column with an Explorer look on: the system's own short
/// date and time, never a relative "Just now".
pub(crate) fn date(seconds: i64) -> Option<String> {
    if !active() {
        return None;
    }
    let local = gtk::glib::DateTime::from_unix_local(seconds).ok()?;
    Some(system_date(&local))
}

#[cfg(windows)]
fn system_date(local: &gtk::glib::DateTime) -> String {
    use windows_sys::Win32::{
        Foundation::SYSTEMTIME,
        Globalization::{DATE_SHORTDATE, GetDateFormatEx, GetTimeFormatEx, TIME_NOSECONDS},
    };
    let time = SYSTEMTIME {
        wYear: local.year() as u16,
        wMonth: local.month() as u16,
        wDayOfWeek: (local.day_of_week() % 7) as u16,
        wDay: local.day_of_month() as u16,
        wHour: local.hour() as u16,
        wMinute: local.minute() as u16,
        wSecond: local.second() as u16,
        wMilliseconds: 0,
    };
    let mut date = [0_u16; 128];
    let mut clock = [0_u16; 128];
    // SAFETY: both buffers are writable and their lengths are passed.
    let (date_len, clock_len) = unsafe {
        (
            GetDateFormatEx(
                std::ptr::null(),
                DATE_SHORTDATE,
                &time,
                std::ptr::null(),
                date.as_mut_ptr(),
                date.len() as i32,
                std::ptr::null(),
            ),
            GetTimeFormatEx(
                std::ptr::null(),
                TIME_NOSECONDS,
                &time,
                std::ptr::null(),
                clock.as_mut_ptr(),
                clock.len() as i32,
            ),
        )
    };
    if date_len <= 1 || clock_len <= 1 {
        return fallback_date(local);
    }
    let text =
        |buffer: &[u16], length: i32| String::from_utf16_lossy(&buffer[..length as usize - 1]);
    format!("{} {}", text(&date, date_len), text(&clock, clock_len))
}

#[cfg(not(windows))]
fn system_date(local: &gtk::glib::DateTime) -> String {
    fallback_date(local)
}

/// Explorer's English short date and time, for systems without Windows'.
fn fallback_date(local: &gtk::glib::DateTime) -> String {
    local
        .format("%-m/%-d/%Y %-I:%M %p")
        .map(|text| text.to_string())
        .unwrap_or_default()
}

/// The name Explorer shows: without the extension of a known file type,
/// unless Windows is set to show extensions.
pub(crate) fn shown_name(entry: &FileEntry) -> Option<String> {
    if !is_polished() || entry.is_directory() || !hides_extensions() {
        return None;
    }
    let (stem, extension) = entry.display_name.rsplit_once('.')?;
    (!stem.is_empty() && known_type(extension)).then(|| stem.to_owned())
}

/// The heading Explorer gives a column, for those it names differently.
pub(crate) fn heading(text: &str) -> Option<String> {
    (is_polished() && text == "Modified").then(|| crate::i18n::tr("fahras.columns.date_modified"))
}

thread_local! {
    static KNOWN_TYPES: std::cell::RefCell<std::collections::HashMap<String, bool>> =
        std::cell::RefCell::new(std::collections::HashMap::new());
}

fn known_type(extension: &str) -> bool {
    let extension = extension.to_ascii_lowercase();
    if let Some(known) = KNOWN_TYPES.with_borrow(|known| known.get(&extension).copied()) {
        return known;
    }
    let known = registered_type(&extension);
    KNOWN_TYPES.with_borrow_mut(|types| types.insert(extension, known));
    known
}

/// Windows hides extensions unless "Hide extensions for known file types" is off.
#[cfg(windows)]
fn hides_extensions() -> bool {
    use windows_sys::Win32::System::Registry::{HKEY_CURRENT_USER, RRF_RT_REG_DWORD, RegGetValueW};
    let key: Vec<u16> = "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Advanced\0"
        .encode_utf16()
        .collect();
    let name: Vec<u16> = "HideFileExt\0".encode_utf16().collect();
    let mut value = 1_u32;
    let mut size = size_of::<u32>() as u32;
    // SAFETY: the buffers are valid for the sizes passed.
    let status = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            key.as_ptr(),
            name.as_ptr(),
            RRF_RT_REG_DWORD,
            std::ptr::null_mut(),
            std::ptr::from_mut(&mut value).cast(),
            &mut size,
        )
    };
    status != 0 || value != 0
}

#[cfg(not(windows))]
fn hides_extensions() -> bool {
    true
}

/// A type is known when Windows has a class registered for its extension.
#[cfg(windows)]
fn registered_type(extension: &str) -> bool {
    use windows_sys::Win32::System::Registry::{
        HKEY, HKEY_CLASSES_ROOT, KEY_READ, RegCloseKey, RegOpenKeyExW,
    };
    let name: Vec<u16> = format!(".{extension}\0").encode_utf16().collect();
    let mut key: HKEY = std::ptr::null_mut();
    // SAFETY: the name is terminated and the key is closed when opened.
    unsafe {
        if RegOpenKeyExW(HKEY_CLASSES_ROOT, name.as_ptr(), 0, KEY_READ, &mut key) != 0 {
            return false;
        }
        RegCloseKey(key);
    }
    true
}

/// Elsewhere a type is known when the system's MIME database knows it.
#[cfg(not(windows))]
fn registered_type(extension: &str) -> bool {
    let (content_type, _) = gtk::gio::content_type_guess(Some(format!("file.{extension}")), None);
    !gtk::gio::content_type_is_unknown(&content_type)
}

/// Explorer lines sizes up on the right; Fahras's own look on the left.
pub(crate) fn align_size(label: &gtk::Label) {
    use gtk::prelude::*;
    crate::ui::preferences::PreferenceManager::shared().bind_preference(
        label,
        |preferences| preferences.explorer_skin().is_polished(),
        |label, polished| {
            if let Some(label) = label.downcast_ref::<gtk::Label>() {
                label.set_xalign(if polished { 1.0 } else { 0.0 });
            }
        },
    );
}

#[cfg(test)]
mod tests;
