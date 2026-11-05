// SPDX-License-Identifier: MIT

use gtk::prelude::*;

use crate::{fahras::look::AppIcon, test_support::gtk_test, ui::preferences::PreferenceManager};

#[test]
fn chosen_icon_reaches_open_windows_and_is_bundled() {
    gtk_test(
        "ui::fahras_app_icon::tests::chosen_icon_reaches_open_windows_and_is_bundled",
        || {
            let preferences = PreferenceManager::shared();
            super::follow_preference();
            let window = gtk::Window::new();
            let theme = gtk::IconTheme::for_display(&WidgetExt::display(&window));
            for icon in AppIcon::ALL {
                assert!(theme.has_icon(icon.icon_name()), "{icon:?} is bundled");
                preferences.set_app_icon(icon);
                assert_eq!(
                    gtk::Window::default_icon_name().as_deref(),
                    Some(icon.icon_name())
                );
            }
            preferences.set_app_icon(AppIcon::Manila);
            window.destroy();
        },
    );
}
