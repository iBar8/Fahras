// SPDX-License-Identifier: MIT

//! Fahras: the `std::os::unix` extensions on Linux, and stand-ins with the
//! same names on Windows, so Strata's Unix code builds on both.

#[cfg(windows)]
pub(crate) use rustix_compat::unix;

#[cfg(windows)]
#[path = "platform/windows/shell.rs"]
pub(crate) mod windows_shell;
#[cfg(unix)]
pub(crate) use std::os::unix;

/// The archive path as the bundled unrar library takes it: bytes on Linux,
/// UTF-16 on Windows. Both are NUL-terminated.
#[cfg(unix)]
pub(crate) fn archive_path(path: &std::path::Path) -> Result<std::ffi::CString, String> {
    use std::os::unix::ffi::OsStrExt;
    std::ffi::CString::new(path.as_os_str().as_bytes()).map_err(|error| error.to_string())
}

#[cfg(windows)]
pub(crate) fn archive_path(path: &std::path::Path) -> Result<Vec<u16>, String> {
    use std::os::windows::ffi::OsStrExt;
    let wide: Vec<u16> = path.as_os_str().encode_wide().collect();
    if wide.contains(&0) {
        return Err("The archive path contains a NUL character".to_owned());
    }
    Ok(wide.into_iter().chain([0]).collect())
}

/// Makes bundled fonts available to Pango for this process only.
#[cfg(windows)]
pub(crate) fn register_fonts(
    paths: impl IntoIterator<Item = std::path::PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    use gtk::pango::prelude::FontMapExt;
    let font_map = pangocairo::FontMap::default();
    for path in paths {
        font_map.add_font_file(&path)?;
    }
    Ok(())
}

/// Splits an absolute path into its filesystem root and the path below it:
/// `/` on Linux, the drive or share root (`C:\`, `\\server\share\`) on Windows.
pub(crate) fn split_root(path: &std::path::Path) -> Option<(std::path::PathBuf, &std::path::Path)> {
    use std::path::Component;
    let mut components = path.components();
    let mut root = std::path::PathBuf::new();
    if let Some(Component::Prefix(prefix)) = components.clone().next() {
        root.push(prefix.as_os_str());
        components.next();
    }
    match components.next() {
        Some(Component::RootDir) => root.push(Component::RootDir),
        _ => return None,
    }
    Some((root, components.as_path()))
}

/// Whether deleting at `path` can go to the Recycle Bin: Windows keeps one on
/// fixed drives only, while removable and network drives delete permanently.
/// GLib reports `access::can-trash` as false everywhere on Windows, although
/// its `g_file_trash` uses the Recycle Bin.
#[cfg(windows)]
#[expect(
    unsafe_code,
    reason = "GetDriveTypeW is only exposed through the Win32 FFI"
)]
pub(crate) fn recycle_bin_covers(path: &std::path::Path) -> bool {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::GetDriveTypeW;
    const DRIVE_FIXED: u32 = 3;
    let Some((root, _)) = split_root(path) else {
        return false;
    };
    let wide: Vec<u16> = root.as_os_str().encode_wide().chain([0]).collect();
    // SAFETY: `wide` is a NUL-terminated root path that outlives the call.
    unsafe { GetDriveTypeW(wide.as_ptr()) == DRIVE_FIXED }
}

/// Starts a GIO file monitor on the main thread and waits for it. GLib's
/// Windows monitors deliver changes as completion callbacks to the thread that
/// started them, and only GLib's main loop waits alertably for them; a monitor
/// started on a plain worker thread never reports anything.
#[cfg(windows)]
pub(crate) fn watch_on_main_thread(
    file: &gtk::gio::File,
    folder: bool,
    flags: gtk::gio::FileMonitorFlags,
) -> Result<gtk::gio::FileMonitor, gtk::glib::Error> {
    use gtk::{gio, gio::prelude::FileExt, glib};
    let (sender, receiver) = std::sync::mpsc::channel();
    let file = file.clone();
    glib::MainContext::default().invoke(move || {
        let created = if folder {
            file.monitor_directory(flags, gio::Cancellable::NONE)
        } else {
            file.monitor_file(flags, gio::Cancellable::NONE)
        };
        let _ = sender.send(created.map(Unconnected));
    });
    match receiver.recv() {
        Ok(created) => created.map(|Unconnected(monitor)| monitor),
        Err(_) => Err(glib::Error::new(
            gio::IOErrorEnum::Cancelled,
            "the main loop stopped before the monitor started",
        )),
    }
}

/// A monitor that has no signal handlers yet.
#[cfg(windows)]
struct Unconnected(gtk::gio::FileMonitor);

#[cfg(windows)]
#[expect(
    unsafe_code,
    reason = "a monitor without handlers moves from the UI thread to the monitors worker"
)]
// SAFETY: as for `BareMonitor`: no handler is connected while it crosses threads, and GObject
// reference counts are thread-safe.
unsafe impl Send for Unconnected {}
