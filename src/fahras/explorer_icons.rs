// SPDX-License-Identifier: MIT

//! Microsoft's Fluent UI icons (MIT) for the Explorer looks: colored icons in
//! place of Strata's file and place icons, and symbolic ones for the toolbar.

use std::cell::Cell;

use crate::assets::icons;

pub(crate) const PREFIX: &str = "fahras-fluent-";
const RESOURCE: &str = "/io/github/lgse/Strata/icons/fluent";
/// The fill Fluent's monochrome icons are drawn with.
const SYMBOLIC_FILL: &str = "#212121";

thread_local! {
    static ACTIVE: Cell<bool> = const { Cell::new(false) };
}

pub(crate) fn set_active(active: bool) {
    ACTIVE.with(|current| current.set(active));
}

pub(crate) fn is_active() -> bool {
    ACTIVE.with(Cell::get)
}

/// The Fluent icon an Explorer look shows for one of Strata's icons.
pub(crate) fn substitute(name: &str) -> Option<&'static str> {
    if !is_active() {
        return None;
    }
    if name.starts_with(crate::assets::CATPPUCCIN_ICON_PREFIX) {
        return Some("fahras-fluent-code");
    }
    Some(match name {
        icons::FOLDER
        | icons::FOLDER_OPEN
        | icons::FOLDER_INPUT
        | icons::FOLDER_OUTPUT
        | icons::FOLDER_PLUS => "fahras-fluent-folder",
        icons::FOLDER_ARCHIVE | icons::FILE_ARCHIVE => "fahras-fluent-folder-zip",
        icons::PICTURES => "fahras-fluent-image",
        icons::VIDEOS => "fahras-fluent-video",
        icons::FILE_AUDIO | icons::MUSIC => "fahras-fluent-music",
        icons::DOCUMENTS | icons::FILE_TYPE => "fahras-fluent-document-text",
        icons::FILE_SPREADSHEET => "fahras-fluent-table",
        icons::FILE_CODE | icons::CODE_XML | icons::FILE_TERMINAL => "fahras-fluent-code",
        icons::PRESENTATION | icons::LIBRARY => "fahras-fluent-book",
        icons::BOX | icons::KEY_ROUND | icons::FILE_PLUS => "fahras-fluent-document",
        icons::HARD_DRIVE | icons::DISC => "fahras-fluent-drive",
        icons::HOME => "fahras-fluent-home",
        icons::NETWORK | icons::GLOBE => "fahras-fluent-network",
        icons::DOWNLOADS => "fahras-fluent-downloads",
        icons::MONITOR => "fahras-fluent-this-pc",
        _ => return None,
    })
}

/// The SVG for a Fluent icon; symbolic ones take `color`, colored ones keep
/// their own palette.
pub(crate) fn svg(name: &str, color: &str) -> Option<String> {
    let data = gtk::gio::resources_lookup_data(
        &format!("{RESOURCE}/{name}.svg"),
        gtk::gio::ResourceLookupFlags::NONE,
    )
    .ok()?;
    // Toolbar icons come on a 20-unit grid; their viewBox scales them to 24.
    let source = std::str::from_utf8(data.as_ref()).ok()?.replacen(
        "width=\"20\" height=\"20\"",
        "width=\"24\" height=\"24\"",
        1,
    );
    Some(if name.ends_with("-symbolic") {
        source.replace(SYMBOLIC_FILL, color)
    } else {
        source
    })
}

#[cfg(test)]
mod tests;
