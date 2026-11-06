// SPDX-License-Identifier: MIT

//! The sidebar as File Explorer arranges it: Home, the user folders, pinned
//! folders, This PC with its drives, then Network.

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

impl SidebarState {
    pub(super) fn append_explorer_places(self: &Rc<Self>) {
        self.append_ordered_place("home");
        self.append_separator();
        for (id, icon) in USER_FOLDERS {
            self.append_explorer_user_folder(id, icon);
        }
        self.append_pinned_places();
        self.append_devices();
        self.append_separator();
        self.append_ordered_place("network");
    }

    fn append_explorer_user_folder(self: &Rc<Self>, id: &'static str, icon: &str) {
        if !self.standard_place_visible(id) {
            return;
        }
        if let Some((_, name, directory)) = standard_place(id)
            && let Some(path) = glib::user_special_dir(directory)
        {
            self.append_reorderable_place(id, icon, &crate::i18n::tr(name), Location::local(path));
        }
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
            arranged_like_explorer,
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
