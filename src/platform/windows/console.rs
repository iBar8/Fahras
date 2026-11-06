// SPDX-License-Identifier: MIT

//! Fahras is a windowed program, so it opens no console of its own. Started
//! from a terminal, it writes `--version`, `--help` and its log there.

#![expect(unsafe_code, reason = "the Windows console is a Win32 API")]

use windows_sys::Win32::{
    Foundation::{GENERIC_READ, GENERIC_WRITE, INVALID_HANDLE_VALUE},
    Storage::FileSystem::{
        CreateFileW, FILE_ATTRIBUTE_NORMAL, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING,
    },
    System::Console::{
        ATTACH_PARENT_PROCESS, AttachConsole, GetStdHandle, STD_ERROR_HANDLE, STD_OUTPUT_HANDLE,
        SetStdHandle,
    },
};

/// Writes to the terminal Fahras was started from, unless its output already
/// goes to a file or pipe.
pub(crate) fn use_parent_terminal() {
    // SAFETY: plain Win32 calls on this process's own standard handles.
    unsafe {
        let output = GetStdHandle(STD_OUTPUT_HANDLE);
        if !(output.is_null() || output == INVALID_HANDLE_VALUE) {
            return;
        }
        if AttachConsole(ATTACH_PARENT_PROCESS) == 0 {
            return;
        }
        let name: Vec<u16> = "CONOUT$\0".encode_utf16().collect();
        let console = CreateFileW(
            name.as_ptr(),
            GENERIC_READ | GENERIC_WRITE,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            std::ptr::null(),
            OPEN_EXISTING,
            FILE_ATTRIBUTE_NORMAL,
            std::ptr::null_mut(),
        );
        if console != INVALID_HANDLE_VALUE {
            SetStdHandle(STD_OUTPUT_HANDLE, console);
            SetStdHandle(STD_ERROR_HANDLE, console);
        }
    }
}
