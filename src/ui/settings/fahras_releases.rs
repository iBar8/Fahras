// SPDX-License-Identifier: MIT

//! The Updates page of a Linux build, which does not update itself.

use gtk::{gio, prelude::*};

use super::{append_heading, page_content, scrollable_page, search};

pub(super) fn releases_page() -> gtk::Widget {
    let content = page_content();
    content.add_css_class("settings-updates-page");
    append_heading(&content, "STATUS");
    let row = gtk::Box::new(gtk::Orientation::Vertical, 8);
    row.add_css_class("settings-option");
    search::tag(&row, "Check for updates");
    let title = gtk::Label::new(Some(&crate::i18n::tr("fahras.releases.title")));
    title.set_xalign(0.0);
    title.add_css_class("settings-option-title");
    let description = gtk::Label::new(Some(&crate::i18n::tr("fahras.releases.description")));
    description.set_xalign(0.0);
    description.set_wrap(true);
    description.add_css_class("settings-option-description");
    let open = gtk::Button::with_label(&crate::i18n::tr("fahras.releases.open"));
    open.add_css_class("settings-update-check");
    open.set_halign(gtk::Align::Start);
    open.connect_clicked(open_releases_page);
    row.append(&title);
    row.append(&description);
    row.append(&open);
    content.append(&row);
    scrollable_page(&content, None)
}

fn open_releases_page(button: &gtk::Button) {
    let context = crate::child_environment::launch_context(&button.display());
    if let Err(error) =
        gio::AppInfo::launch_default_for_uri(crate::fahras::updates::RELEASES_PAGE, Some(&context))
    {
        tracing::warn!(%error, "could not open the Fahras Releases page");
    }
}

#[cfg(test)]
mod tests;
