// SPDX-License-Identifier: MIT

use std::{fmt, io};

use crate::fd::{AsFd, OwnedFd};

/// A Linux errno value. Windows errors are translated to the errno Linux
/// reports for the same condition, so errno-based handling stays portable.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Errno(i32);

pub type Result<T> = std::result::Result<T, Errno>;

macro_rules! errnos {
    ($($name:ident = $value:literal, $text:literal;)+) => {
        impl Errno {
            $(pub const $name: Self = Self($value);)+

            fn text(self) -> &'static str {
                match self.0 {
                    $($value => $text,)+
                    _ => "Unknown error",
                }
            }
        }
    };
}

errnos! {
    PERM = 1, "Operation not permitted";
    NOENT = 2, "No such file or directory";
    SRCH = 3, "No such process";
    INTR = 4, "Interrupted system call";
    IO = 5, "Input/output error";
    NXIO = 6, "No such device or address";
    BADF = 9, "Bad file descriptor";
    CHILD = 10, "No child processes";
    AGAIN = 11, "Resource temporarily unavailable";
    NOMEM = 12, "Cannot allocate memory";
    ACCESS = 13, "Permission denied";
    FAULT = 14, "Bad address";
    BUSY = 16, "Device or resource busy";
    EXIST = 17, "File exists";
    XDEV = 18, "Invalid cross-device link";
    NODEV = 19, "No such device";
    NOTDIR = 20, "Not a directory";
    ISDIR = 21, "Is a directory";
    INVAL = 22, "Invalid argument";
    NFILE = 23, "Too many open files in system";
    MFILE = 24, "Too many open files";
    NOTTY = 25, "Inappropriate ioctl for device";
    TXTBSY = 26, "Text file busy";
    FBIG = 27, "File too large";
    NOSPC = 28, "No space left on device";
    SPIPE = 29, "Illegal seek";
    ROFS = 30, "Read-only file system";
    MLINK = 31, "Too many links";
    PIPE = 32, "Broken pipe";
    RANGE = 34, "Numerical result out of range";
    NAMETOOLONG = 36, "File name too long";
    NOLCK = 37, "No locks available";
    NOSYS = 38, "Function not implemented";
    NOTEMPTY = 39, "Directory not empty";
    LOOP = 40, "Too many levels of symbolic links";
    NODATA = 61, "No data available";
    BADMSG = 74, "Bad message";
    OVERFLOW = 75, "Value too large for defined data type";
    ILSEQ = 84, "Invalid or incomplete multibyte or wide character";
    NOTSOCK = 88, "Socket operation on non-socket";
    OPNOTSUPP = 95, "Operation not supported";
    NETDOWN = 100, "Network is down";
    NETUNREACH = 101, "Network is unreachable";
    CONNABORTED = 103, "Software caused connection abort";
    CONNRESET = 104, "Connection reset by peer";
    NOTCONN = 107, "Transport endpoint is not connected";
    TIMEDOUT = 110, "Connection timed out";
    HOSTDOWN = 112, "Host is down";
    HOSTUNREACH = 113, "No route to host";
    STALE = 116, "Stale file handle";
    UCLEAN = 117, "Structure needs cleaning";
    DQUOT = 122, "Disk quota exceeded";
    CANCELED = 125, "Operation canceled";
}

impl Errno {
    pub const NOTSUP: Self = Self::OPNOTSUPP;
    pub const WOULDBLOCK: Self = Self::AGAIN;

    pub const fn raw_os_error(self) -> i32 {
        self.0
    }

    pub const fn from_raw_os_error(raw: i32) -> Self {
        Self(raw)
    }

    /// The errno an `io::Error` stands for, whether it carries an `Errno`
    /// produced here or a native Windows error code.
    pub fn from_io_error(error: &io::Error) -> Option<Self> {
        if let Some(errno) = error
            .get_ref()
            .and_then(|inner| inner.downcast_ref::<Self>())
        {
            return Some(*errno);
        }
        if let Some(code) = error.raw_os_error() {
            return Some(Self::from_windows_error(code));
        }
        Some(match error.kind() {
            io::ErrorKind::NotFound => Self::NOENT,
            io::ErrorKind::PermissionDenied => Self::ACCESS,
            io::ErrorKind::AlreadyExists => Self::EXIST,
            io::ErrorKind::Interrupted => Self::INTR,
            io::ErrorKind::InvalidInput | io::ErrorKind::InvalidFilename => Self::INVAL,
            io::ErrorKind::Unsupported => Self::OPNOTSUPP,
            io::ErrorKind::OutOfMemory => Self::NOMEM,
            io::ErrorKind::TimedOut => Self::TIMEDOUT,
            io::ErrorKind::WouldBlock => Self::AGAIN,
            io::ErrorKind::DirectoryNotEmpty => Self::NOTEMPTY,
            io::ErrorKind::NotADirectory => Self::NOTDIR,
            io::ErrorKind::IsADirectory => Self::ISDIR,
            io::ErrorKind::ReadOnlyFilesystem => Self::ROFS,
            io::ErrorKind::StorageFull => Self::NOSPC,
            io::ErrorKind::CrossesDevices => Self::XDEV,
            io::ErrorKind::ResourceBusy => Self::BUSY,
            io::ErrorKind::FileTooLarge => Self::FBIG,
            io::ErrorKind::InvalidData => Self::INVAL,
            io::ErrorKind::UnexpectedEof => Self::IO,
            _ => return None,
        })
    }

    pub(crate) fn from_windows_error(code: i32) -> Self {
        match code {
            2 | 3 | 15 | 53 | 67 | 161 => Self::NOENT,
            5 | 225 | 1920 => Self::ACCESS,
            6 => Self::BADF,
            8 | 14 => Self::NOMEM,
            16 | 32 | 33 | 170 => Self::BUSY,
            17 => Self::XDEV,
            19 => Self::ROFS,
            39 | 112 => Self::NOSPC,
            50 | 120 => Self::OPNOTSUPP,
            80 | 183 => Self::EXIST,
            87 | 123 | 4390 | 4393 => Self::INVAL,
            145 => Self::NOTEMPTY,
            206 => Self::NAMETOOLONG,
            267 => Self::NOTDIR,
            995 => Self::INTR,
            1117 => Self::IO,
            1223 => Self::CANCELED,
            1314 => Self::PERM,
            1921 => Self::LOOP,
            _ => Self::IO,
        }
    }

    pub fn kind(self) -> io::ErrorKind {
        match self {
            Self::NOENT => io::ErrorKind::NotFound,
            Self::PERM | Self::ACCESS => io::ErrorKind::PermissionDenied,
            Self::EXIST => io::ErrorKind::AlreadyExists,
            Self::INTR => io::ErrorKind::Interrupted,
            Self::INVAL => io::ErrorKind::InvalidInput,
            Self::OPNOTSUPP | Self::NOSYS => io::ErrorKind::Unsupported,
            Self::NOMEM => io::ErrorKind::OutOfMemory,
            Self::TIMEDOUT => io::ErrorKind::TimedOut,
            Self::AGAIN => io::ErrorKind::WouldBlock,
            Self::NOTEMPTY => io::ErrorKind::DirectoryNotEmpty,
            Self::NOTDIR => io::ErrorKind::NotADirectory,
            Self::ISDIR => io::ErrorKind::IsADirectory,
            Self::ROFS => io::ErrorKind::ReadOnlyFilesystem,
            Self::NOSPC => io::ErrorKind::StorageFull,
            Self::XDEV => io::ErrorKind::CrossesDevices,
            Self::BUSY => io::ErrorKind::ResourceBusy,
            Self::FBIG => io::ErrorKind::FileTooLarge,
            _ => io::ErrorKind::Other,
        }
    }

    pub(crate) fn from_io(error: io::Error) -> Self {
        Self::from_io_error(&error).unwrap_or(Self::IO)
    }
}

impl fmt::Debug for Errno {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Errno({}, {:?})", self.0, self.text())
    }
}

impl fmt::Display for Errno {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} (os error {})", self.text(), self.0)
    }
}

impl std::error::Error for Errno {}

impl From<Errno> for io::Error {
    fn from(errno: Errno) -> Self {
        io::Error::new(errno.kind(), errno)
    }
}

impl From<io::Error> for Errno {
    fn from(error: io::Error) -> Self {
        Self::from_io(error)
    }
}

pub fn dup<Fd: AsFd>(fd: Fd) -> Result<OwnedFd> {
    fd.as_fd().try_clone_to_owned().map_err(Errno::from_io)
}

pub fn read<Fd: AsFd>(fd: Fd, buf: &mut [u8]) -> Result<usize> {
    use std::io::Read;
    let mut file = fd.as_fd().file().map_err(Errno::from_io)?;
    file.read(buf).map_err(Errno::from_io)
}

pub fn write<Fd: AsFd>(fd: Fd, buf: &[u8]) -> Result<usize> {
    use std::io::Write;
    let mut file = fd.as_fd().file().map_err(Errno::from_io)?;
    file.write(buf).map_err(Errno::from_io)
}

pub fn pread<Fd: AsFd>(fd: Fd, buf: &mut [u8], offset: u64) -> Result<usize> {
    use std::os::windows::fs::FileExt;
    let file = fd.as_fd().file().map_err(Errno::from_io)?;
    file.seek_read(buf, offset).map_err(Errno::from_io)
}

pub fn pwrite<Fd: AsFd>(fd: Fd, buf: &[u8], offset: u64) -> Result<usize> {
    use std::os::windows::fs::FileExt;
    let file = fd.as_fd().file().map_err(Errno::from_io)?;
    file.seek_write(buf, offset).map_err(Errno::from_io)
}
