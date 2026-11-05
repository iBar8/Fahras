// SPDX-License-Identifier: MIT

use super::*;

#[test]
fn each_release_target_has_its_own_artifact() {
    for (target, expected) in [
        (WINDOWS_TARGET, Some("Fahras-Setup-1.2.3-x64.exe")),
        (
            "x86_64-unknown-linux-gnu",
            Some("strata-1.2.3-x86_64-unknown-linux-gnu.tar.gz"),
        ),
        (
            "aarch64-unknown-linux-gnu",
            Some("strata-1.2.3-aarch64-unknown-linux-gnu.tar.gz"),
        ),
        ("aarch64-pc-windows-msvc", None),
    ] {
        assert_eq!(
            artifact_name("1.2.3", target).as_deref(),
            expected,
            "{target}"
        );
    }
}

#[test]
fn this_build_installs_its_own_target() {
    assert!(artifact_name("1.2.3", &this_target()).is_some());
}
