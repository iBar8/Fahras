// SPDX-License-Identifier: MIT

use super::*;

#[test]
fn explorer_looks_swap_file_and_place_icons_for_bundled_fluent_ones() {
    crate::assets::prepare().expect("bundled assets");
    set_active(false);
    assert_eq!(substitute(icons::FOLDER), None);
    set_active(true);
    for (strata, fluent) in [
        (icons::FOLDER, "fahras-fluent-folder"),
        (icons::PICTURES, "fahras-fluent-image"),
        (icons::HARD_DRIVE, "fahras-fluent-drive"),
        (icons::HOME, "fahras-fluent-home"),
        ("strata-ctp-rust", "fahras-fluent-code"),
    ] {
        assert_eq!(substitute(strata), Some(fluent), "{strata}");
        assert!(svg(fluent, "#000000").is_some(), "{fluent} is bundled");
    }
    assert_eq!(substitute(icons::SETTINGS), None);
    let symbolic = svg("fahras-fluent-cut-symbolic", "#123456").expect("toolbar icon");
    assert!(symbolic.contains("#123456") && !symbolic.contains(SYMBOLIC_FILL));
    set_active(false);
}
