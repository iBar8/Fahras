// SPDX-License-Identifier: MIT

//! Details view columns as File Explorer writes them: the system's short date
//! and time, sizes in whole kilobytes, "File folder", and no size for folders.

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

/// The Type column for a folder with an Explorer look on.
pub(crate) fn kind(entry: &FileEntry) -> Option<String> {
    (active() && entry.is_directory()).then(|| crate::i18n::tr("fahras.details.file_folder"))
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
#[expect(unsafe_code, reason = "Windows formats dates through a Win32 API")]
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

#[cfg(test)]
mod tests;
