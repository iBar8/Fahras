// SPDX-License-Identifier: MIT

use crate::process::Pid;

pub fn gettid() -> Pid {
    // SAFETY: GetCurrentThreadId has no preconditions.
    let id = unsafe { windows_sys::Win32::System::Threading::GetCurrentThreadId() };
    Pid::from_raw(id as i32).unwrap_or(Pid::INIT)
}
