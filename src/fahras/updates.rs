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

pub(crate) const RELEASES_PAGE: &str = "https://github.com/iBar8/Fahras/releases";

#[cfg(test)]
static ACT_AS_LINUX_FOR_TEST: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

/// Makes this test process behave like a Linux release build. Process-wide,
/// so only for isolated GTK test processes.
#[cfg(test)]
pub(crate) fn act_as_linux_release_for_test() {
    ACT_AS_LINUX_FOR_TEST.store(true, std::sync::atomic::Ordering::Relaxed);
}

/// Fahras publishes signed updates for Windows only, so Linux builds leave the
/// updater off and point to the Releases page. Tests keep Strata's updater on
/// unless they ask for Linux release behavior.
#[cfg(not(test))]
pub(crate) fn in_app_updates() -> bool {
    cfg!(windows)
}

#[cfg(test)]
pub(crate) fn in_app_updates() -> bool {
    !ACT_AS_LINUX_FOR_TEST.load(std::sync::atomic::Ordering::Relaxed)
}

#[cfg(test)]
mod tests;
