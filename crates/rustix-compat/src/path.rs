// SPDX-License-Identifier: MIT

use std::{
    borrow::Cow,
    ffi::{CStr, CString, OsStr, OsString},
    io,
    path::{Component, Components, Path, PathBuf},
};

/// Anything rustix accepts as a path. Byte strings are WTF-8, the encoding
/// `OsStr` uses on Windows, so names round-trip through `Dir` entries.
pub trait Arg {
    fn to_path(self) -> io::Result<PathBuf>;
}

fn from_bytes(bytes: &[u8]) -> io::Result<PathBuf> {
    match std::str::from_utf8(bytes) {
        Ok(text) => Ok(PathBuf::from(text)),
        // SAFETY: non-UTF-8 names only reach here from `Dir` entries, which this
        // crate produces from `OsStr::as_encoded_bytes`.
        Err(_) => Ok(PathBuf::from(unsafe { OsStr::from_encoded_bytes_unchecked(bytes) })),
    }
}

macro_rules! by_as_ref {
    ($($type:ty),+) => {
        $(impl Arg for $type {
            fn to_path(self) -> io::Result<PathBuf> {
                Ok(AsRef::<Path>::as_ref(&self).to_path_buf())
            }
        })+
    };
}

by_as_ref!(
    &str, &String, String, &Path, &PathBuf, PathBuf, &OsStr, &OsString, OsString,
    Cow<'_, OsStr>, Cow<'_, Path>, Component<'_>, Components<'_>
);

impl Arg for Cow<'_, str> {
    fn to_path(self) -> io::Result<PathBuf> {
        Ok(PathBuf::from(self.as_ref() as &str))
    }
}

impl Arg for &CStr {
    fn to_path(self) -> io::Result<PathBuf> {
        from_bytes(self.to_bytes())
    }
}

impl Arg for &CString {
    fn to_path(self) -> io::Result<PathBuf> {
        from_bytes(self.to_bytes())
    }
}

impl Arg for CString {
    fn to_path(self) -> io::Result<PathBuf> {
        from_bytes(self.to_bytes())
    }
}

impl Arg for Cow<'_, CStr> {
    fn to_path(self) -> io::Result<PathBuf> {
        from_bytes(self.to_bytes())
    }
}

impl Arg for &[u8] {
    fn to_path(self) -> io::Result<PathBuf> {
        from_bytes(self)
    }
}

impl Arg for &Vec<u8> {
    fn to_path(self) -> io::Result<PathBuf> {
        from_bytes(self)
    }
}
