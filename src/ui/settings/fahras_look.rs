// SPDX-License-Identifier: MIT

use std::rc::Rc;

use gtk::prelude::*;

use crate::{
    fahras::look::{AppIcon, ExplorerSkin},
    ui::preferences::PreferenceManager,
};

const ICON_PREVIEW_SIZE: i32 = 32;

/// The Explorer skin and the application icon, under the theme choice.
pub(super) fn append_look_options(content: &gtk::Box, preferences: &Rc<PreferenceManager>) {
    let group = super::settings_group(content, "fahras.look.heading");
    let skin = super::bindings::choice_menu(
        preferences,
        "fahras.skin.title",
        &[
            ("fahras.skin.off", ExplorerSkin::Off),
            ("fahras.skin.windows10", ExplorerSkin::Windows10),
            ("fahras.skin.windows11", ExplorerSkin::Windows11),
        ],
        PreferenceManager::explorer_skin,
        PreferenceManager::set_explorer_skin,
    );
    group.append(&super::control_row(
        "fahras.skin.title",
        "fahras.skin.description",
        &skin,
    ));
    group.append(&super::control_row(
        "fahras.icon.title",
        "fahras.icon.description",
        &app_icon_choice(preferences),
    ));
}

fn app_icon_choice(preferences: &Rc<PreferenceManager>) -> gtk::Box {
    let choices = AppIcon::ALL.map(|icon| (icon.label_key(), icon));
    let menu = super::bindings::choice_menu(
        preferences,
        "fahras.icon.title",
        &choices,
        PreferenceManager::app_icon,
        PreferenceManager::set_app_icon,
    );
    let preview = gtk::Image::new();
    preview.set_pixel_size(ICON_PREVIEW_SIZE);
    preview.set_accessible_role(gtk::AccessibleRole::Presentation);
    preferences.bind_preference(&preview, PreferenceManager::app_icon, |widget, icon| {
        if let Some(preview) = widget.downcast_ref::<gtk::Image>() {
            preview.set_icon_name(Some(icon.icon_name()));
        }
    });
    let control = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    control.set_valign(gtk::Align::Center);
    control.append(&preview);
    control.append(&menu);
    control
}

#[cfg(test)]
mod tests;
