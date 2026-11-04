// SPDX-License-Identifier: MIT

//! Windows versions of the `std::os::unix` extensions Fahras uses. File
//! identity (`dev`/`ino`) is deliberately absent: `std::fs::Metadata` cannot
//! provide it on Windows, so callers use `rustix::fs::stat` instead.

pub mod ffi;
pub mod fs;
pub mod net;
pub mod process;
