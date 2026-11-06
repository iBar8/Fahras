// SPDX-License-Identifier: MIT

//! File and directory handles that remember where they were opened, so the
//! POSIX `*at` calls can resolve names relative to them.

use std::{
    fs::File,
    io,
    os::windows::io::{AsRawHandle, FromRawHandle, IntoRawHandle, RawHandle},
    path::{Path, PathBuf},
    rc::Rc,
    sync::{
        Arc,
        atomic::{AtomicU32, Ordering},
    },
};

pub type RawFd = isize;

#[derive(Debug)]
pub(crate) struct FdInner {
    pub(crate) file: Option<File>,
    /// `None` is the process working directory.
    pub(crate) path: Option<PathBuf>,
    pub(crate) flags: AtomicU32,
}

static CWD_INNER: FdInner = FdInner {
    file: None,
    path: None,
    flags: AtomicU32::new(0),
};

pub(crate) const CWD: BorrowedFd<'static> = BorrowedFd {
    repr: Repr::Inner(&CWD_INNER),
};

#[derive(Debug)]
pub struct OwnedFd {
    inner: Box<FdInner>,
}

#[derive(Clone, Copy, Debug)]
enum Repr<'a> {
    Inner(&'a FdInner),
    File(&'a File),
    /// A handle that is not a file, such as a child process pipe.
    Handle(isize, std::marker::PhantomData<&'a ()>),
}

#[derive(Clone, Copy, Debug)]
pub struct BorrowedFd<'a> {
    repr: Repr<'a>,
}

impl OwnedFd {
    pub(crate) fn new(file: File, path: Option<PathBuf>, flags: u32) -> Self {
        Self {
            inner: Box::new(FdInner {
                file: Some(file),
                path,
                flags: AtomicU32::new(flags),
            }),
        }
    }

    pub fn try_clone(&self) -> io::Result<Self> {
        self.as_fd().try_clone_to_owned()
    }
}

impl<'a> BorrowedFd<'a> {
    /// # Safety
    /// Windows handles carry no descriptor table; this only exists for API parity.
    pub unsafe fn borrow_raw(_fd: RawFd) -> Self {
        CWD
    }

    pub fn try_clone_to_owned(&self) -> io::Result<OwnedFd> {
        let file = self.file()?.try_clone()?;
        Ok(OwnedFd::new(
            file,
            self.path().map(Path::to_path_buf),
            self.flags(),
        ))
    }

    pub(crate) fn file(&self) -> io::Result<&'a File> {
        match self.repr {
            Repr::Inner(inner) => inner
                .file
                .as_ref()
                .ok_or_else(|| crate::io::Errno::BADF.into()),
            Repr::File(file) => Ok(file),
            Repr::Handle(..) => Err(crate::io::Errno::BADF.into()),
        }
    }

    pub(crate) fn path(&self) -> Option<&'a Path> {
        match self.repr {
            Repr::Inner(inner) => inner.path.as_deref(),
            Repr::File(_) | Repr::Handle(..) => None,
        }
    }

    pub(crate) fn is_cwd(&self) -> bool {
        matches!(self.repr, Repr::Inner(inner) if inner.file.is_none() && inner.path.is_none())
    }

    pub(crate) fn flags(&self) -> u32 {
        match self.repr {
            Repr::Inner(inner) => inner.flags.load(Ordering::Relaxed),
            Repr::File(_) | Repr::Handle(..) => 0,
        }
    }

    pub(crate) fn set_flags(&self, flags: u32) {
        if let Repr::Inner(inner) = self.repr {
            inner.flags.store(flags, Ordering::Relaxed);
        }
    }

    /// The path this handle refers to, asking Windows when it was not opened by path.
    pub(crate) fn location(&self) -> io::Result<Option<PathBuf>> {
        if self.is_cwd() {
            return Ok(None);
        }
        match self.path() {
            Some(path) => Ok(Some(path.to_path_buf())),
            None => crate::win::final_path(self.file()?).map(Some),
        }
    }

    pub(crate) fn from_handle<T: AsRawHandle + ?Sized>(handle: &'a T) -> Self {
        Self {
            repr: Repr::Handle(handle.as_raw_handle() as isize, std::marker::PhantomData),
        }
    }
}

macro_rules! handle_as_fd {
    ($($type:ty),+) => {
        $(impl AsFd for $type {
            fn as_fd(&self) -> BorrowedFd<'_> {
                BorrowedFd::from_handle(self)
            }
        })+
    };
}

handle_as_fd!(
    std::process::ChildStdin,
    std::process::ChildStdout,
    std::process::ChildStderr,
    std::io::Stdin,
    std::io::Stdout,
    std::io::Stderr
);

pub trait AsFd {
    fn as_fd(&self) -> BorrowedFd<'_>;
}

pub trait AsRawFd {
    fn as_raw_fd(&self) -> RawFd;
}

pub trait IntoRawFd {
    fn into_raw_fd(self) -> RawFd;
}

pub trait FromRawFd {
    /// # Safety
    /// `fd` must be an open Windows handle that the caller transfers.
    unsafe fn from_raw_fd(fd: RawFd) -> Self;
}

impl AsFd for OwnedFd {
    fn as_fd(&self) -> BorrowedFd<'_> {
        BorrowedFd {
            repr: Repr::Inner(&self.inner),
        }
    }
}

impl AsFd for BorrowedFd<'_> {
    fn as_fd(&self) -> BorrowedFd<'_> {
        *self
    }
}

impl AsFd for File {
    fn as_fd(&self) -> BorrowedFd<'_> {
        BorrowedFd {
            repr: Repr::File(self),
        }
    }
}

impl<T: AsFd + ?Sized> AsFd for &T {
    fn as_fd(&self) -> BorrowedFd<'_> {
        (**self).as_fd()
    }
}

impl<T: AsFd + ?Sized> AsFd for &mut T {
    fn as_fd(&self) -> BorrowedFd<'_> {
        (**self).as_fd()
    }
}

impl<T: AsFd + ?Sized> AsFd for Box<T> {
    fn as_fd(&self) -> BorrowedFd<'_> {
        (**self).as_fd()
    }
}

impl<T: AsFd + ?Sized> AsFd for Arc<T> {
    fn as_fd(&self) -> BorrowedFd<'_> {
        (**self).as_fd()
    }
}

impl<T: AsFd + ?Sized> AsFd for Rc<T> {
    fn as_fd(&self) -> BorrowedFd<'_> {
        (**self).as_fd()
    }
}

fn raw(handle: RawHandle) -> RawFd {
    handle as RawFd
}

impl AsRawFd for BorrowedFd<'_> {
    fn as_raw_fd(&self) -> RawFd {
        match self.repr {
            Repr::Handle(handle, _) => handle,
            _ => self.file().map_or(-100, |file| raw(file.as_raw_handle())),
        }
    }
}

impl AsRawFd for OwnedFd {
    fn as_raw_fd(&self) -> RawFd {
        self.as_fd().as_raw_fd()
    }
}

impl AsRawFd for File {
    fn as_raw_fd(&self) -> RawFd {
        raw(self.as_raw_handle())
    }
}

impl IntoRawFd for OwnedFd {
    fn into_raw_fd(self) -> RawFd {
        self.inner
            .file
            .map_or(-1, |file| raw(file.into_raw_handle()))
    }
}

impl FromRawFd for OwnedFd {
    unsafe fn from_raw_fd(fd: RawFd) -> Self {
        // SAFETY: the caller transfers ownership of an open handle.
        let file = unsafe { File::from_raw_handle(fd as RawHandle) };
        Self::new(file, None, 0)
    }
}

impl From<File> for OwnedFd {
    fn from(file: File) -> Self {
        Self::new(file, None, 0)
    }
}

impl From<OwnedFd> for File {
    fn from(fd: OwnedFd) -> Self {
        fd.inner
            .file
            .expect("only the working directory handle has no file, and it is never owned")
    }
}

impl From<OwnedFd> for std::process::Stdio {
    fn from(fd: OwnedFd) -> Self {
        File::from(fd).into()
    }
}
