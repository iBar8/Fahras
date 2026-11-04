// SPDX-License-Identifier: MIT

use std::{
    ffi::{CStr, CString},
    fs::{self, ReadDir},
    iter::Peekable,
    mem::MaybeUninit,
};

use super::types::FileType;
use crate::{
    fd::AsFd,
    io::{Errno, Result},
};

pub struct DirEntry {
    name: CString,
    kind: FileType,
}

impl DirEntry {
    pub fn file_name(&self) -> &CStr {
        &self.name
    }

    pub fn file_type(&self) -> FileType {
        self.kind
    }

    /// Windows listings carry no file index; callers that need identity stat the entry.
    pub fn ino(&self) -> u64 {
        0
    }
}

fn kind_of(entry: &fs::DirEntry) -> FileType {
    match entry.file_type() {
        Ok(kind) if kind.is_symlink() => FileType::Symlink,
        Ok(kind) if kind.is_dir() => FileType::Directory,
        Ok(kind) if kind.is_file() => FileType::RegularFile,
        _ => FileType::Unknown,
    }
}

fn entry(entry: std::io::Result<fs::DirEntry>) -> Result<DirEntry> {
    let entry = entry.map_err(Errno::from_io)?;
    let name = CString::new(entry.file_name().into_encoded_bytes()).map_err(|_| Errno::ILSEQ)?;
    Ok(DirEntry {
        kind: kind_of(&entry),
        name,
    })
}

fn open(fd: impl AsFd) -> Result<ReadDir> {
    let path = fd
        .as_fd()
        .location()
        .map_err(Errno::from_io)?
        .unwrap_or_else(|| ".".into());
    fs::read_dir(path).map_err(Errno::from_io)
}

pub struct Dir {
    entries: ReadDir,
}

impl Dir {
    pub fn read_from<Fd: AsFd>(fd: Fd) -> Result<Self> {
        Ok(Self { entries: open(fd)? })
    }

    pub fn new<Fd: AsFd>(fd: Fd) -> Result<Self> {
        Self::read_from(fd)
    }
}

impl Iterator for Dir {
    type Item = Result<DirEntry>;

    fn next(&mut self) -> Option<Self::Item> {
        self.entries.next().map(entry)
    }
}

/// How many entries make up one "buffer" for callers that batch per refill.
const ENTRIES_PER_BUFFER: usize = 512;

pub struct RawDir<'buf> {
    source: Option<Result<Peekable<ReadDir>>>,
    current: Option<DirEntry>,
    served: usize,
    _buffer: &'buf mut [MaybeUninit<u8>],
}

pub struct RawDirEntry<'a> {
    entry: &'a DirEntry,
}

impl RawDirEntry<'_> {
    pub fn file_name(&self) -> &CStr {
        self.entry.file_name()
    }

    pub fn file_type(&self) -> FileType {
        self.entry.file_type()
    }

    pub fn ino(&self) -> u64 {
        self.entry.ino()
    }
}

impl<'buf> RawDir<'buf> {
    pub fn new<Fd: AsFd>(fd: Fd, buf: &'buf mut [MaybeUninit<u8>]) -> Self {
        Self {
            source: Some(open(fd).map(Iterator::peekable)),
            current: None,
            served: 0,
            _buffer: buf,
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Option<Result<RawDirEntry<'_>>> {
        let entries = match self.source.as_mut()? {
            Ok(entries) => entries,
            Err(_) => {
                let Some(Err(error)) = self.source.take() else {
                    return None;
                };
                return Some(Err(error));
            }
        };
        match entry(entries.next()?) {
            Ok(next) => {
                self.served += 1;
                self.current = Some(next);
                self.current.as_ref().map(|entry| Ok(RawDirEntry { entry }))
            }
            Err(error) => Some(Err(error)),
        }
    }

    pub fn is_buffer_empty(&mut self) -> bool {
        let exhausted = match self.source.as_mut() {
            Some(Ok(entries)) => entries.peek().is_none(),
            _ => true,
        };
        exhausted || self.served % ENTRIES_PER_BUFFER == 0
    }
}
