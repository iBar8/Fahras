// SPDX-License-Identifier: MIT

use super::*;

#[test]
fn drive_roots_from_explorer_get_their_backslash_back() {
    for (argument, expected) in [
        ("C:\"", "C:\\"),
        ("d:\"", "d:\\"),
        ("C:\\", "C:\\"),
        ("C:\\Users\\me", "C:\\Users\\me"),
        ("C:\\folder\"", "C:\\folder\""),
        ("--version", "--version"),
    ] {
        assert_eq!(
            repaired_drive_root(OsString::from(argument)),
            OsString::from(expected),
            "{argument}"
        );
    }
}
