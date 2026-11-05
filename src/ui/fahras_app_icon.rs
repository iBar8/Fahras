// SPDX-License-Identifier: MIT

//! The application icon chosen in Settings, shown on every window and on the
//! Windows taskbar.

use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

use gtk::{gio, glib, prelude::*};

use crate::{fahras::look::AppIcon, ui::preferences::PreferenceManager};

thread_local! {
    static FOLLOWED: RefCell<Weak<PreferenceManager>> = const { RefCell::new(Weak::new()) };
}

pub(crate) fn register_resources() -> Result<(), glib::Error> {
    gio::resources_register_include!("fahras.gresource")
}

/// Shows the saved icon now and follows later changes, on new windows through
/// the default icon and on open ones directly.
pub(crate) fn follow_preference() {
    let preferences = PreferenceManager::shared();
    apply(preferences.app_icon());
    let weak = Rc::downgrade(&preferences);
    if FOLLOWED.with(|followed| followed.replace(weak.clone()).ptr_eq(&weak)) {
        return;
    }
    preferences.observe(Rc::new(move || {
        if let Some(preferences) = weak.upgrade() {
            apply(preferences.app_icon());
        }
    }));
}

fn apply(icon: AppIcon) {
    let name = icon.icon_name();
    if gtk::Window::default_icon_name().as_deref() == Some(name) {
        return;
    }
    gtk::Window::set_default_icon_name(name);
    // On Windows an open window keeps its taskbar icon until it is given a new one.
    for window in gtk::Window::list_toplevels() {
        if let Ok(window) = window.downcast::<gtk::Window>() {
            window.set_icon_name(Some(name));
        }
    }
}

#[cfg(test)]
mod tests;
