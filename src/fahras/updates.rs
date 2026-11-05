// SPDX-License-Identifier: MIT

//! Release artifacts for Fahras updates. Windows updates through the signed
//! installer; Linux keeps Strata's archives.

pub(crate) const WINDOWS_TARGET: &str = "x86_64-pc-windows-gnu";
const LINUX_TARGETS: [&str; 2] = ["x86_64-unknown-linux-gnu", "aarch64-unknown-linux-gnu"];

pub(crate) fn windows_installer_name(version: &str) -> String {
    format!("Fahras-Setup-{version}-x64.exe")
}

/// The file name a release publishes for `target`, or `None` for a target
/// Fahras does not release.
pub(crate) fn artifact_name(version: &str, target: &str) -> Option<String> {
    if target == WINDOWS_TARGET {
        return Some(windows_installer_name(version));
    }
    LINUX_TARGETS
        .contains(&target)
        .then(|| format!("strata-{version}-{target}.tar.gz"))
}

/// The release target this build installs updates for.
pub(crate) fn this_target() -> String {
    if cfg!(windows) {
        WINDOWS_TARGET.to_owned()
    } else {
        format!("{}-unknown-linux-gnu", std::env::consts::ARCH)
    }
}

#[cfg(test)]
mod tests;
