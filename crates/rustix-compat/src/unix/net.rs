// SPDX-License-Identifier: MIT

//! Unix sockets are not exposed by std on Windows; connecting reports
//! `Unsupported`, so features built on them are unavailable.

use std::{io, path::Path};

#[derive(Debug)]
enum Never {}

/// Never constructed: every way to obtain one fails with `Unsupported`.
#[derive(Debug)]
pub struct UnixStream(Never);

impl From<UnixStream> for crate::fd::OwnedFd {
    fn from(stream: UnixStream) -> Self {
        match stream.0 {}
    }
}

impl From<crate::fd::OwnedFd> for UnixStream {
    fn from(_fd: crate::fd::OwnedFd) -> Self {
        panic!("Unix sockets are not available on Windows")
    }
}

impl crate::fd::AsFd for UnixStream {
    fn as_fd(&self) -> crate::fd::BorrowedFd<'_> {
        match self.0 {}
    }
}

impl UnixStream {
    pub fn connect<P: AsRef<Path>>(_path: P) -> io::Result<Self> {
        Err(io::ErrorKind::Unsupported.into())
    }

    pub fn pair() -> io::Result<(Self, Self)> {
        Err(io::ErrorKind::Unsupported.into())
    }

    pub fn try_clone(&self) -> io::Result<Self> {
        Err(io::ErrorKind::Unsupported.into())
    }

    pub fn set_nonblocking(&self, _nonblocking: bool) -> io::Result<()> {
        Err(io::ErrorKind::Unsupported.into())
    }

    pub fn set_read_timeout(&self, _timeout: Option<std::time::Duration>) -> io::Result<()> {
        Err(io::ErrorKind::Unsupported.into())
    }

    pub fn set_write_timeout(&self, _timeout: Option<std::time::Duration>) -> io::Result<()> {
        Err(io::ErrorKind::Unsupported.into())
    }

    pub fn shutdown(&self, _how: std::net::Shutdown) -> io::Result<()> {
        Err(io::ErrorKind::Unsupported.into())
    }
}

impl io::Read for UnixStream {
    fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
        Err(io::ErrorKind::Unsupported.into())
    }
}

impl io::Write for UnixStream {
    fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
        Err(io::ErrorKind::Unsupported.into())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl io::Read for &UnixStream {
    fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
        Err(io::ErrorKind::Unsupported.into())
    }
}

impl io::Write for &UnixStream {
    fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
        Err(io::ErrorKind::Unsupported.into())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[derive(Debug)]
pub struct UnixListener(Never);

impl UnixListener {
    pub fn bind<P: AsRef<Path>>(_path: P) -> io::Result<Self> {
        Err(io::ErrorKind::Unsupported.into())
    }
}
