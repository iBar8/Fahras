// SPDX-License-Identifier: MIT

use std::{
    fs::{DirBuilder, File, FileType, Metadata, OpenOptions, Permissions},
    io,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

fn seconds(time: io::Result<SystemTime>) -> (i64, i64) {
    match time.map(|time| time.duration_since(UNIX_EPOCH)) {
        Ok(Ok(since)) => (since.as_secs() as i64, i64::from(since.subsec_nanos())),
        Ok(Err(before)) => (-(before.duration().as_secs() as i64), 0),
        Err(_) => (0, 0),
    }
}

/// Unix metadata derived from what Windows reports: one owner (uid 0) and
/// permission bits from the read-only attribute.
///
/// `std::fs::Metadata` carries no volume or file index on Windows, so `dev`
/// is always 0 and `ino` is the creation time, which changes whenever a file
/// is replaced. That serves change stamps and cache keys; real identity
/// comparisons use `rustix::fs::stat`, which reports the file index.
pub trait MetadataExt {
    fn dev(&self) -> u64;
    fn ino(&self) -> u64;
    fn mode(&self) -> u32;
    fn uid(&self) -> u32;
    fn gid(&self) -> u32;
    fn nlink(&self) -> u64;
    fn size(&self) -> u64;
    fn atime(&self) -> i64;
    fn atime_nsec(&self) -> i64;
    fn mtime(&self) -> i64;
    fn mtime_nsec(&self) -> i64;
    fn ctime(&self) -> i64;
    fn ctime_nsec(&self) -> i64;
    fn blksize(&self) -> u64;
    fn blocks(&self) -> u64;
}

impl MetadataExt for Metadata {
    fn dev(&self) -> u64 {
        0
    }

    fn ino(&self) -> u64 {
        std::os::windows::fs::MetadataExt::creation_time(self)
    }

    fn mode(&self) -> u32 {
        let kind = if self.file_type().is_symlink() {
            0o120000
        } else if self.is_dir() {
            0o40000
        } else {
            0o100000
        };
        kind | self.permissions().mode()
    }

    fn uid(&self) -> u32 {
        0
    }

    fn gid(&self) -> u32 {
        0
    }

    fn nlink(&self) -> u64 {
        1
    }

    fn size(&self) -> u64 {
        self.len()
    }

    fn atime(&self) -> i64 {
        seconds(self.accessed()).0
    }

    fn atime_nsec(&self) -> i64 {
        seconds(self.accessed()).1
    }

    fn mtime(&self) -> i64 {
        seconds(self.modified()).0
    }

    fn mtime_nsec(&self) -> i64 {
        seconds(self.modified()).1
    }

    /// Windows has no inode change time; the last write is the closest match.
    fn ctime(&self) -> i64 {
        self.mtime()
    }

    fn ctime_nsec(&self) -> i64 {
        self.mtime_nsec()
    }

    fn blksize(&self) -> u64 {
        4096
    }

    fn blocks(&self) -> u64 {
        self.len().div_ceil(512)
    }
}

pub trait PermissionsExt {
    fn mode(&self) -> u32;
    fn set_mode(&mut self, mode: u32);
    fn from_mode(mode: u32) -> Self;
}

impl PermissionsExt for Permissions {
    fn mode(&self) -> u32 {
        if self.readonly() { 0o555 } else { 0o755 }
    }

    fn set_mode(&mut self, mode: u32) {
        self.set_readonly(mode & 0o222 == 0);
    }

    /// Windows `Permissions` has no constructor; start from a regular file's
    /// and apply the mode. A directory's would carry its directory attribute,
    /// which Windows rejects on a file with "The parameter is incorrect".
    fn from_mode(mode: u32) -> Self {
        let mut permissions = std::env::current_exe()
            .and_then(std::fs::metadata)
            .expect("the running executable exists")
            .permissions();
        permissions.set_mode(mode);
        permissions
    }
}

pub trait OpenOptionsExt {
    fn mode(&mut self, mode: u32) -> &mut Self;
    fn custom_flags(&mut self, flags: i32) -> &mut Self;
}

impl OpenOptionsExt for OpenOptions {
    fn mode(&mut self, _mode: u32) -> &mut Self {
        self
    }

    fn custom_flags(&mut self, _flags: i32) -> &mut Self {
        self
    }
}

pub trait DirBuilderExt {
    fn mode(&mut self, mode: u32) -> &mut Self;
}

impl DirBuilderExt for DirBuilder {
    fn mode(&mut self, _mode: u32) -> &mut Self {
        self
    }
}

/// Listings carry no file index on Windows; callers fall back to comparing paths.
pub trait DirEntryExt {
    fn ino(&self) -> u64;
}

impl DirEntryExt for std::fs::DirEntry {
    fn ino(&self) -> u64 {
        0
    }
}

pub trait FileTypeExt {
    fn is_block_device(&self) -> bool;
    fn is_char_device(&self) -> bool;
    fn is_fifo(&self) -> bool;
    fn is_socket(&self) -> bool;
}

impl FileTypeExt for FileType {
    fn is_block_device(&self) -> bool {
        false
    }

    fn is_char_device(&self) -> bool {
        false
    }

    fn is_fifo(&self) -> bool {
        false
    }

    fn is_socket(&self) -> bool {
        false
    }
}

pub trait FileExt {
    fn read_at(&self, buf: &mut [u8], offset: u64) -> io::Result<usize>;
    fn write_at(&self, buf: &[u8], offset: u64) -> io::Result<usize>;

    fn read_exact_at(&self, mut buf: &mut [u8], mut offset: u64) -> io::Result<()> {
        while !buf.is_empty() {
            match self.read_at(buf, offset) {
                Ok(0) => return Err(io::ErrorKind::UnexpectedEof.into()),
                Ok(read) => {
                    buf = &mut buf[read..];
                    offset += read as u64;
                }
                Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
                Err(error) => return Err(error),
            }
        }
        Ok(())
    }
}

impl FileExt for File {
    fn read_at(&self, buf: &mut [u8], offset: u64) -> io::Result<usize> {
        std::os::windows::fs::FileExt::seek_read(self, buf, offset)
    }

    fn write_at(&self, buf: &[u8], offset: u64) -> io::Result<usize> {
        std::os::windows::fs::FileExt::seek_write(self, buf, offset)
    }
}

pub fn symlink<P: AsRef<Path>, Q: AsRef<Path>>(original: P, link: Q) -> io::Result<()> {
    let (original, link) = (original.as_ref(), link.as_ref());
    let target = link
        .parent()
        .map_or_else(|| original.to_path_buf(), |parent| parent.join(original));
    if target.is_dir() {
        std::os::windows::fs::symlink_dir(original, link)
    } else {
        std::os::windows::fs::symlink_file(original, link)
    }
}
