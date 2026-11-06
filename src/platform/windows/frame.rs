// SPDX-License-Identifier: MIT

//! The Windows title bar of an Explorer look: dark with a dark theme, and on
//! Windows 11 the Mica material Explorer's own title bar uses. GTK paints the
//! window's content opaque, so the material shows in the title bar only.

#![expect(unsafe_code, reason = "window frames are a Win32 API")]

use std::cell::Cell;

use gtk::prelude::*;
use windows_sys::Win32::Graphics::Dwm::{
    DWMSBT_AUTO, DWMSBT_MAINWINDOW, DWMWA_SYSTEMBACKDROP_TYPE, DWMWA_USE_IMMERSIVE_DARK_MODE,
    DwmSetWindowAttribute,
};

#[derive(Clone, Copy, Default, PartialEq, Eq)]
struct Frame {
    dark: bool,
    mica: bool,
}

thread_local! {
    static FRAME: Cell<Frame> = Cell::new(Frame::default());
}

fn apply(window: &gtk::Window, frame: Frame) {
    let Some(surface) = window.surface().and_downcast::<gdk4_win32::Win32Surface>() else {
        return;
    };
    let hwnd = surface.handle().0;
    let dark: i32 = frame.dark.into();
    let backdrop = if frame.mica {
        DWMSBT_MAINWINDOW
    } else {
        DWMSBT_AUTO
    };
    // SAFETY: the window is realized, and each value is the size passed with it.
    // Older Windows ignores attributes it does not know.
    unsafe {
        DwmSetWindowAttribute(
            hwnd,
            DWMWA_USE_IMMERSIVE_DARK_MODE as u32,
            std::ptr::from_ref(&dark).cast(),
            size_of::<i32>() as u32,
        );
        DwmSetWindowAttribute(
            hwnd,
            DWMWA_SYSTEMBACKDROP_TYPE as u32,
            std::ptr::from_ref(&backdrop).cast(),
            size_of_val(&backdrop) as u32,
        );
    }
}

/// Restyles every open window's title bar for the look and theme.
pub(crate) fn follow_look(dark: bool, mica: bool) {
    let frame = Frame { dark, mica };
    if FRAME.with(|current| current.replace(frame)) == frame {
        return;
    }
    for window in gtk::Window::list_toplevels() {
        if let Ok(window) = window.downcast::<gtk::Window>()
            && window.is_realized()
        {
            apply(&window, frame);
        }
    }
}

/// Styles a window's title bar as it appears.
pub(crate) fn style_when_shown(window: &impl IsA<gtk::Window>) {
    let window = window.as_ref();
    if window.is_realized() {
        apply(window, FRAME.with(Cell::get));
    }
    window.connect_realize(|window| apply(window, FRAME.with(Cell::get)));
}
