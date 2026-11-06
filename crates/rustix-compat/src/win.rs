// SPDX-License-Identifier: MIT

use std::{
    ffi::OsString,
    fs::{File, OpenOptions},
    io,
    mem::{MaybeUninit, size_of},
    os::windows::{
        ffi::{OsStrExt, OsStringExt},
        fs::OpenOptionsExt,
        io::AsRawHandle,
    },
    path::{Path, PathBuf},
};

use windows_sys::Win32::Storage::FileSystem::{
    BY_HANDLE_FILE_INFORMATION, FILE_ATTRIBUTE_DIRECTORY, FILE_ATTRIBUTE_READONLY, FILE_BASIC_INFO,
    FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT, FILE_NAME_NORMALIZED, FileBasicInfo,
    GetDiskFreeSpaceExW, GetFileInformationByHandle, GetFileInformationByHandleEx,
    GetFinalPathNameByHandleW, GetVolumeInformationByHandleW,
};

const FILE_READ_ATTRIBUTES: u32 = 0x80;

pub(crate) fn final_path(file: &File) -> io::Result<PathBuf> {
    let mut buffer = vec![0u16; 512];
    loop {
        // SAFETY: the handle is open for the borrow and the buffer length is passed.
        let length = unsafe {
            GetFinalPathNameByHandleW(
                file.as_raw_handle(),
                buffer.as_mut_ptr(),
                buffer.len() as u32,
                FILE_NAME_NORMALIZED,
            )
        } as usize;
        if length == 0 {
            return Err(io::Error::last_os_error());
        }
        if length < buffer.len() {
            buffer.truncate(length);
            return Ok(PathBuf::from(OsString::from_wide(&buffer)));
        }
        buffer.resize(length + 1, 0);
    }
}

/// Opens only for attribute queries; works on directories and, with
/// `follow` off, on the link itself rather than its target.
pub(crate) fn open_for_attributes(path: &Path, follow: bool) -> io::Result<File> {
    let mut flags = FILE_FLAG_BACKUP_SEMANTICS;
    if !follow {
        flags |= FILE_FLAG_OPEN_REPARSE_POINT;
    }
    OpenOptions::new()
        .access_mode(FILE_READ_ATTRIBUTES)
        .share_mode(7)
        .custom_flags(flags)
        .open(path)
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Info {
    pub attributes: u32,
    pub volume: u32,
    pub index: u64,
    pub links: u32,
    pub size: u64,
    pub created: i64,
    pub accessed: i64,
    pub written: i64,
    pub changed: i64,
}

impl Info {
    pub fn is_dir(&self) -> bool {
        self.attributes & FILE_ATTRIBUTE_DIRECTORY != 0
    }

    pub fn is_readonly(&self) -> bool {
        self.attributes & FILE_ATTRIBUTE_READONLY != 0
    }
}

pub(crate) fn info(file: &File) -> io::Result<Info> {
    let mut by_handle = MaybeUninit::<BY_HANDLE_FILE_INFORMATION>::zeroed();
    // SAFETY: the handle is open and the out pointer is valid for the struct.
    if unsafe { GetFileInformationByHandle(file.as_raw_handle(), by_handle.as_mut_ptr()) } == 0 {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: the call above succeeded and filled the struct.
    let by_handle = unsafe { by_handle.assume_init() };
    let mut basic = MaybeUninit::<FILE_BASIC_INFO>::zeroed();
    // SAFETY: as above, with the matching size for `FileBasicInfo`.
    let has_basic = unsafe {
        GetFileInformationByHandleEx(
            file.as_raw_handle(),
            FileBasicInfo,
            basic.as_mut_ptr().cast(),
            size_of::<FILE_BASIC_INFO>() as u32,
        )
    } != 0;
    let filetime = |time: windows_sys::Win32::Foundation::FILETIME| {
        (u64::from(time.dwHighDateTime) << 32 | u64::from(time.dwLowDateTime)) as i64
    };
    let written = filetime(by_handle.ftLastWriteTime);
    let changed = if has_basic {
        // SAFETY: filled by the successful call.
        unsafe { basic.assume_init() }.ChangeTime
    } else {
        written
    };
    Ok(Info {
        attributes: by_handle.dwFileAttributes,
        volume: by_handle.dwVolumeSerialNumber,
        index: u64::from(by_handle.nFileIndexHigh) << 32 | u64::from(by_handle.nFileIndexLow),
        links: by_handle.nNumberOfLinks,
        size: u64::from(by_handle.nFileSizeHigh) << 32 | u64::from(by_handle.nFileSizeLow),
        created: filetime(by_handle.ftCreationTime),
        accessed: filetime(by_handle.ftLastAccessTime),
        written,
        changed,
    })
}

/// Converts a FILETIME (100 ns ticks since 1601) to Unix seconds and nanoseconds.
pub(crate) fn unix_time(filetime: i64) -> (i64, u32) {
    const EPOCH_DIFFERENCE: i64 = 116_444_736_000_000_000;
    let ticks = filetime - EPOCH_DIFFERENCE;
    (
        ticks.div_euclid(10_000_000),
        (ticks.rem_euclid(10_000_000) * 100) as u32,
    )
}

pub(crate) struct Space {
    pub available: u64,
    pub total: u64,
}

pub(crate) fn disk_space(path: &Path) -> io::Result<Space> {
    let wide: Vec<u16> = path.as_os_str().encode_wide().chain([0]).collect();
    let (mut available, mut total, mut free) = (0u64, 0u64, 0u64);
    // SAFETY: the path is NUL-terminated and the out pointers are valid.
    if unsafe { GetDiskFreeSpaceExW(wide.as_ptr(), &mut available, &mut total, &mut free) } == 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(Space { available, total })
}

pub(crate) struct Volume {
    pub filesystem: String,
    pub name_max: u32,
}

pub(crate) fn volume(file: &File) -> io::Result<Volume> {
    let mut name_max = 0u32;
    let mut filesystem = [0u16; 64];
    // SAFETY: the handle is open; unused outputs are null, used ones are sized.
    let ok = unsafe {
        GetVolumeInformationByHandleW(
            file.as_raw_handle(),
            std::ptr::null_mut(),
            0,
            std::ptr::null_mut(),
            &mut name_max,
            std::ptr::null_mut(),
            filesystem.as_mut_ptr(),
            filesystem.len() as u32,
        )
    };
    if ok == 0 {
        return Err(io::Error::last_os_error());
    }
    let end = filesystem
        .iter()
        .position(|&c| c == 0)
        .unwrap_or(filesystem.len());
    Ok(Volume {
        filesystem: String::from_utf16_lossy(&filesystem[..end]),
        name_max,
    })
}
