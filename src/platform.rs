// SPDX-License-Identifier: MIT

//! Fahras: the `std::os::unix` extensions on Linux, and stand-ins with the
//! same names on Windows, so Strata's Unix code builds on both.

#[cfg(windows)]
pub(crate) use rustix_compat::unix;
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
