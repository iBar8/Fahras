// SPDX-License-Identifier: MIT

//! Explorer's tabs inside the title bar, in the Windows 11 look. Windows keeps
//! drawing the frame and its minimize, maximize and close buttons; the window's
//! content extends up into the title area, where the tab row is drawn, and GTK
//! leaves that row see-through. `FAHRAS_TITLE_TABS=0` turns it off.

#![expect(unsafe_code, reason = "window frames are a Win32 API")]

use gtk::prelude::*;
use windows_sys::Win32::{
    Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM},
    Graphics::Dwm::{DwmDefWindowProc, DwmExtendFrameIntoClientArea},
    UI::{
        Controls::MARGINS,
        Shell::{DefSubclassProc, SetWindowSubclass},
        WindowsAndMessaging::{
            GetSystemMetrics, GetWindowRect, HTCAPTION, HTCLIENT, IsZoomed, NCCALCSIZE_PARAMS,
            SM_CXPADDEDBORDER, SM_CYSIZEFRAME, SWP_FRAMECHANGED, SWP_NOMOVE, SWP_NOSIZE,
            SWP_NOZORDER, SetWindowPos, WM_NCCALCSIZE, WM_NCHITTEST,
        },
    },
};

/// Explorer's tab row, in pixels at 100% scale.
pub(crate) const TITLE_HEIGHT: i32 = 40;
/// Room Windows' minimize, maximize and close take at the end of the row.
pub(crate) const CAPTION_BUTTONS_WIDTH: i32 = 140;
const SUBCLASS_ID: usize = 0x5469_746c;

pub(crate) fn enabled() -> bool {
    std::env::var_os("FAHRAS_TITLE_TABS").is_none_or(|value| value != "0")
}

/// The class a window whose tabs sit in its title bar carries.
pub(crate) const MERGED: &str = "fahras-title-tabs";

unsafe extern "system" fn title_proc(
    window: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    _id: usize,
    _data: usize,
) -> LRESULT {
    unsafe {
        let mut result: LRESULT = 0;
        if DwmDefWindowProc(window, message, wparam, lparam, &mut result) != 0 {
            return result;
        }
        if message == WM_NCCALCSIZE && wparam != 0 {
            let params = &mut *(lparam as *mut NCCALCSIZE_PARAMS);
            let top = params.rgrc[0].top;
            DefSubclassProc(window, message, wparam, lparam);
            // Keep the side and bottom borders; the title area becomes content.
            params.rgrc[0].top = if IsZoomed(window) != 0 {
                top + GetSystemMetrics(SM_CYSIZEFRAME) + GetSystemMetrics(SM_CXPADDEDBORDER)
            } else {
                top
            };
            return 0;
        }
        if message == WM_NCHITTEST {
            let hit = DefSubclassProc(window, message, wparam, lparam);
            if hit == HTCLIENT as LRESULT {
                let mut bounds = RECT::default();
                GetWindowRect(window, &mut bounds);
                let x = (lparam & 0xffff) as i16 as i32;
                let y = ((lparam >> 16) & 0xffff) as i16 as i32;
                // The empty part of the tab row drags the window, as Explorer's does.
                if y - bounds.top < TITLE_HEIGHT
                    && x - bounds.left > (bounds.right - bounds.left) / 2
                {
                    return HTCAPTION as LRESULT;
                }
            }
            return hit;
        }
        DefSubclassProc(window, message, wparam, lparam)
    }
}

/// Extends the window's content into its title bar once it is shown.
pub(crate) fn merge_when_shown(window: &impl IsA<gtk::Window>) {
    window.as_ref().connect_realize(|window| {
        let Some(surface) = window.surface().and_downcast::<gdk4_win32::Win32Surface>() else {
            return;
        };
        let hwnd = surface.handle().0;
        let margins = MARGINS {
            cxLeftWidth: 0,
            cxRightWidth: 0,
            cyTopHeight: TITLE_HEIGHT,
            cyBottomHeight: 0,
        };
        // SAFETY: the window is realized; the subclass lives as long as it does.
        unsafe {
            SetWindowSubclass(hwnd, Some(title_proc), SUBCLASS_ID, 0);
            DwmExtendFrameIntoClientArea(hwnd, &margins);
            SetWindowPos(
                hwnd,
                std::ptr::null_mut(),
                0,
                0,
                0,
                0,
                SWP_FRAMECHANGED | SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER,
            );
        }
        window.add_css_class(MERGED);
        // The tab row now holds the title area: keep it shown.
        let mut pending = vec![window.clone().upcast::<gtk::Widget>()];
        while let Some(widget) = pending.pop() {
            if widget.has_css_class("tab-strip") {
                widget.set_visible(true);
            }
            pending.extend(std::iter::successors(
                widget.first_child(),
                gtk::Widget::next_sibling,
            ));
        }
    });
}
