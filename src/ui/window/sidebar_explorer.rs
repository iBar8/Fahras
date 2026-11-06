// SPDX-License-Identifier: MIT

//! The sidebar as File Explorer arranges it: Home, the user folders, pinned
//! folders, This PC with its drives, then Network and the trash. Its order is
//! fixed, so dragging cannot rearrange Fahras's own sidebar order.

use std::rc::Rc;

use gtk::{glib, prelude::*};

use super::{SidebarState, places::standard_place, sidebar_sections::SidebarSection};
use crate::{fahras::look::ExplorerSkin, model::Location, ui::preferences::PreferenceManager};

/// Explorer lists the user folders in this order, each with its own icon.
const USER_FOLDERS: [(&str, &str); 6] = [
    ("desktop", "fahras-fluent-desktop"),
    ("downloads", "fahras-fluent-downloads"),
    ("documents", "fahras-fluent-document-text"),
    ("pictures", "fahras-fluent-image"),
    ("music", "fahras-fluent-music"),
    ("videos", "fahras-fluent-video"),
];

pub(super) fn arranged_like_explorer(preferences: &PreferenceManager) -> bool {
    preferences.explorer_skin() != ExplorerSkin::Off
}

/// The polished looks add Explorer's Gallery, pins and This PC tree.
fn polished(preferences: &PreferenceManager) -> bool {
    preferences.explorer_skin().is_polished()
}

impl SidebarState {
    pub(super) fn append_explorer_places(self: &Rc<Self>) {
        self.append_ordered_place("home");
        if polished(&self.preference_manager) {
            self.append_gallery();
        }
        self.append_separator();
        for (id, icon) in USER_FOLDERS {
            self.append_explorer_user_folder(id, icon);
        }
        self.append_pinned_places();
        self.append_devices();
        if polished(&self.preference_manager) {
            // Network is a tree row beside This PC, as in Explorer.
            let before = sidebar_rows(self.widget.upcast_ref());
            self.append_ordered_place("network");
            for row in sidebar_rows(self.widget.upcast_ref()) {
                if !before.contains(&row) {
                    tree_row(&row);
                }
            }
        } else {
            self.append_separator();
            self.append_ordered_place("network");
        }
        self.append_ordered_place("trash");
    }

    fn append_explorer_user_folder(self: &Rc<Self>, id: &'static str, icon: &str) {
        if !self.standard_place_visible(id) {
            return;
        }
        if let Some((_, name, directory)) = standard_place(id)
            && let Some(path) = glib::user_special_dir(directory)
        {
            let row = self.append_place(icon, &crate::i18n::tr(name), Location::local(path));
            if polished(&self.preference_manager) {
                mark_pinned(&row);
            }
        }
    }

    /// Explorer's Gallery: the Pictures folder as large pictures.
    fn append_gallery(self: &Rc<Self>) {
        let Some(path) = glib::user_special_dir(glib::UserDirectory::Pictures) else {
            return;
        };
        let row = self.append_place(
            "fahras-fluent-gallery",
            &crate::i18n::tr("fahras.sidebar.gallery"),
            Location::local(path),
        );
        row.add_css_class("explorer-gallery");
        let manager = Rc::downgrade(&self.preference_manager);
        row.connect_clicked(move |_| {
            if let Some(manager) = manager.upgrade() {
                manager.set_browser_mode(crate::ui::browser_modes::BrowserMode::Icons);
            }
        });
    }

    /// In the polished looks the drives hang under a "This PC" row with its
    /// chevron in front, as in Explorer's tree.
    pub(super) fn style_tree_heading(
        &self,
        section: SidebarSection,
        toggle: &gtk::Button,
        content: &gtk::Box,
        rule: &gtk::Separator,
        chevron: &gtk::Image,
    ) {
        if section != SidebarSection::Devices || !polished(&self.preference_manager) {
            return;
        }
        toggle.add_css_class("explorer-this-pc");
        content.add_css_class("explorer-tree-heading");
        content.reorder_child_after(chevron, None::<&gtk::Widget>);
        let icon = crate::assets::primary_icon("fahras-fluent-this-pc", 16);
        content.insert_child_after(&icon, Some(chevron));
        rule.set_visible(false);
    }

    /// Explorer heads the drives with "This PC".
    pub(super) fn section_title(&self, section: SidebarSection) -> &'static str {
        if section == SidebarSection::Devices && arranged_like_explorer(&self.preference_manager) {
            return "This PC";
        }
        section.title()
    }

    pub(super) fn bind_explorer_arrangement(self: &Rc<Self>) {
        let weak = Rc::downgrade(self);
        self.preference_manager.bind_preference(
            &self.widget,
            |preferences| (arranged_like_explorer(preferences), polished(preferences)),
            move |widget, _| {
                if let Some(state) = weak.upgrade()
                    && widget.first_child().is_some()
                {
                    state.rebuild();
                }
            },
        );
    }
}

/// Explorer marks the user folders it keeps in Quick access with a pin.
fn mark_pinned(row: &gtk::Button) {
    let Some(content) = row.child().and_downcast::<gtk::Box>() else {
        return;
    };
    let pin = crate::assets::primary_icon("fahras-fluent-pin-symbolic", 14);
    pin.add_css_class("explorer-pin");
    pin.set_hexpand(true);
    pin.set_halign(gtk::Align::End);
    crate::ui::accessibility::set_description(
        &pin,
        Some(&crate::i18n::tr("fahras.sidebar.pinned")),
    );
    content.append(&pin);
}

fn sidebar_rows(root: &gtk::Widget) -> Vec<gtk::Widget> {
    let mut rows = Vec::new();
    let mut pending = vec![root.clone()];
    while let Some(widget) = pending.pop() {
        if widget.has_css_class("sidebar-row") {
            rows.push(widget.clone());
        }
        pending.extend(std::iter::successors(
            widget.first_child(),
            gtk::Widget::next_sibling,
        ));
    }
    rows
}

/// A top-level tree row: a collapsed chevron before its icon.
fn tree_row(row: &gtk::Widget) {
    let button = if row.has_css_class("sidebar-row") {
        Some(row.clone())
    } else {
        std::iter::successors(row.first_child(), gtk::Widget::next_sibling)
            .find(|child| child.has_css_class("sidebar-row"))
    };
    let Some(content) = button
        .and_then(|button| button.downcast::<gtk::Button>().ok())
        .and_then(|button| button.child())
        .and_downcast::<gtk::Box>()
    else {
        return;
    };
    row.add_css_class("explorer-tree-row");
    let chevron = crate::assets::primary_icon(crate::assets::icons::CHEVRON_RIGHT, 14);
    chevron.add_css_class("sidebar-heading-chevron");
    content.prepend(&chevron);
}
