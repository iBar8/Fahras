// SPDX-License-Identifier: MIT

//! Fahras: the sandboxed preview worker relies on Linux namespaces, Landlock
//! and seccomp. Windows reports it unsupported, so previews that need it fail
//! closed to an icon instead of parsing untrusted files unsandboxed.

pub(crate) fn run() -> Result<(), String> {
    Err("The preview sandbox is not available on Windows".to_owned())
}

pub(super) fn supported() -> bool {
    false
}
