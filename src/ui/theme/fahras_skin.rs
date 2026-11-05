// SPDX-License-Identifier: MIT

//! Fahras's Explorer skins: File Explorer's colors, in light or dark to match
//! the chosen theme, and a stylesheet that reshapes the toolbar, sidebar and
//! file list.

use gtk::gdk;

use super::{TOKEN_STYLE_PRIORITY, ThemeTokens, parse_rgb_channels};
use crate::fahras::look::ExplorerSkin;

const WINDOWS_10_CSS: &str = include_str!("../../../branding/skins/windows10.css");
const WINDOWS_11_CSS: &str = include_str!("../../../branding/skins/windows11.css");

/// background, surface, text, accent, danger, muted, highlight, border, dim text.
type Palette = [&'static str; 9];

const WINDOWS_11_LIGHT: Palette = [
    "#ffffff", "#f3f3f3", "#1a1a1a", "#005fb8", "#c42b1c", "#ebebeb", "#e6e6e6", "#e5e5e5",
    "#616161",
];
const WINDOWS_11_DARK: Palette = [
    "#191919", "#202020", "#ffffff", "#60cdff", "#ff99a4", "#1c1c1c", "#2d2d2d", "#333333",
    "#a0a0a0",
];
const WINDOWS_10_LIGHT: Palette = [
    "#ffffff", "#f5f6f7", "#000000", "#0078d7", "#e81123", "#f0f0f0", "#e5f3ff", "#d9d9d9",
    "#6d6d6d",
];
const WINDOWS_10_DARK: Palette = [
    "#191919", "#2b2b2b", "#ffffff", "#3a96dd", "#e81123", "#202020", "#3a3a3a", "#434343",
    "#a0a0a0",
];

thread_local! {
    static PROVIDER: gtk::CssProvider = gtk::CssProvider::new();
    static INSTALLED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    static LOADED: std::cell::Cell<Option<ExplorerSkin>> = const { std::cell::Cell::new(None) };
}

/// The theme's tokens with the skin's palette, keeping the theme's name and
/// syntax colors. Light or dark follows the theme.
pub(super) fn tokens(skin: ExplorerSkin, theme: &ThemeTokens) -> ThemeTokens {
    let Some(palette) = palette(skin, is_dark(&theme.background)) else {
        return theme.clone();
    };
    let [
        background,
        surface,
        text,
        accent,
        danger,
        muted,
        highlight,
        border,
        dim_text,
    ] = palette.map(str::to_owned);
    ThemeTokens {
        background,
        surface,
        text,
        accent,
        danger,
        muted,
        highlight,
        border,
        dim_text,
        ..theme.clone()
    }
}

/// Explorer's own font on Windows, unless the person chose an interface font.
pub(super) fn interface_font(skin: ExplorerSkin, chosen: Option<String>) -> Option<String> {
    if chosen.is_none() && skin != ExplorerSkin::Off && cfg!(windows) {
        return Some("Segoe UI".to_owned());
    }
    chosen
}

pub(super) fn apply_stylesheet(skin: ExplorerSkin) {
    PROVIDER.with(|provider| {
        if !INSTALLED.replace(true)
            && let Some(display) = gdk::Display::default()
        {
            // Added after the token provider, so it wins at the same priority.
            gtk::style_context_add_provider_for_display(&display, provider, TOKEN_STYLE_PRIORITY);
        }
        if LOADED.replace(Some(skin)) != Some(skin) {
            provider.load_from_string(stylesheet(skin));
        }
    });
}

fn stylesheet(skin: ExplorerSkin) -> &'static str {
    match skin {
        ExplorerSkin::Off => "",
        ExplorerSkin::Windows10 => WINDOWS_10_CSS,
        ExplorerSkin::Windows11 => WINDOWS_11_CSS,
    }
}

fn palette(skin: ExplorerSkin, dark: bool) -> Option<Palette> {
    match (skin, dark) {
        (ExplorerSkin::Off, _) => None,
        (ExplorerSkin::Windows10, false) => Some(WINDOWS_10_LIGHT),
        (ExplorerSkin::Windows10, true) => Some(WINDOWS_10_DARK),
        (ExplorerSkin::Windows11, false) => Some(WINDOWS_11_LIGHT),
        (ExplorerSkin::Windows11, true) => Some(WINDOWS_11_DARK),
    }
}

fn is_dark(background: &str) -> bool {
    parse_rgb_channels(background).is_none_or(|[red, green, blue]| {
        u32::from(red) * 299 + u32::from(green) * 587 + u32::from(blue) * 114 < 128_000
    })
}

#[cfg(test)]
mod tests;
