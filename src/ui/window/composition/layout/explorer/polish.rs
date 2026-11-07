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

/// Explorer's status bar is plain text: the item count first and no
/// shortcuts button. Fahras's own look keeps its footer as it was.
pub(super) fn arrange_status(status: &gtk::Box, polished: bool) {
    let Some(row) = status.parent().and_downcast::<gtk::Box>() else {
        return;
    };
    let mut anchor = None;
    let mut child = row.first_child();
    while let Some(current) = child {
        child = current.next_sibling();
        if current.has_css_class("shortcut-footer-count") {
            anchor = Some(current.clone());
        }
        if current.is::<gtk::ToggleButton>() && current.has_css_class("shortcut-footer-button") {
            keep_shortcuts_button(&current, polished);
        }
    }
    let after = if polished { None } else { anchor };
    row.reorder_child_after(status, after.as_ref());
}

const HIDDEN_FOR_EXPLORER: &str = "explorer-hidden";

fn keep_shortcuts_button(button: &gtk::Widget, polished: bool) {
    if button.has_css_class(HIDDEN_FOR_EXPLORER) == polished {
        return;
    }
    if polished {
        button.add_css_class(HIDDEN_FOR_EXPLORER);
        button.set_visible(false);
        // The footer shows it again as hints change; the Explorer look keeps it hidden.
        button.connect_visible_notify(|button| {
            if button.has_css_class(HIDDEN_FOR_EXPLORER) && button.is_visible() {
                button.set_visible(false);
            }
        });
    } else {
        button.remove_css_class(HIDDEN_FOR_EXPLORER);
        button.set_visible(PreferenceManager::shared().show_keybinding_hints());
    }
}

/// Explorer greys out the commands that act on a selection while nothing is
/// selected; the legacy looks keep them available.
pub(super) fn grey_without_selection(
    buttons: &[&gtk::Button],
    controller: &Rc<crate::app::Browser>,
    preferences: &Rc<PreferenceManager>,
) {
    let anchor = buttons.first().map(|button| (*button).clone());
    let buttons: Vec<_> = buttons.iter().map(|button| button.downgrade()).collect();
    let weak = Rc::downgrade(controller);
    let manager = Rc::downgrade(preferences);
    let update = Rc::new(move || {
        let (Some(controller), Some(manager)) = (weak.upgrade(), manager.upgrade()) else {
            return;
        };
        let available = !polished(&manager) || controller.selected_count() > 0;
        for button in buttons.iter().filter_map(gtk::glib::WeakRef::upgrade) {
            button.set_sensitive(available);
        }
    });
    update();
    {
        let update = update.clone();
        controller.observe(move |_| update());
    }
    if let Some(anchor) = anchor {
        preferences.bind_preference(&anchor, polished, move |_, _| update());
    }
}

fn find(root: &gtk::Widget, class: &str) -> Option<gtk::Widget> {
    let mut pending = vec![root.clone()];
    while let Some(widget) = pending.pop() {
        if widget.has_css_class(class) {
            return Some(widget);
        }
        pending.extend(std::iter::successors(
            widget.first_child(),
            gtk::Widget::next_sibling,
        ));
    }
    None
}

/// Gives the sidebar Explorer's width in the Windows 11 look, and Fahras's
/// back in the others, without opening a sidebar the person shrank to a rail.
pub(super) fn fit_sidebar(window: &gtk::ApplicationWindow) {
    use crate::ui::window::sidebar_width::{MIN_SIDEBAR_WIDTH, SIDEBAR_WIDTH};
    let root: gtk::Widget = window.clone().upcast();
    let width = crate::fahras::look::sidebar_width(SIDEBAR_WIDTH);
    let minimum = crate::fahras::look::min_sidebar_width(MIN_SIDEBAR_WIDTH);
    if let Some(scroller) = find(&root, "sidebar-scroll") {
        scroller.set_width_request(width);
    }
    let Some(split) = find(&root, "sidebar-split").and_downcast::<gtk::Paned>() else {
        return;
    };
    if let Some(sidebar) = split.start_child() {
        sidebar.set_size_request(minimum, -1);
    }
    if split.position() >= MIN_SIDEBAR_WIDTH.min(minimum) {
        split.set_position(width);
    }
}
