// SPDX-License-Identifier: MIT

use super::*;
use crate::test_support::gtk_test;

fn descendants(root: &impl IsA<gtk::Widget>) -> Vec<gtk::Widget> {
    let mut found = Vec::new();
    let mut pending = vec![root.as_ref().clone()];
    while let Some(widget) = pending.pop() {
        let mut child = widget.first_child();
        while let Some(current) = child {
            child = current.next_sibling();
            pending.push(current);
        }
        found.push(widget);
    }
    found
}

fn menu_buttons(content: &gtk::Box) -> Vec<gtk::MenuButton> {
    let mut buttons: Vec<_> = descendants(content)
        .into_iter()
        .filter_map(|widget| widget.downcast::<gtk::MenuButton>().ok())
        .collect();
    buttons.sort_by_key(|button| button.label().map(|label| label.to_string()));
    buttons
}

fn choose(button: &gtk::MenuButton, label: &str) {
    let menu = button.popover().expect("choice menu");
    let option = descendants(&menu)
        .into_iter()
        .filter_map(|widget| widget.downcast::<gtk::Button>().ok())
        .find(|option| {
            descendants(option).iter().any(|widget| {
                widget
                    .downcast_ref::<gtk::Label>()
                    .is_some_and(|text| text.text() == label)
            })
        })
        .unwrap_or_else(|| panic!("no option {label}"));
    option.emit_clicked();
}

fn preview_icon(content: &gtk::Box) -> Option<String> {
    descendants(content)
        .into_iter()
        .filter_map(|widget| widget.downcast::<gtk::Image>().ok())
        .find(|image| image.pixel_size() == ICON_PREVIEW_SIZE)
        .and_then(|image| image.icon_name())
        .map(|name| name.to_string())
}

#[test]
fn look_choices_apply_in_every_window_and_are_saved() {
    gtk_test(
        "ui::settings::fahras_look::tests::look_choices_apply_in_every_window_and_are_saved",
        || {
            let preferences = PreferenceManager::shared();
            let pages: Vec<_> = (0..2)
                .map(|_| {
                    let content = gtk::Box::new(gtk::Orientation::Vertical, 0);
                    append_look_options(&content, &preferences);
                    let window = gtk::Window::new();
                    window.set_child(Some(&content));
                    window.present();
                    (window, content)
                })
                .collect();
            let (_, first) = &pages[0];
            let [skin, icon] = <[gtk::MenuButton; 2]>::try_from(menu_buttons(first))
                .expect("icon and skin choices");
            assert_eq!(icon.label().as_deref(), Some("Manila"));
            assert_eq!(skin.label().as_deref(), Some("Fahras"));

            choose(&icon, "Forest");
            choose(&skin, "Windows 11");
            assert_eq!(preferences.app_icon(), AppIcon::Forest);
            assert_eq!(preferences.explorer_skin(), ExplorerSkin::Windows11);
            for (_, content) in &pages {
                assert_eq!(preview_icon(content).as_deref(), Some("fahras-forest"));
                let labels: Vec<_> = menu_buttons(content)
                    .iter()
                    .filter_map(|button| button.label().map(|label| label.to_string()))
                    .collect();
                assert_eq!(labels, ["Forest", "Windows 11"]);
            }

            crate::storage::writer::settle();
            let saved = std::fs::read_to_string(
                crate::ui::preferences::config_directory().join("settings.toml"),
            )
            .expect("saved settings");
            assert!(saved.contains("app_icon = \"forest\""), "{saved}");
            assert!(saved.contains("explorer_skin = \"windows11\""), "{saved}");
            for (window, _) in pages {
                window.destroy();
            }
        },
    );
}
