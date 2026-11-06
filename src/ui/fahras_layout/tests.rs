// SPDX-License-Identifier: MIT

use super::*;
use crate::test_support::gtk_test;

fn settle() {
    let context = gtk::glib::MainContext::default();
    while context.iteration(false) {}
}

#[test]
fn explorer_layout_shows_the_pane_header_only_for_its_notices() {
    gtk_test(
        "ui::fahras_layout::tests::explorer_layout_shows_the_pane_header_only_for_its_notices",
        || {
            let preferences = PreferenceManager::shared();
            preferences.set_explorer_skin(ExplorerSkin::Off);
            let header = gtk::Box::new(gtk::Orientation::Horizontal, 0);
            header.add_css_class(PANE_HEADER);
            hide_with_tenxer_or_explorer(&header);
            let actions = gtk::Box::new(gtk::Orientation::Horizontal, 0);
            let empty_trash = gtk::Button::builder().visible(false).build();
            keep_in_explorer_layout(&empty_trash);
            actions.append(&empty_trash);
            header.append(&actions);
            settle();
            assert!(header.is_visible(), "Fahras's look keeps the header");

            preferences.set_explorer_skin(ExplorerSkin::Windows11);
            settle();
            assert!(!header.is_visible(), "Explorer has no pane header");
            empty_trash.set_visible(true);
            assert!(header.is_visible(), "Empty Trash stays reachable");
            assert!(header.is_sensitive());
            empty_trash.set_visible(false);
            assert!(!header.is_visible());

            empty_trash.set_visible(true);
            preferences.set_tenxer_mode(true);
            settle();
            assert!(!header.is_visible(), "10xer mode hides it regardless");
            preferences.set_tenxer_mode(false);
            preferences.set_explorer_skin(ExplorerSkin::Off);
            settle();
            assert!(header.is_visible());
        },
    );
}
