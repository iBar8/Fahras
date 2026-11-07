// SPDX-License-Identifier: MIT

use super::{ListColumnId, PreferenceManager};
use crate::fahras::look::{AppIcon, ExplorerSkin};

const EXPLORER_COLUMNS: [ListColumnId; 5] = [
    ListColumnId::Name,
    ListColumnId::Modified,
    ListColumnId::Type,
    ListColumnId::Size,
    ListColumnId::Mode,
];

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

    /// Turning an Explorer look on opens folders in Details view with
    /// Explorer's columns: Name, Date modified, Type and Size.
    pub fn set_explorer_skin(&self, skin: ExplorerSkin) {
        let previous = self.explorer_skin();
        if previous == skin {
            return;
        }
        {
            let mut preferences = self.preferences.borrow_mut();
            preferences.explorer_skin = skin;
            if previous == ExplorerSkin::Off {
                preferences.browser_mode = "list".to_owned();
                preferences.list_column_order = EXPLORER_COLUMNS.to_vec();
                preferences.list_hidden_columns = vec![ListColumnId::Mode];
            }
            // Explorer opens with This PC folded, its drives a click away.
            if skin == ExplorerSkin::Windows11 {
                preferences.sidebar_devices_collapsed = true;
            }
        }
        self.save_preferences();
    }
}
