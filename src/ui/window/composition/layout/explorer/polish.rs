// SPDX-License-Identifier: MIT

//! File Explorer's finishing touches, shown only in the polished (not legacy)
//! looks: the coloured New icon, the Details pane button and the view switches.

use std::rc::Rc;

use gtk::prelude::*;

use crate::{
    assets,
    ui::{
        browser::BrowserView, browser_modes::BrowserMode, preferences::PreferenceManager,
        preview::PreviewDrawer,
    },
};

const PLAIN_NEW: &str = "fahras-fluent-new-symbolic";
const COLORED_NEW: &str = "fahras-fluent-new-color";

fn polished(preferences: &PreferenceManager) -> bool {
    preferences.explorer_skin().is_polished()
}

/// Shown only while a polished look is on.
fn show_when_polished(widget: &impl IsA<gtk::Widget>, preferences: &Rc<PreferenceManager>) {
    preferences.bind_preference(widget, polished, |widget, shown| widget.set_visible(shown));
}

/// Explorer's New is the one coloured command; the legacy looks keep it plain.
pub(super) fn color_new_icon(
    new: &gtk::MenuButton,
    preferences: &Rc<PreferenceManager>,
    size: i32,
) {
    preferences.bind_preference(new, polished, move |new, polished| {
        let Some(content) = new
            .downcast_ref::<gtk::MenuButton>()
            .and_then(gtk::MenuButton::child)
            .and_downcast::<gtk::Box>()
        else {
            return;
        };
        let name = if polished { COLORED_NEW } else { PLAIN_NEW };
        if let Some(old) = content.first_child() {
            content.remove(&old);
        }
        let icon = assets::text_icon(name, size);
        icon.set_widget_name(name);
        content.prepend(&icon);
    });
}

/// "Details" opens the preview pane for the selected item, as Explorer's opens
/// its details pane.
pub(super) fn details_pane_button(
    browser: &BrowserView,
    preview: &PreviewDrawer,
    preferences: &Rc<PreferenceManager>,
    size: i32,
) -> gtk::Button {
    let content = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    content.append(&assets::text_icon(
        "fahras-fluent-details-pane-symbolic",
        size,
    ));
    let label = crate::i18n::tr("fahras.explorer.details");
    content.append(&gtk::Label::new(Some(&label)));
    let button = gtk::Button::builder().child(&content).build();
    button.add_css_class("explorer-command");
    button.add_css_class("explorer-details-pane");
    crate::ui::accessibility::set_label(&button, &label);
    let (browser, preview) = (browser.clone(), preview.clone());
    button.connect_clicked(move |_| {
        preview.toggle(
            crate::ui::preview::preview_target(browser.search_result_for_space()),
            browser.browser().active_depth(),
        );
    });
    show_when_polished(&button, preferences);
    button
}

fn view_switch(
    icon: &str,
    label: &str,
    mode: BrowserMode,
    preferences: &Rc<PreferenceManager>,
) -> gtk::ToggleButton {
    let button = gtk::ToggleButton::builder()
        .child(&assets::text_icon(icon, 16))
        .build();
    button.add_css_class("explorer-view-switch");
    let label = crate::i18n::tr(label);
    button.set_tooltip_text(Some(&label));
    crate::ui::accessibility::set_label(&button, &label);
    preferences.bind_preference(
        &button,
        PreferenceManager::browser_mode,
        move |button, current| {
            if let Some(button) = button.downcast_ref::<gtk::ToggleButton>() {
                button.set_active(current == mode);
            }
        },
    );
    let manager = Rc::downgrade(preferences);
    button.connect_clicked(move |button| {
        if let Some(manager) = manager.upgrade() {
            manager.set_browser_mode(mode);
        }
        // Clicking the shown view again keeps it shown.
        button.set_active(true);
    });
    button
}

/// Details and Large icons, at the right of the status bar.
pub(super) fn view_switches(preferences: &Rc<PreferenceManager>) -> gtk::Box {
    let switches = gtk::Box::new(gtk::Orientation::Horizontal, 2);
    switches.add_css_class("explorer-view-switches");
    switches.set_hexpand(true);
    switches.set_halign(gtk::Align::End);
    switches.append(&view_switch(
        "fahras-fluent-view-details-symbolic",
        "fahras.explorer.details",
        BrowserMode::List,
        preferences,
    ));
    switches.append(&view_switch(
        "fahras-fluent-view-icons-symbolic",
        "fahras.explorer.large_icons",
        BrowserMode::Icons,
        preferences,
    ));
    show_when_polished(&switches, preferences);
    switches
}
