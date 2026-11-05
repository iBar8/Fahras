// SPDX-License-Identifier: MIT

use super::PreferenceManager;
use crate::fahras::look::{AppIcon, ExplorerSkin};

impl PreferenceManager {
    pub fn app_icon(&self) -> AppIcon {
        self.preferences.borrow().app_icon
    }

    pub fn set_app_icon(&self, icon: AppIcon) {
        if self.app_icon() == icon {
            return;
        }
        self.preferences.borrow_mut().app_icon = icon;
        self.save_preferences();
    }

    pub fn explorer_skin(&self) -> ExplorerSkin {
        self.preferences.borrow().explorer_skin
    }

    pub fn set_explorer_skin(&self, skin: ExplorerSkin) {
        if self.explorer_skin() == skin {
            return;
        }
        self.preferences.borrow_mut().explorer_skin = skin;
        self.save_preferences();
    }
}
