// SPDX-License-Identifier: MIT

//! Flag and metadata types with Linux values, so code that stores or
//! compares raw bits behaves the same on both platforms.

use bitflags::bitflags;

pub type RawMode = u32;
pub type FsWord = i64;
pub type Dev = u64;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
    pub struct OFlags: u32 {
        const RDONLY = 0;
        const WRONLY = 0o1;
        const RDWR = 0o2;
        const ACCMODE = 0o3;
        const CREATE = 0o100;
        const EXCL = 0o200;
        const NOCTTY = 0o400;
        const TRUNC = 0o1000;
        const APPEND = 0o2000;
        const NONBLOCK = 0o4000;
        const DSYNC = 0o10000;
        const DIRECT = 0o40000;
        const LARGEFILE = 0o100000;
        const DIRECTORY = 0o200000;
        const NOFOLLOW = 0o400000;
        const NOATIME = 0o1000000;
        const CLOEXEC = 0o2000000;
        const SYNC = 0o4010000;
        const RSYNC = 0o4010000;
        const PATH = 0o10000000;
        const TMPFILE = 0o20200000;
        const _ = !0;
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
    pub struct Mode: u32 {
        const RWXU = 0o700;
        const RUSR = 0o400;
        const WUSR = 0o200;
        const XUSR = 0o100;
        const RWXG = 0o70;
        const RGRP = 0o40;
        const WGRP = 0o20;
        const XGRP = 0o10;
        const RWXO = 0o7;
        const ROTH = 0o4;
        const WOTH = 0o2;
        const XOTH = 0o1;
        const SUID = 0o4000;
        const SGID = 0o2000;
        const SVTX = 0o1000;
        const _ = !0;
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
    pub struct AtFlags: u32 {
        const SYMLINK_NOFOLLOW = 0x100;
        const EACCESS = 0x200;
        const REMOVEDIR = 0x200;
        const SYMLINK_FOLLOW = 0x400;
        const NO_AUTOMOUNT = 0x800;
        const EMPTY_PATH = 0x1000;
        const STATX_SYNC_AS_STAT = 0;
        const STATX_FORCE_SYNC = 0x2000;
        const STATX_DONT_SYNC = 0x4000;
        const _ = !0;
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
    pub struct ResolveFlags: u64 {
        const NO_XDEV = 0x1;
        const NO_MAGICLINKS = 0x2;
        const NO_SYMLINKS = 0x4;
        const BENEATH = 0x8;
        const IN_ROOT = 0x10;
        const CACHED = 0x20;
        const _ = !0;
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
    pub struct RenameFlags: u32 {
        const NOREPLACE = 1;
        const EXCHANGE = 2;
        const WHITEOUT = 4;
        const _ = !0;
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
    pub struct StatxFlags: u32 {
        const TYPE = 0x1;
        const MODE = 0x2;
        const NLINK = 0x4;
        const UID = 0x8;
        const GID = 0x10;
        const ATIME = 0x20;
        const MTIME = 0x40;
        const CTIME = 0x80;
        const INO = 0x100;
        const SIZE = 0x200;
        const BLOCKS = 0x400;
        const BASIC_STATS = 0x7ff;
        const BTIME = 0x800;
        const MNT_ID = 0x1000;
        const DIOALIGN = 0x2000;
        const ALL = 0xfff;
        const _ = !0;
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
    pub struct StatxAttributes: u64 {
        const _ = !0;
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
    pub struct XattrFlags: u32 {
        const CREATE = 1;
        const REPLACE = 2;
        const _ = !0;
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
    pub struct Access: u32 {
        const EXISTS = 0;
        const EXEC_OK = 1;
        const WRITE_OK = 2;
        const READ_OK = 4;
        const _ = !0;
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
    pub struct MemfdFlags: u32 {
        const CLOEXEC = 1;
        const ALLOW_SEALING = 2;
        const _ = !0;
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
    pub struct IFlags: u32 {
        const APPEND = 0x20;
        const IMMUTABLE = 0x10;
        const _ = !0;
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
    pub struct StatVfsMountFlags: u64 {
        const RDONLY = 1;
        const _ = !0;
    }
}

impl Mode {
    pub const fn from_raw_mode(mode: RawMode) -> Self {
        Self::from_bits_retain(mode & 0o7777)
    }

    pub const fn as_raw_mode(self) -> RawMode {
        self.bits()
    }
}

impl From<RawMode> for Mode {
    fn from(mode: RawMode) -> Self {
        Self::from_raw_mode(mode)
    }
}

impl From<Mode> for RawMode {
    fn from(mode: Mode) -> Self {
        mode.as_raw_mode()
    }
}

pub(crate) const S_IFMT: u32 = 0o170000;
pub(crate) const S_IFSOCK: u32 = 0o140000;
pub(crate) const S_IFLNK: u32 = 0o120000;
pub(crate) const S_IFREG: u32 = 0o100000;
pub(crate) const S_IFBLK: u32 = 0o60000;
pub(crate) const S_IFDIR: u32 = 0o40000;
pub(crate) const S_IFCHR: u32 = 0o20000;
pub(crate) const S_IFIFO: u32 = 0o10000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FileType {
    RegularFile,
    Directory,
    Symlink,
    Fifo,
    Socket,
    CharacterDevice,
    BlockDevice,
    Unknown,
}

impl FileType {
    pub const fn from_raw_mode(mode: RawMode) -> Self {
        match mode & S_IFMT {
            S_IFREG => Self::RegularFile,
            S_IFDIR => Self::Directory,
            S_IFLNK => Self::Symlink,
            S_IFIFO => Self::Fifo,
            S_IFSOCK => Self::Socket,
            S_IFCHR => Self::CharacterDevice,
            S_IFBLK => Self::BlockDevice,
            _ => Self::Unknown,
        }
    }

    pub const fn as_raw_mode(self) -> RawMode {
        match self {
            Self::RegularFile => S_IFREG,
            Self::Directory => S_IFDIR,
            Self::Symlink => S_IFLNK,
            Self::Fifo => S_IFIFO,
            Self::Socket => S_IFSOCK,
            Self::CharacterDevice => S_IFCHR,
            Self::BlockDevice => S_IFBLK,
            Self::Unknown => S_IFMT,
        }
    }

    pub fn is_dir(self) -> bool {
        self == Self::Directory
    }

    pub fn is_file(self) -> bool {
        self == Self::RegularFile
    }

    pub fn is_symlink(self) -> bool {
        self == Self::Symlink
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct Timespec {
    pub tv_sec: i64,
    pub tv_nsec: i64,
}

pub const UTIME_NOW: i64 = (1 << 30) - 1;
pub const UTIME_OMIT: i64 = (1 << 30) - 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Timestamps {
    pub last_access: Timespec,
    pub last_modification: Timespec,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeekFrom {
    Start(u64),
    End(i64),
    Current(i64),
    Data(u64),
    Hole(u64),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlockOperation {
    LockShared,
    LockExclusive,
    Unlock,
    NonBlockingLockShared,
    NonBlockingLockExclusive,
    NonBlockingUnlock,
}

/// The fields of Linux `struct stat` that Fahras reads.
#[derive(Debug, Clone, Copy, Default)]
pub struct Stat {
    pub st_dev: u64,
    pub st_ino: u64,
    pub st_nlink: u64,
    pub st_mode: u32,
    pub st_uid: u32,
    pub st_gid: u32,
    pub st_rdev: u64,
    pub st_size: i64,
    pub st_blksize: i64,
    pub st_blocks: i64,
    pub st_atime: i64,
    pub st_atime_nsec: i64,
    pub st_mtime: i64,
    pub st_mtime_nsec: i64,
    pub st_ctime: i64,
    pub st_ctime_nsec: i64,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct StatxTimestamp {
    pub tv_sec: i64,
    pub tv_nsec: u32,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Statx {
    pub stx_mask: u32,
    pub stx_blksize: u32,
    pub stx_attributes: u64,
    pub stx_nlink: u32,
    pub stx_uid: u32,
    pub stx_gid: u32,
    pub stx_mode: u16,
    pub stx_ino: u64,
    pub stx_size: u64,
    pub stx_blocks: u64,
    pub stx_attributes_mask: u64,
    pub stx_atime: StatxTimestamp,
    pub stx_btime: StatxTimestamp,
    pub stx_ctime: StatxTimestamp,
    pub stx_mtime: StatxTimestamp,
    pub stx_rdev_major: u32,
    pub stx_rdev_minor: u32,
    pub stx_dev_major: u32,
    pub stx_dev_minor: u32,
    pub stx_mnt_id: u64,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct StatFs {
    pub f_type: FsWord,
    pub f_bsize: FsWord,
    pub f_blocks: u64,
    pub f_bfree: u64,
    pub f_bavail: u64,
    pub f_files: u64,
    pub f_ffree: u64,
    pub f_namelen: FsWord,
    pub f_frsize: FsWord,
    pub f_flags: FsWord,
}

#[derive(Debug, Clone, Copy)]
pub struct StatVfs {
    pub f_bsize: u64,
    pub f_frsize: u64,
    pub f_blocks: u64,
    pub f_bfree: u64,
    pub f_bavail: u64,
    pub f_files: u64,
    pub f_ffree: u64,
    pub f_favail: u64,
    pub f_fsid: u64,
    pub f_flag: StatVfsMountFlags,
    pub f_namemax: u64,
}

pub const fn makedev(major: u32, minor: u32) -> Dev {
    ((major as u64 & 0xffff_f000) << 32)
        | ((major as u64 & 0xfff) << 8)
        | ((minor as u64 & 0xffff_ff00) << 12)
        | (minor as u64 & 0xff)
}

pub const fn major(dev: Dev) -> u32 {
    (((dev >> 32) & 0xffff_f000) | ((dev >> 8) & 0xfff)) as u32
}

pub const fn minor(dev: Dev) -> u32 {
    (((dev >> 12) & 0xffff_ff00) | (dev & 0xff)) as u32
}
