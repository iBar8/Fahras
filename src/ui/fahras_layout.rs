// SPDX-License-Identifier: MIT

//! Chrome the Explorer layout replaces with its own.

use gtk::prelude::*;

use crate::{fahras::look::ExplorerSkin, ui::preferences::PreferenceManager};

/// Hidden in 10xer mode, like `tenxer_mode::hide_while_enabled`, and also
/// while an Explorer look shows its own address row and command bar.
pub(crate) fn hide_with_tenxer_or_explorer(widget: &impl IsA<gtk::Widget>) {
    widget.add_css_class("tenxer-suppressed-chrome");
    PreferenceManager::shared().bind_preference(
        widget,
        |preferences| preferences.tenxer_mode() || preferences.explorer_skin() != ExplorerSkin::Off,
        |widget, hidden| {
            widget.set_visible(!hidden);
            widget.set_sensitive(!hidden);
        },
    );
}
