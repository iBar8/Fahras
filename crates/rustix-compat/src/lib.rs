// SPDX-License-Identifier: MIT

//! The subset of [rustix](https://docs.rs/rustix) that Fahras uses, implemented
//! for Windows on top of `std::fs` and Win32. Fahras aliases this crate as
//! `rustix` on Windows only, so Strata's code compiles unchanged.
//!
//! Directory handles remember the path they were opened at, and every
//! `*at` call resolves relative to that path. Features Windows lacks, such
//! as extended attributes, device nodes or exchange renames, fail with the
//! errno that Linux reports on filesystems without them, so callers take the
//! fallbacks they already have for such filesystems.

#![cfg(windows)]

pub mod buffer;
pub mod event;
pub mod fd;
pub mod fs;
pub mod io;
pub mod net;
pub mod path;
pub mod pipe;
pub mod process;
pub mod thread;
pub mod time;
pub mod unix;

mod win;
