// SPDX-License-Identifier: MIT

//! Paths Fahras is started with from the Windows shell.

use std::ffi::OsString;

/// Explorer passes a drive root as `"C:\"`, and Windows argument parsing reads
/// its `\"` as an escaped quote, which leaves `C:"`. This puts the root back.
pub(crate) fn repaired_argument(argument: OsString) -> OsString {
    if cfg!(windows) {
        repaired_drive_root(argument)
    } else {
        argument
    }
}

fn repaired_drive_root(argument: OsString) -> OsString {
    match argument.to_str() {
        Some(text) if is_mangled_drive_root(text) => format!("{}\\", &text[..2]).into(),
        _ => argument,
    }
}

fn is_mangled_drive_root(text: &str) -> bool {
    let bytes = text.as_bytes();
    bytes.len() == 3 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':' && bytes[2] == b'"'
}

#[cfg(test)]
mod tests;
