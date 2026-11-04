// SPDX-License-Identifier: MIT

use super::PreferenceManager;

impl PreferenceManager {
    /// The chosen bundled Arabic family; `None` keeps the default.
    pub fn arabic_font(&self) -> Option<String> {
        self.preferences.borrow().arabic_font.clone()
    }

    pub fn set_arabic_font(&self, family: Option<String>) {
        let family = super::stored::normalized_font_family(family)
            .filter(|family| family != crate::fahras::arabic_fonts::DEFAULT_FAMILY);
        if self.arabic_font() == family {
            return;
        }
        self.preferences.borrow_mut().arabic_font = family;
        self.save_preferences();
    }
}
