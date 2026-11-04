// SPDX-License-Identifier: MIT

//! Descriptor passing over Unix sockets. There are no Unix socket streams on
//! Windows (see `unix::net`), so these are never reached; they fail cleanly.

use std::{io, marker::PhantomData, mem::MaybeUninit};

use bitflags::bitflags;

use crate::{
    fd::{AsFd, BorrowedFd, OwnedFd},
    io::{Errno, Result},
};

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct SendFlags: u32 {
        const NOSIGNAL = 0x4000;
        const _ = !0;
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct RecvFlags: u32 {
        const CMSG_CLOEXEC = 0x4000_0000;
        const _ = !0;
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct ReturnFlags: u32 {
        const TRUNC = 0x20;
        const CTRUNC = 0x8;
        const _ = !0;
    }
}

pub enum SendAncillaryMessage<'slice, 'fd> {
    ScmRights(&'slice [BorrowedFd<'fd>]),
}

pub enum RecvAncillaryMessage<'a> {
    ScmRights(std::vec::IntoIter<OwnedFd>),
    #[doc(hidden)]
    Other(PhantomData<&'a ()>),
}

pub struct SendAncillaryBuffer<'buf, 'slice, 'fd> {
    _marker: PhantomData<(&'buf mut [MaybeUninit<u8>], &'slice (), &'fd ())>,
}

impl<'buf, 'slice, 'fd> SendAncillaryBuffer<'buf, 'slice, 'fd> {
    pub fn new(_buffer: &'buf mut [MaybeUninit<u8>]) -> Self {
        Self { _marker: PhantomData }
    }

    pub fn push(&mut self, _message: SendAncillaryMessage<'slice, 'fd>) -> bool {
        false
    }
}

pub struct RecvAncillaryBuffer<'buf> {
    _marker: PhantomData<&'buf mut [MaybeUninit<u8>]>,
}

impl<'buf> RecvAncillaryBuffer<'buf> {
    pub fn new(_buffer: &'buf mut [MaybeUninit<u8>]) -> Self {
        Self { _marker: PhantomData }
    }

    pub fn drain(&mut self) -> std::vec::IntoIter<RecvAncillaryMessage<'buf>> {
        Vec::new().into_iter()
    }
}

pub struct RecvMsg {
    pub bytes: usize,
    pub flags: ReturnFlags,
}

pub fn sendmsg<Fd: AsFd>(
    _socket: Fd,
    _iov: &[io::IoSlice<'_>],
    _control: &mut SendAncillaryBuffer<'_, '_, '_>,
    _flags: SendFlags,
) -> Result<usize> {
    Err(Errno::OPNOTSUPP)
}

pub fn recvmsg<Fd: AsFd>(
    _socket: Fd,
    _iov: &mut [io::IoSliceMut<'_>],
    _control: &mut RecvAncillaryBuffer<'_>,
    _flags: RecvFlags,
) -> Result<RecvMsg> {
    Err(Errno::OPNOTSUPP)
}

/// The control-buffer size rustix computes; any size works for the stubs above.
#[macro_export]
macro_rules! cmsg_space {
    ($($tokens:tt)*) => {
        64
    };
}
