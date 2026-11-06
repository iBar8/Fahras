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
    let group = super::settings_group(content, "LOOK");
    let skin = super::bindings::choice_menu(
        preferences,
        "Explorer look",
        &[
            ("Fahras", ExplorerSkin::Off),
            ("Windows 10", ExplorerSkin::Windows10),
            ("Windows 11", ExplorerSkin::Windows11),
            (
                "fahras.look.windows10_legacy",
                ExplorerSkin::Windows10Legacy,
            ),
            (
                "fahras.look.windows11_legacy",
                ExplorerSkin::Windows11Legacy,
            ),
        ],
        PreferenceManager::explorer_skin,
        PreferenceManager::set_explorer_skin,
    );
    group.append(&super::control_row(
        "Explorer look",
        "Looks like Windows File Explorer, with its colors in light or dark to match your theme.",
        &skin,
    ));
    #[cfg(windows)]
    group.append(&folder_handler_row());
    group.append(&super::control_row(
        "App icon",
        "Shown on the window and the taskbar.",
        &app_icon_choice(preferences),
    ));
}

/// The setting lives in the Windows registry, not in Fahras's settings file.
#[cfg(windows)]
fn folder_handler_row() -> gtk::Box {
    let switch = gtk::Switch::new();
    switch.set_valign(gtk::Align::Center);
    switch.set_active(crate::platform::windows_shell::opens_folders());
    crate::ui::accessibility::set_label(&switch, &crate::i18n::tr("Open folders in Fahras"));
    switch.connect_state_set(
        |_, enabled| match crate::platform::windows_shell::set_opens_folders(enabled) {
            Ok(()) => gtk::glib::Propagation::Proceed,
            Err(error) => {
                tracing::warn!(%error, "could not change the program that opens folders");
                gtk::glib::Propagation::Stop
            }
        },
    );
    super::control_row(
        "Open folders in Fahras",
        "Double-clicking a folder in Windows opens it in Fahras instead of File Explorer.",
        &switch,
    )
}

fn app_icon_choice(preferences: &Rc<PreferenceManager>) -> gtk::Box {
    let choices = AppIcon::ALL.map(|icon| (icon.label(), icon));
    let menu = super::bindings::choice_menu(
        preferences,
        "App icon",
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
    menu.set_hexpand(true);
    let control = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    control.set_valign(gtk::Align::Center);
    control.set_hexpand(true);
    control.append(&preview);
    control.append(&menu);
    control
}

#[cfg(test)]
mod tests;
