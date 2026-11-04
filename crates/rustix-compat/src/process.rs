// SPDX-License-Identifier: MIT

//! Process identity and signals. Windows has no uids or signals; ownership
//! checks see one user, and signalling other processes is unsupported.

use bitflags::bitflags;

use crate::{
    fd::OwnedFd,
    io::{Errno, Result},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Pid(i32);

impl Pid {
    pub const INIT: Self = Self(1);

    pub const fn from_raw(raw: i32) -> Option<Self> {
        if raw > 0 { Some(Self(raw)) } else { None }
    }

    /// # Safety
    /// `raw` must be positive.
    pub const unsafe fn from_raw_unchecked(raw: i32) -> Self {
        Self(raw)
    }

    pub fn from_child(child: &std::process::Child) -> Self {
        Self(child.id() as i32)
    }

    pub const fn as_raw_nonzero(self) -> std::num::NonZeroI32 {
        match std::num::NonZeroI32::new(self.0) {
            Some(value) => value,
            None => std::num::NonZeroI32::MIN,
        }
    }

    pub const fn as_raw(pid: Option<Self>) -> i32 {
        match pid {
            Some(pid) => pid.0,
            None => 0,
        }
    }

    pub const fn is_init(self) -> bool {
        self.0 == 1
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Uid(u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Gid(u32);

impl Uid {
    pub const ROOT: Self = Self(0);

    pub const fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    pub const fn as_raw(self) -> u32 {
        self.0
    }

    pub const fn is_root(self) -> bool {
        self.0 == 0
    }
}

impl Gid {
    pub const ROOT: Self = Self(0);

    pub const fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    pub const fn as_raw(self) -> u32 {
        self.0
    }

    pub const fn is_root(self) -> bool {
        self.0 == 0
    }
}

/// Files on Windows report uid and gid 0, so the user is their owner.
pub fn getuid() -> Uid {
    Uid(0)
}

pub fn geteuid() -> Uid {
    Uid(0)
}

pub fn getgid() -> Gid {
    Gid(0)
}

pub fn getegid() -> Gid {
    Gid(0)
}

pub fn getgroups() -> Result<Vec<Gid>> {
    Ok(Vec::new())
}

pub fn getpid() -> Pid {
    Pid(std::process::id() as i32)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Signal(i32);

impl Signal {
    pub const HUP: Self = Self(1);
    pub const INT: Self = Self(2);
    pub const QUIT: Self = Self(3);
    pub const KILL: Self = Self(9);
    pub const USR1: Self = Self(10);
    pub const USR2: Self = Self(12);
    pub const TERM: Self = Self(15);
    pub const CHILD: Self = Self(17);
    pub const CONT: Self = Self(18);
    pub const STOP: Self = Self(19);

    pub const fn as_raw(self) -> i32 {
        self.0
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct PidfdFlags: u32 {
        const NONBLOCK = 0o4000;
        const _ = !0;
    }
}

pub fn kill_process(_pid: Pid, _signal: Signal) -> Result<()> {
    Err(Errno::NOSYS)
}

pub fn kill_process_group(_pid: Pid, _signal: Signal) -> Result<()> {
    Err(Errno::NOSYS)
}

pub fn pidfd_open(_pid: Pid, _flags: PidfdFlags) -> Result<OwnedFd> {
    Err(Errno::NOSYS)
}

pub fn pidfd_send_signal<Fd: crate::fd::AsFd>(_pidfd: Fd, _signal: Signal) -> Result<()> {
    Err(Errno::NOSYS)
}

pub fn setpriority_process(_pid: Option<Pid>, _priority: i32) -> Result<()> {
    Ok(())
}
