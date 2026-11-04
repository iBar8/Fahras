// SPDX-License-Identifier: MIT

use std::{
    ffi::{CString, OsStr},
    fs::{self, File, FileTimes, OpenOptions},
    io::{Seek, SeekFrom as StdSeekFrom},
    os::windows::{
        ffi::OsStrExt,
        fs::OpenOptionsExt,
    },
    path::{Component, Path, PathBuf},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use windows_sys::Win32::Storage::FileSystem::{
    FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_DELETE_ON_CLOSE, FILE_FLAG_OPEN_REPARSE_POINT,
    MoveFileExW,
};

use super::types::*;
use crate::{
    buffer::Buffer,
    fd::{AsFd, BorrowedFd, OwnedFd},
    io::{Errno, Result},
    path::Arg,
    process::{Gid, Uid},
    win,
};

pub const CWD: BorrowedFd<'static> = crate::fd::CWD;

const FILE_READ_ATTRIBUTES: u32 = 0x80;
const FILE_WRITE_ATTRIBUTES: u32 = 0x100;

fn errno(error: std::io::Error) -> Errno {
    Errno::from_io(error)
}

/// Resolves `path` against the directory handle, as the `*at` calls do.
pub(crate) fn resolve(dir: BorrowedFd<'_>, path: impl Arg, flags: AtFlags) -> Result<PathBuf> {
    let path = path.to_path().map_err(errno)?;
    if path.as_os_str().is_empty() {
        if !flags.contains(AtFlags::EMPTY_PATH) {
            return Err(Errno::NOENT);
        }
        return dir.location().map_err(errno)?.ok_or(Errno::NOENT);
    }
    if path.is_absolute() {
        return Ok(path);
    }
    Ok(match dir.location().map_err(errno)? {
        Some(base) => base.join(path),
        None => path,
    })
}

fn is_symlink(path: &Path) -> bool {
    fs::symlink_metadata(path).is_ok_and(|metadata| metadata.file_type().is_symlink())
}

fn mode_of(file: &File, info: &win::Info) -> Result<u32> {
    let kind = file.metadata().map_err(errno)?.file_type();
    Ok(if kind.is_symlink() {
        S_IFLNK | 0o777
    } else if info.is_dir() {
        S_IFDIR | 0o755
    } else if info.is_readonly() {
        S_IFREG | 0o444
    } else {
        S_IFREG | 0o644
    })
}

fn stat_file(file: &File) -> Result<Stat> {
    let info = win::info(file).map_err(errno)?;
    let (atime, atime_nsec) = win::unix_time(info.accessed);
    let (mtime, mtime_nsec) = win::unix_time(info.written);
    let (ctime, ctime_nsec) = win::unix_time(info.changed);
    Ok(Stat {
        st_dev: u64::from(info.volume),
        st_ino: info.index,
        st_nlink: u64::from(info.links),
        st_mode: mode_of(file, &info)?,
        st_size: info.size as i64,
        st_blksize: 4096,
        st_blocks: info.size.div_ceil(512) as i64,
        st_atime: atime,
        st_atime_nsec: i64::from(atime_nsec),
        st_mtime: mtime,
        st_mtime_nsec: i64::from(mtime_nsec),
        st_ctime: ctime,
        st_ctime_nsec: i64::from(ctime_nsec),
        ..Stat::default()
    })
}

fn stat_path(path: &Path, flags: AtFlags) -> Result<Stat> {
    let follow = !flags.contains(AtFlags::SYMLINK_NOFOLLOW) || !is_symlink(path);
    let file = win::open_for_attributes(path, follow).map_err(errno)?;
    stat_file(&file)
}

pub fn statat<P: Arg, Fd: AsFd>(dirfd: Fd, path: P, flags: AtFlags) -> Result<Stat> {
    let dir = dirfd.as_fd();
    if path_is_empty_with(&flags) {
        if let Ok(file) = dir.file() {
            return stat_file(file);
        }
    }
    stat_path(&resolve(dir, path, flags)?, flags)
}

fn path_is_empty_with(flags: &AtFlags) -> bool {
    flags.contains(AtFlags::EMPTY_PATH)
}

pub fn stat<P: Arg>(path: P) -> Result<Stat> {
    statat(CWD, path, AtFlags::empty())
}

pub fn lstat<P: Arg>(path: P) -> Result<Stat> {
    statat(CWD, path, AtFlags::SYMLINK_NOFOLLOW)
}

pub fn fstat<Fd: AsFd>(fd: Fd) -> Result<Stat> {
    let fd = fd.as_fd();
    match fd.file() {
        Ok(file) => stat_file(file),
        Err(_) => stat_path(Path::new("."), AtFlags::empty()),
    }
}

fn statx_timestamp(filetime: i64) -> StatxTimestamp {
    let (tv_sec, tv_nsec) = win::unix_time(filetime);
    StatxTimestamp { tv_sec, tv_nsec }
}

fn statx_from(file: &File, mask: StatxFlags) -> Result<Statx> {
    let info = win::info(file).map_err(errno)?;
    let supported = StatxFlags::BASIC_STATS | StatxFlags::BTIME;
    Ok(Statx {
        stx_mask: (mask & supported).bits() | StatxFlags::TYPE.bits() | StatxFlags::MODE.bits(),
        stx_blksize: 4096,
        stx_nlink: info.links,
        stx_mode: mode_of(file, &info)? as u16,
        stx_ino: info.index,
        stx_size: info.size,
        stx_blocks: info.size.div_ceil(512),
        stx_atime: statx_timestamp(info.accessed),
        stx_btime: statx_timestamp(info.created),
        stx_ctime: statx_timestamp(info.changed),
        stx_mtime: statx_timestamp(info.written),
        stx_dev_major: major(u64::from(info.volume)),
        stx_dev_minor: minor(u64::from(info.volume)),
        ..Statx::default()
    })
}

pub fn statx<P: Arg, Fd: AsFd>(
    dirfd: Fd,
    path: P,
    flags: AtFlags,
    mask: StatxFlags,
) -> Result<Statx> {
    let dir = dirfd.as_fd();
    if path_is_empty_with(&flags) {
        if let Ok(file) = dir.file() {
            return statx_from(file, mask);
        }
    }
    let path = resolve(dir, path, flags)?;
    let follow = !flags.contains(AtFlags::SYMLINK_NOFOLLOW) || !is_symlink(&path);
    let file = win::open_for_attributes(&path, follow).map_err(errno)?;
    statx_from(&file, mask)
}

struct OpenPlan {
    path: PathBuf,
    flags: OFlags,
}

fn open_plan(path: PathBuf, flags: OFlags) -> Result<OpenPlan> {
    if flags.contains(OFlags::TMPFILE) {
        return Err(Errno::OPNOTSUPP);
    }
    let link = fs::symlink_metadata(&path).ok();
    let is_link = link.as_ref().is_some_and(|m| m.file_type().is_symlink());
    if is_link && flags.contains(OFlags::NOFOLLOW) && !flags.contains(OFlags::PATH) {
        return Err(Errno::LOOP);
    }
    if flags.contains(OFlags::CREATE | OFlags::EXCL) && link.is_some() {
        return Err(Errno::EXIST);
    }
    let target_is_dir = fs::metadata(&path).is_ok_and(|m| m.is_dir());
    if flags.contains(OFlags::DIRECTORY) && link.is_some() && !target_is_dir && !is_link {
        return Err(Errno::NOTDIR);
    }
    let writes = flags.intersects(OFlags::WRONLY | OFlags::RDWR);
    if target_is_dir && writes {
        return Err(Errno::ISDIR);
    }
    Ok(OpenPlan { path, flags })
}

fn open_options(plan: &OpenPlan) -> OpenOptions {
    let flags = plan.flags;
    let mut options = OpenOptions::new();
    let mut custom = FILE_FLAG_BACKUP_SEMANTICS;
    if flags.contains(OFlags::NOFOLLOW) && flags.contains(OFlags::PATH) {
        custom |= FILE_FLAG_OPEN_REPARSE_POINT;
    }
    options.custom_flags(custom).share_mode(7);
    if flags.contains(OFlags::PATH) {
        options.access_mode(FILE_READ_ATTRIBUTES);
        return options;
    }
    let access = flags & OFlags::ACCMODE;
    options.read(access != OFlags::WRONLY);
    options.write(access == OFlags::WRONLY || access == OFlags::RDWR);
    options.append(flags.contains(OFlags::APPEND));
    options.truncate(flags.contains(OFlags::TRUNC));
    if flags.contains(OFlags::CREATE) {
        if flags.contains(OFlags::EXCL) {
            options.create_new(true);
        } else {
            options.create(true);
        }
    }
    options
}

fn open_path(path: PathBuf, flags: OFlags, mode: Mode) -> Result<OwnedFd> {
    let plan = open_plan(path, flags)?;
    let file = open_options(&plan).open(&plan.path).map_err(errno)?;
    if flags.contains(OFlags::CREATE) && !mode.intersects(Mode::WUSR | Mode::WGRP | Mode::WOTH) {
        let _ = fs::metadata(&plan.path).map(|metadata| {
            let mut permissions = metadata.permissions();
            permissions.set_readonly(true);
            fs::set_permissions(&plan.path, permissions)
        });
    }
    Ok(OwnedFd::new(file, Some(plan.path), flags.bits()))
}

pub fn open<P: Arg>(path: P, flags: OFlags, mode: Mode) -> Result<OwnedFd> {
    open_path(resolve(CWD, path, AtFlags::empty())?, flags, mode)
}

pub fn openat<P: Arg, Fd: AsFd>(dirfd: Fd, path: P, oflags: OFlags, create_mode: Mode) -> Result<OwnedFd> {
    open_path(resolve(dirfd.as_fd(), path, AtFlags::empty())?, oflags, create_mode)
}

/// `RESOLVE_BENEATH`/`IN_ROOT` refuse absolute paths and `..`; `NO_SYMLINKS`
/// refuses any symlink along the way, as Linux does.
pub fn openat2<Fd: AsFd, P: Arg>(
    dirfd: Fd,
    path: P,
    oflags: OFlags,
    mode: Mode,
    resolve_flags: ResolveFlags,
) -> Result<OwnedFd> {
    let dir = dirfd.as_fd();
    let relative = path.to_path().map_err(errno)?;
    if resolve_flags.intersects(ResolveFlags::BENEATH | ResolveFlags::IN_ROOT)
        && relative
            .components()
            .any(|part| !matches!(part, Component::Normal(_) | Component::CurDir))
    {
        return Err(Errno::XDEV);
    }
    let full = resolve(dir, &relative, AtFlags::empty())?;
    if resolve_flags.contains(ResolveFlags::NO_SYMLINKS) {
        let mut walked = dir.location().map_err(errno)?.unwrap_or_default();
        for part in relative.components() {
            walked.push(part);
            if is_symlink(&walked) {
                return Err(Errno::LOOP);
            }
        }
    }
    open_path(full, oflags, mode)
}

pub fn mkdirat<P: Arg, Fd: AsFd>(dirfd: Fd, path: P, _mode: Mode) -> Result<()> {
    fs::create_dir(resolve(dirfd.as_fd(), path, AtFlags::empty())?).map_err(errno)
}

fn clear_readonly(path: &Path) -> bool {
    fs::symlink_metadata(path).is_ok_and(|metadata| {
        let mut permissions = metadata.permissions();
        permissions.readonly()
            && {
                permissions.set_readonly(false);
                fs::set_permissions(path, permissions).is_ok()
            }
    })
}

/// Linux lets the directory owner unlink read-only files; Windows refuses
/// until the read-only attribute is cleared.
fn retry_writable(path: &Path, remove: impl Fn(&Path) -> std::io::Result<()>) -> Result<()> {
    match remove(path) {
        Err(error) if Errno::from_io_error(&error) == Some(Errno::ACCESS) && clear_readonly(path) => {
            remove(path).map_err(errno)
        }
        result => result.map_err(errno),
    }
}

pub fn unlinkat<P: Arg, Fd: AsFd>(dirfd: Fd, path: P, flags: AtFlags) -> Result<()> {
    let path = resolve(dirfd.as_fd(), path, flags)?;
    if flags.contains(AtFlags::REMOVEDIR) {
        let metadata = fs::symlink_metadata(&path).map_err(errno)?;
        if !metadata.is_dir() {
            return Err(Errno::NOTDIR);
        }
        return retry_writable(&path, |path| fs::remove_dir(path));
    }
    let metadata = fs::symlink_metadata(&path).map_err(errno)?;
    if metadata.is_dir() {
        return Err(Errno::ISDIR);
    }
    let is_directory_link = metadata.file_type().is_symlink() && fs::metadata(&path).is_ok_and(|m| m.is_dir());
    if is_directory_link {
        return retry_writable(&path, |path| fs::remove_dir(path));
    }
    retry_writable(&path, |path| fs::remove_file(path))
}

fn wide(path: &Path) -> Vec<u16> {
    path.as_os_str().encode_wide().chain([0]).collect()
}

pub fn renameat_with<P: Arg, Q: Arg, PFd: AsFd, QFd: AsFd>(
    old_dirfd: PFd,
    old_path: P,
    new_dirfd: QFd,
    new_path: Q,
    flags: RenameFlags,
) -> Result<()> {
    if flags.contains(RenameFlags::EXCHANGE) || flags.contains(RenameFlags::WHITEOUT) {
        return Err(Errno::INVAL);
    }
    let from = resolve(old_dirfd.as_fd(), old_path, AtFlags::empty())?;
    let to = resolve(new_dirfd.as_fd(), new_path, AtFlags::empty())?;
    fs::symlink_metadata(&from).map_err(errno)?;
    if flags.contains(RenameFlags::NOREPLACE) {
        // Without MOVEFILE_REPLACE_EXISTING the move itself refuses an existing target.
        // SAFETY: both paths are NUL-terminated wide strings that outlive the call.
        if unsafe { MoveFileExW(wide(&from).as_ptr(), wide(&to).as_ptr(), 0) } == 0 {
            return Err(errno(std::io::Error::last_os_error()));
        }
        return Ok(());
    }
    fs::rename(&from, &to).map_err(errno)
}

pub fn renameat<P: Arg, Q: Arg, PFd: AsFd, QFd: AsFd>(
    old_dirfd: PFd,
    old_path: P,
    new_dirfd: QFd,
    new_path: Q,
) -> Result<()> {
    renameat_with(old_dirfd, old_path, new_dirfd, new_path, RenameFlags::empty())
}

pub fn linkat<P: Arg, Q: Arg, PFd: AsFd, QFd: AsFd>(
    old_dirfd: PFd,
    old_path: P,
    new_dirfd: QFd,
    new_path: Q,
    flags: AtFlags,
) -> Result<()> {
    let from = resolve(old_dirfd.as_fd(), old_path, flags)?;
    let to = resolve(new_dirfd.as_fd(), new_path, AtFlags::empty())?;
    fs::hard_link(from, to).map_err(errno)
}

pub fn symlinkat<P: Arg, Q: Arg, Fd: AsFd>(old_path: P, new_dirfd: Fd, new_path: Q) -> Result<()> {
    let target = old_path.to_path().map_err(errno)?;
    let link = resolve(new_dirfd.as_fd(), new_path, AtFlags::empty())?;
    let target_is_dir = link
        .parent()
        .map(|parent| parent.join(&target))
        .is_some_and(|resolved| resolved.is_dir());
    let made = if target_is_dir {
        std::os::windows::fs::symlink_dir(&target, &link)
    } else {
        std::os::windows::fs::symlink_file(&target, &link)
    };
    made.map_err(errno)
}

pub fn readlinkat<P: Arg, Fd: AsFd, B: Into<Vec<u8>>>(dirfd: Fd, path: P, _reuse: B) -> Result<CString> {
    let target = fs::read_link(resolve(dirfd.as_fd(), path, AtFlags::empty())?).map_err(errno)?;
    CString::new(target.into_os_string().into_encoded_bytes()).map_err(|_| Errno::ILSEQ)
}

fn system_time(time: Timespec) -> Option<SystemTime> {
    match time.tv_nsec {
        UTIME_OMIT => None,
        UTIME_NOW => Some(SystemTime::now()),
        nanos if time.tv_sec >= 0 => {
            Some(UNIX_EPOCH + Duration::new(time.tv_sec as u64, nanos as u32))
        }
        nanos => UNIX_EPOCH.checked_sub(Duration::from_secs(time.tv_sec.unsigned_abs()))
            .map(|base| base + Duration::from_nanos(nanos as u64)),
    }
}

fn set_times(path: &Path, times: &Timestamps) -> Result<()> {
    let mut file_times = FileTimes::new();
    if let Some(accessed) = system_time(times.last_access) {
        file_times = file_times.set_accessed(accessed);
    }
    if let Some(modified) = system_time(times.last_modification) {
        file_times = file_times.set_modified(modified);
    }
    let file = OpenOptions::new()
        .access_mode(FILE_WRITE_ATTRIBUTES)
        .share_mode(7)
        .custom_flags(FILE_FLAG_BACKUP_SEMANTICS)
        .open(path)
        .map_err(errno)?;
    file.set_times(file_times).map_err(errno)
}

pub fn utimensat<P: Arg, Fd: AsFd>(dirfd: Fd, path: P, times: &Timestamps, flags: AtFlags) -> Result<()> {
    set_times(&resolve(dirfd.as_fd(), path, flags)?, times)
}

fn location(fd: BorrowedFd<'_>) -> Result<PathBuf> {
    fd.location().map_err(errno)?.ok_or(Errno::BADF)
}

pub fn futimens<Fd: AsFd>(fd: Fd, times: &Timestamps) -> Result<()> {
    set_times(&location(fd.as_fd())?, times)
}

pub fn fchmod<Fd: AsFd>(fd: Fd, mode: Mode) -> Result<()> {
    let path = location(fd.as_fd())?;
    let mut permissions = fs::metadata(&path).map_err(errno)?.permissions();
    permissions.set_readonly(!mode.intersects(Mode::WUSR | Mode::WGRP | Mode::WOTH));
    fs::set_permissions(path, permissions).map_err(errno)
}

pub fn chmod<P: Arg>(path: P, mode: Mode) -> Result<()> {
    fchmod(open(path, OFlags::PATH, Mode::empty())?, mode)
}

pub fn fchown<Fd: AsFd>(_fd: Fd, _owner: Option<Uid>, _group: Option<Gid>) -> Result<()> {
    Ok(())
}

/// Windows refuses to flush directory handles; their entries are durable once
/// the operation that changed them returns.
fn flush(fd: BorrowedFd<'_>, sync: impl Fn(&File) -> std::io::Result<()>) -> Result<()> {
    let file = fd.file().map_err(errno)?;
    if file.metadata().is_ok_and(|metadata| metadata.is_dir()) {
        return Ok(());
    }
    sync(file).map_err(errno)
}

pub fn fsync<Fd: AsFd>(fd: Fd) -> Result<()> {
    flush(fd.as_fd(), File::sync_all)
}

pub fn fdatasync<Fd: AsFd>(fd: Fd) -> Result<()> {
    flush(fd.as_fd(), File::sync_data)
}

pub fn syncfs<Fd: AsFd>(_fd: Fd) -> Result<()> {
    Ok(())
}

pub fn ftruncate<Fd: AsFd>(fd: Fd, length: u64) -> Result<()> {
    fd.as_fd().file().map_err(errno)?.set_len(length).map_err(errno)
}

pub fn seek<Fd: AsFd>(fd: Fd, pos: SeekFrom) -> Result<u64> {
    let mut file = fd.as_fd().file().map_err(errno)?;
    let pos = match pos {
        SeekFrom::Start(offset) => StdSeekFrom::Start(offset),
        SeekFrom::End(offset) => StdSeekFrom::End(offset),
        SeekFrom::Current(offset) => StdSeekFrom::Current(offset),
    };
    file.seek(pos).map_err(errno)
}

pub fn fcntl_getfl<Fd: AsFd>(fd: Fd) -> Result<OFlags> {
    Ok(OFlags::from_bits_retain(fd.as_fd().flags()))
}

pub fn fcntl_setfl<Fd: AsFd>(fd: Fd, flags: OFlags) -> Result<()> {
    fd.as_fd().set_flags(flags.bits());
    Ok(())
}

pub fn flock<Fd: AsFd>(_fd: Fd, _operation: FlockOperation) -> Result<()> {
    Ok(())
}

pub fn access<P: Arg>(path: P, access: Access) -> Result<()> {
    accessat(CWD, path, access, AtFlags::empty())
}

pub fn accessat<P: Arg, Fd: AsFd>(dirfd: Fd, path: P, access: Access, flags: AtFlags) -> Result<()> {
    let metadata = fs::metadata(resolve(dirfd.as_fd(), path, flags)?).map_err(errno)?;
    if access.contains(Access::WRITE_OK) && metadata.permissions().readonly() {
        return Err(Errno::ACCESS);
    }
    Ok(())
}

fn magic(filesystem: &str) -> FsWord {
    match filesystem {
        "NTFS" => 0x5346_544e,
        "FAT" | "FAT32" => 0x4d44,
        "exFAT" => 0x2011_bab0,
        "ReFS" => 0x5265_4653,
        _ => 0,
    }
}

pub fn fstatfs<Fd: AsFd>(fd: Fd) -> Result<StatFs> {
    let fd = fd.as_fd();
    let volume = win::volume(fd.file().map_err(errno)?).map_err(errno)?;
    let space = win::disk_space(&location(fd)?).map_err(errno)?;
    Ok(StatFs {
        f_type: magic(&volume.filesystem),
        f_bsize: 4096,
        f_frsize: 4096,
        f_blocks: space.total / 4096,
        f_bavail: space.available / 4096,
        f_bfree: space.available / 4096,
        f_namelen: FsWord::from(volume.name_max),
        ..StatFs::default()
    })
}

pub fn statfs<P: Arg>(path: P) -> Result<StatFs> {
    fstatfs(open(path, OFlags::PATH, Mode::empty())?)
}

pub fn fstatvfs<Fd: AsFd>(fd: Fd) -> Result<StatVfs> {
    let fs = fstatfs(fd)?;
    Ok(StatVfs {
        f_bsize: 4096,
        f_frsize: 4096,
        f_blocks: fs.f_blocks,
        f_bfree: fs.f_bfree,
        f_bavail: fs.f_bavail,
        f_files: 0,
        f_ffree: 0,
        f_favail: 0,
        f_fsid: 0,
        f_flag: StatVfsMountFlags::empty(),
        f_namemax: fs.f_namelen as u64,
    })
}

pub fn statvfs<P: Arg>(path: P) -> Result<StatVfs> {
    fstatvfs(open(path, OFlags::PATH, Mode::empty())?)
}

pub fn copy_file_range<InFd: AsFd, OutFd: AsFd>(
    _fd_in: InFd,
    _off_in: Option<&mut u64>,
    _fd_out: OutFd,
    _off_out: Option<&mut u64>,
    _len: usize,
) -> Result<usize> {
    Err(Errno::NOSYS)
}

pub fn ioctl_ficlone<Fd: AsFd, SrcFd: AsFd>(_fd: Fd, _src_fd: SrcFd) -> Result<()> {
    Err(Errno::OPNOTSUPP)
}

pub fn ioctl_getflags<Fd: AsFd>(_fd: Fd) -> Result<IFlags> {
    Err(Errno::NOTTY)
}

pub fn getxattr<P: Arg, Name: Arg, Buf: Buffer<u8>>(_path: P, _name: Name, _value: Buf) -> Result<Buf::Output> {
    Err(Errno::OPNOTSUPP)
}

pub fn lgetxattr<P: Arg, Name: Arg, Buf: Buffer<u8>>(_path: P, _name: Name, _value: Buf) -> Result<Buf::Output> {
    Err(Errno::OPNOTSUPP)
}

pub fn fgetxattr<Fd: AsFd, Name: Arg, Buf: Buffer<u8>>(_fd: Fd, _name: Name, _value: Buf) -> Result<Buf::Output> {
    Err(Errno::OPNOTSUPP)
}

pub fn setxattr<P: Arg, Name: Arg>(_path: P, _name: Name, _value: &[u8], _flags: XattrFlags) -> Result<()> {
    Err(Errno::OPNOTSUPP)
}

pub fn fsetxattr<Fd: AsFd, Name: Arg>(_fd: Fd, _name: Name, _value: &[u8], _flags: XattrFlags) -> Result<()> {
    Err(Errno::OPNOTSUPP)
}

pub fn flistxattr<Fd: AsFd, Buf: Buffer<u8>>(_fd: Fd, _list: Buf) -> Result<Buf::Output> {
    Err(Errno::OPNOTSUPP)
}

pub fn mknodat<P: Arg, Fd: AsFd>(_dirfd: Fd, _path: P, _file_type: FileType, _mode: Mode, _dev: Dev) -> Result<()> {
    Err(Errno::PERM)
}

pub fn mkfifoat<P: Arg, Fd: AsFd>(_dirfd: Fd, _path: P, _mode: Mode) -> Result<()> {
    Err(Errno::PERM)
}

/// An anonymous file: a temporary file that Windows deletes when the last handle closes.
pub fn memfd_create<P: Arg>(name: P, _flags: MemfdFlags) -> Result<OwnedFd> {
    let stem = name.to_path().map_err(errno)?;
    let unique = format!(
        "fahras-{}-{}-{}",
        std::process::id(),
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos(),
        stem.file_name().unwrap_or(OsStr::new("memfd")).to_string_lossy()
    );
    let path = std::env::temp_dir().join(unique);
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create_new(true)
        .share_mode(7)
        .custom_flags(FILE_FLAG_DELETE_ON_CLOSE)
        .open(&path)
        .map_err(errno)?;
    Ok(OwnedFd::new(file, Some(path), OFlags::RDWR.bits()))
}
