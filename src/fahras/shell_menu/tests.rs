// SPDX-License-Identifier: MIT

use super::*;

#[test]
fn the_shell_menu_is_only_for_files_in_one_folder() {
    let paths = |list: &[&str]| list.iter().map(PathBuf::from).collect::<Vec<_>>();
    assert!(share_one_folder(&paths(&["/a/x.txt", "/a/y.txt"])));
    assert!(share_one_folder(&paths(&["/a/x.txt"])));
    assert!(!share_one_folder(&paths(&["/a/x.txt", "/b/y.txt"])));
    assert!(!share_one_folder(&paths(&[])));
}

#[test]
fn a_keyboard_menu_opens_at_the_row_in_window_pixels() {
    crate::test_support::gtk_test(
        "fahras::shell_menu::tests::a_keyboard_menu_opens_at_the_row_in_window_pixels",
        || {
            use gtk::prelude::*;
            let row = gtk::Box::new(gtk::Orientation::Horizontal, 0);
            row.set_size_request(200, 40);
            assert_eq!(window_point(row.upcast_ref(), 5.0, 5.0), None);
            row.set_margin_start(30);
            row.set_margin_top(70);
            let window = gtk::Window::new();
            window.set_decorated(false);
            window.set_child(Some(&row));
            window.present();
            let context = gtk::glib::MainContext::default();
            while context.iteration(false) {}
            let scale = row.scale_factor();
            assert_eq!(
                window_point(row.upcast_ref(), 10.0, 20.0),
                Some((40 * scale, 90 * scale))
            );
            window.destroy();
        },
    );
}
