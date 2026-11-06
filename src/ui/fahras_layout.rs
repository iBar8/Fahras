// SPDX-License-Identifier: MIT

//! Chrome the Explorer layout replaces with its own.

use gtk::prelude::*;

use crate::{fahras::look::ExplorerSkin, ui::preferences::PreferenceManager};

const PANE_HEADER: &str = "mode-pane-header";
const PANE_NOTICE: &str = "explorer-pane-notice";
const WINDOW_BUTTON: &str = "fahras-window-button";

fn explorer_layout_on(preferences: &PreferenceManager) -> bool {
    preferences.explorer_skin() != ExplorerSkin::Off
}

/// Hidden in 10xer mode, like `tenxer_mode::hide_while_enabled`. An Explorer
/// look hides it too, except while it holds a notice marked with
/// [`keep_in_explorer_layout`].
pub(crate) fn hide_with_tenxer_or_explorer(header: &impl IsA<gtk::Widget>) {
    header.add_css_class("tenxer-suppressed-chrome");
    PreferenceManager::shared().bind_preference(
        header,
        |preferences| (preferences.tenxer_mode(), explorer_layout_on(preferences)),
        |header, _| {
            refresh_pane_header(header.upcast_ref());
            // Its notices are added after the header is built.
            let header = header.downgrade();
            gtk::glib::idle_add_local_once(move || {
                if let Some(header) = header.upgrade() {
                    refresh_pane_header(header.upcast_ref());
                }
            });
        },
    );
}

/// Empty Trash, leftovers and the truncated-listing warning stay reachable
/// while an Explorer look hides the rest of the pane header.
pub(crate) fn keep_in_explorer_layout(notice: &impl IsA<gtk::Widget>) {
    notice.add_css_class(PANE_NOTICE);
    notice.connect_notify_local(Some("visible"), |notice, _| {
        let mut parent = notice.parent();
        while let Some(widget) = parent {
            if widget.has_css_class(PANE_HEADER) {
                refresh_pane_header(&widget);
                return;
            }
            parent = widget.parent();
        }
    });
}

fn shows_notice(widget: &gtk::Widget) -> bool {
    let mut child = widget.first_child();
    while let Some(current) = child {
        // Its own flag: the hidden header makes every child count as hidden.
        let own_visible = current.property::<bool>("visible");
        if own_visible && (current.has_css_class(PANE_NOTICE) || shows_notice(&current)) {
            return true;
        }
        child = current.next_sibling();
    }
    false
}

fn refresh_pane_header(header: &gtk::Widget) {
    let preferences = PreferenceManager::shared();
    let shown =
        !preferences.tenxer_mode() && (!explorer_layout_on(&preferences) || shows_notice(header));
    header.set_visible(shown);
    header.set_sensitive(shown);
}

fn explorer_layout_shown() -> bool {
    PreferenceManager::existing().is_some_and(|preferences| explorer_layout_on(&preferences))
}

/// The Explorer layout shows its own counts in the footer instead.
pub(crate) fn hide_in_explorer_layout(widget: &impl IsA<gtk::Widget>) {
    if explorer_layout_shown() {
        widget.set_visible(false);
    }
}

/// A window opened with the Windows title bar keeps it, so Fahras's own
/// minimize, maximize and close stay hidden there.
fn framed_by_system(widget: &gtk::Widget) -> bool {
    widget
        .root()
        .and_downcast::<gtk::Window>()
        .is_some_and(|window| window.titlebar().is_none())
}

/// Shows one of the header's window buttons when its preference asks for it
/// and the window has no system title bar.
pub(crate) fn show_window_button(button: &impl IsA<gtk::Widget>, wanted: bool) {
    button.add_css_class(WINDOW_BUTTON);
    button.set_visible(wanted && !framed_by_system(button.as_ref()));
}

/// Hides the header's window buttons once the window has the system frame.
pub(crate) fn hide_window_buttons_under_system_frame(header: &gtk::Widget) {
    if !framed_by_system(header) {
        return;
    }
    let mut pending = vec![header.clone()];
    while let Some(widget) = pending.pop() {
        if widget.has_css_class(WINDOW_BUTTON) {
            widget.set_visible(false);
        }
        let mut child = widget.first_child();
        while let Some(current) = child {
            child = current.next_sibling();
            pending.push(current);
        }
    }
}

#[cfg(test)]
mod tests;
