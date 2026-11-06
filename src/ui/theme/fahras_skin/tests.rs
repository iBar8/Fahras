// SPDX-License-Identifier: MIT

use super::*;
use crate::{test_support::gtk_test, ui::preferences::PreferenceManager, ui::theme::ThemeManager};
use gtk::prelude::*;

fn theme(background: &str) -> ThemeTokens {
    ThemeTokens {
        name: "Fixture".into(),
        background: background.into(),
        surface: "#123456".into(),
        text: "#abcdef".into(),
        accent: "#ff00ff".into(),
        danger: "#ff0000".into(),
        muted: "#111111".into(),
        highlight: "#222222".into(),
        border: "#333333".into(),
        dim_text: "#444444".into(),
        syntax_keyword: Some("#555555".into()),
        syntax_string: None,
        syntax_constant: None,
        syntax_type: None,
        syntax_preprocessor: None,
    }
}

#[test]
fn skins_use_explorer_colors_in_the_theme_brightness() {
    for (skin, background, expected) in [
        (ExplorerSkin::Off, "#000000", "#000000"),
        (ExplorerSkin::Windows11, "#ffffff", WINDOWS_11_LIGHT[0]),
        (ExplorerSkin::Windows11, "#101010", WINDOWS_11_DARK[0]),
        (ExplorerSkin::Windows10, "#fafafa", WINDOWS_10_LIGHT[0]),
        (ExplorerSkin::Windows10, "#0a0a0a", WINDOWS_10_DARK[0]),
    ] {
        let skinned = tokens(skin, &theme(background));
        assert_eq!(skinned.background, expected, "{skin:?} on {background}");
        assert_eq!(skinned.name, "Fixture");
        assert_eq!(skinned.syntax_keyword.as_deref(), Some("#555555"));
    }
}

#[test]
fn switching_skins_restyles_open_windows_without_restarting() {
    gtk_test(
        "ui::theme::fahras_skin::tests::switching_skins_restyles_open_windows_without_restarting",
        || {
            let preferences = PreferenceManager::shared();
            let themes = ThemeManager::shared();
            let label = gtk::Label::new(Some("probe"));
            label.add_css_class("sidebar");
            let window = gtk::Window::new();
            window.set_child(Some(&label));
            window.present();
            for skin in [
                ExplorerSkin::Windows11,
                ExplorerSkin::Windows10,
                ExplorerSkin::Off,
            ] {
                preferences.set_explorer_skin(skin);
                let skin_css = PROVIDER.with(|provider| provider.to_str().to_string());
                assert_eq!(skin_css.is_empty(), skin == ExplorerSkin::Off, "{skin:?}");
                let base = themes.current_tokens().expect("selected theme");
                let expected = tokens(skin, &base).background;
                #[expect(deprecated, reason = "GTK has no other way to read a named color")]
                let applied = window.style_context().lookup_color("strata_bg");
                assert_eq!(applied, gtk::gdk::RGBA::parse(&expected).ok(), "{skin:?}");
            }
            window.destroy();
        },
    );
}

#[test]
fn only_the_windows_11_look_takes_explorers_measured_sizes() {
    assert!(stylesheet(ExplorerSkin::Windows11).contains(WINDOWS_11_EXPLORER_CSS));
    for skin in [
        ExplorerSkin::Windows11Legacy,
        ExplorerSkin::Windows10,
        ExplorerSkin::Windows10Legacy,
        ExplorerSkin::Off,
    ] {
        assert!(
            !stylesheet(skin).contains(WINDOWS_11_EXPLORER_CSS),
            "{skin:?} keeps its own sizes"
        );
    }
    assert!(stylesheet(ExplorerSkin::Windows11Legacy).contains(WINDOWS_11_CSS));
}
