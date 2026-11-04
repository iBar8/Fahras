// SPDX-License-Identifier: MIT

use std::{fs::File, os::windows::io::FromRawHandle, ptr};

use bitflags::bitflags;
use windows_sys::Win32::System::Pipes::CreatePipe;

use crate::{
    fd::OwnedFd,
    io::{Errno, Result},
};

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct PipeFlags: u32 {
        const CLOEXEC = 0o2000000;
        const NONBLOCK = 0o4000;
        const DIRECT = 0o40000;
        const _ = !0;
    }
}

/// An anonymous pipe; Windows handles are not inherited unless asked, like `O_CLOEXEC`.
pub fn pipe() -> Result<(OwnedFd, OwnedFd)> {
    let (mut read, mut write) = (ptr::null_mut(), ptr::null_mut());
    // SAFETY: the out pointers are valid and no security attributes are passed.
    if unsafe { CreatePipe(&mut read, &mut write, ptr::null(), 0) } == 0 {
        return Err(Errno::from_io(std::io::Error::last_os_error()));
    }
    // SAFETY: CreatePipe returned two fresh handles that we now own.
    let (read, write) = unsafe { (File::from_raw_handle(read), File::from_raw_handle(write)) };
    Ok((OwnedFd::from(read), OwnedFd::from(write)))
}

pub fn pipe_with(_flags: PipeFlags) -> Result<(OwnedFd, OwnedFd)> {
    pipe()
}
