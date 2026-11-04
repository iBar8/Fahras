// SPDX-License-Identifier: MIT

//! Fahras: the `std::os::unix` extensions on Linux, and stand-ins with the
//! same names on Windows, so Strata's Unix code builds on both.

#[cfg(windows)]
pub(crate) use rustix_compat::unix;
#[cfg(unix)]
pub(crate) use std::os::unix;
