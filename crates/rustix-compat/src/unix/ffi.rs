// SPDX-License-Identifier: MIT

use std::ffi::{OsStr, OsString};

/// Bytes are WTF-8, the encoding `OsStr` uses on Windows. Bytes that are not
/// valid UTF-8 and did not come from an `OsStr` are converted lossily.
pub trait OsStrExt {
    fn from_bytes(slice: &[u8]) -> &Self;
    fn as_bytes(&self) -> &[u8];
}

impl OsStrExt for OsStr {
    fn from_bytes(slice: &[u8]) -> &Self {
        match std::str::from_utf8(slice) {
            Ok(text) => OsStr::new(text),
            Err(_) => OsStr::new(Box::leak(
                String::from_utf8_lossy(slice).into_owned().into_boxed_str(),
            )),
        }
    }

    fn as_bytes(&self) -> &[u8] {
        self.as_encoded_bytes()
    }
}

pub trait OsStringExt {
    fn from_vec(vec: Vec<u8>) -> Self;
    fn into_vec(self) -> Vec<u8>;
}

impl OsStringExt for OsString {
    fn from_vec(vec: Vec<u8>) -> Self {
        match String::from_utf8(vec) {
            Ok(text) => OsString::from(text),
            Err(error) => OsString::from(String::from_utf8_lossy(error.as_bytes()).into_owned()),
        }
    }

    fn into_vec(self) -> Vec<u8> {
        self.into_encoded_bytes()
    }
}
