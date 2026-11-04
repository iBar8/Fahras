// SPDX-License-Identifier: MIT

use super::*;
use crate::test_support::gtk_test;

fn checked_families(list: &gtk::ListBox) -> Vec<&'static str> {
    rows(list)
        .filter(|row| {
            row.child()
                .and_then(|content| content.last_child())
                .is_some_and(|check| check.get_visible())
        })
        .filter_map(|row| row_family(&row))
        .collect()
}

fn font_list_of(button: &gtk::MenuButton) -> gtk::ListBox {
    button
        .popover()
        .and_then(|popover| popover.child())
        .and_downcast::<gtk::ScrolledWindow>()
        .and_then(|scroll| scroll.child())
        .and_then(|child| {
            child
                .downcast_ref::<gtk::Viewport>()
                .and_then(gtk::Viewport::child)
                .or(Some(child))
        })
        .and_downcast::<gtk::ListBox>()
        .expect("font list")
}

#[test]
fn arabic_font_choices_follow_the_preference_across_windows() {
    gtk_test(
        "ui::settings::arabic_font::tests::arabic_font_choices_follow_the_preference_across_windows",
        || {
            let preferences = PreferenceManager::shared();
            preferences.set_arabic_font(None);
            crate::storage::writer::settle();
            let settings = crate::ui::preferences::config_directory().join("settings.toml");
            let before = std::fs::read(&settings).ok();
            let windows = [gtk::Window::new(), gtk::Window::new()];
            let buttons: Vec<_> = windows
                .iter()
                .map(|window| {
                    let button = arabic_font_choice(&preferences);
                    window.set_child(Some(&button));
                    window.present();
                    button
                })
                .collect();
            crate::storage::writer::settle();
            assert_eq!(
                std::fs::read(&settings).ok(),
                before,
                "building the choice writes nothing"
            );
            let lists: Vec<_> = buttons.iter().map(font_list_of).collect();
            for (button, list) in buttons.iter().zip(&lists) {
                assert_eq!(
                    button.label().as_deref(),
                    Some(arabic_fonts::DEFAULT_FAMILY)
                );
                assert_eq!(checked_families(list), [arabic_fonts::DEFAULT_FAMILY]);
                assert_eq!(rows(list).count(), FONTS.len());
            }

            let amiri = rows(&lists[0])
                .into_iter()
                .find(|row| row_family(row) == Some("Amiri"))
                .expect("Amiri row");
            amiri.emit_activate();
            assert_eq!(preferences.arabic_font().as_deref(), Some("Amiri"));
            for (button, list) in buttons.iter().zip(&lists) {
                assert_eq!(button.label().as_deref(), Some("Amiri"));
                assert_eq!(checked_families(list), ["Amiri"]);
            }

            lists[1].row_at_index(0).expect("first row").emit_activate();
            assert_eq!(
                preferences.arabic_font(),
                None,
                "the default is stored as unset"
            );
            for button in &buttons {
                assert_eq!(
                    button.label().as_deref(),
                    Some(arabic_fonts::DEFAULT_FAMILY)
                );
            }

            preferences.set_arabic_font(Some("No Longer Bundled".into()));
            for (button, list) in buttons.iter().zip(&lists) {
                assert_eq!(
                    button.label().as_deref(),
                    Some(arabic_fonts::DEFAULT_FAMILY)
                );
                assert_eq!(checked_families(list), [arabic_fonts::DEFAULT_FAMILY]);
            }
            for window in windows {
                window.destroy();
            }
        },
    );
}
