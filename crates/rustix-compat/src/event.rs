// SPDX-License-Identifier: MIT

//! eventfd and poll have no Windows equivalent for arbitrary handles.

use bitflags::bitflags;

use crate::{
    fd::{AsFd, BorrowedFd, OwnedFd},
    io::{Errno, Result},
};

pub use crate::fs::Timespec;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct EventfdFlags: u32 {
        const CLOEXEC = 0o2000000;
        const NONBLOCK = 0o4000;
        const SEMAPHORE = 1;
        const _ = !0;
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct PollFlags: u16 {
        const IN = 0x1;
        const PRI = 0x2;
        const OUT = 0x4;
        const ERR = 0x8;
        const HUP = 0x10;
        const NVAL = 0x20;
        const RDHUP = 0x2000;
        const _ = !0;
    }
}

pub fn eventfd(_initval: u32, _flags: EventfdFlags) -> Result<OwnedFd> {
    Err(Errno::NOSYS)
}

pub struct PollFd<'fd> {
    fd: BorrowedFd<'fd>,
    events: PollFlags,
    revents: PollFlags,
}

impl<'fd> PollFd<'fd> {
    pub fn new<Fd: AsFd>(fd: &'fd Fd, events: PollFlags) -> Self {
        Self {
            fd: fd.as_fd(),
            events,
            revents: PollFlags::empty(),
        }
    }

    pub fn from_borrowed_fd(fd: BorrowedFd<'fd>, events: PollFlags) -> Self {
        Self {
            fd,
            events,
            revents: PollFlags::empty(),
        }
    }

    pub fn revents(&self) -> PollFlags {
        let _ = (self.fd, self.events);
        self.revents
    }
}

pub fn poll(_fds: &mut [PollFd<'_>], _timeout: Option<&Timespec>) -> Result<usize> {
    Err(Errno::NOSYS)
}
