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

#[test]
fn a_folder_the_person_colored_keeps_its_color() {
    crate::test_support::gtk_test(
        "fahras::explorer_icons::tests::a_folder_the_person_colored_keeps_its_color",
        || {
            use crate::assets::{IconContext, set_primary_icon_color, sized_icon_paintable};
            use gtk::prelude::TextureExt;
            let png = |color: &str| {
                sized_icon_paintable(icons::FOLDER, color, 24, 1, IconContext::Interface)
                    .expect("folder icon")
                    .save_to_png_bytes()
            };
            set_primary_icon_color("#3366cc");
            set_active(false);
            let (themed_strata, red_strata) = (png("#3366cc"), png("#cc2222"));
            set_active(true);
            assert_ne!(
                png("#3366cc"),
                themed_strata,
                "the theme's folders turn Fluent"
            );
            assert_eq!(
                png("#cc2222"),
                red_strata,
                "a colored folder keeps its own icon"
            );
            set_active(false);
        },
    );
}
