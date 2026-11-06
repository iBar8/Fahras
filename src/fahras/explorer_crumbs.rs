// SPDX-License-Identifier: MIT

//! The address bar as File Explorer draws it: `This PC › Windows (C:) › …`,
//! with every crumb clickable.

use gtk::{gio, prelude::*};

use crate::model::Location;

fn polished() -> bool {
    super::explorer_details::is_polished()
}

/// Explorer separates crumbs with a chevron; bidi mirroring turns it around
/// right to left.
pub(crate) fn separator() -> Option<&'static str> {
    polished().then_some("›")
}

/// Explorer always shows the whole path from the drive, never from the home folder.
pub(crate) fn shows_full_path() -> bool {
    polished()
}

/// A drive or mounted volume by its system name, such as "Windows (C:)".
pub(crate) fn crumb_label(crumb: &Location) -> Option<String> {
    if !polished() {
        return None;
    }
    let path = crumb.native_path()?;
    let file = gio::File::for_path(path);
    gio::VolumeMonitor::get()
        .mounts()
        .into_iter()
        .find(|mount| mount.root().equal(&file))
        .map(|mount| mount.name().to_string())
}

/// The "This PC" crumb Explorer starts local paths with on Windows.
pub(crate) fn leading_crumb(location: &Location) -> Option<gtk::Box> {
    if !(polished() && cfg!(windows) && location.native_path().is_some()) {
        return None;
    }
    let crumb = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    crumb.add_css_class("explorer-this-pc-crumb");
    crumb.append(&crate::assets::primary_icon(
        crate::assets::icons::MONITOR,
        16,
    ));
    let label = gtk::Label::new(Some(&crate::i18n::tr("This PC")));
    label.add_css_class("breadcrumb");
    crumb.append(&label);
    Some(crumb)
}
