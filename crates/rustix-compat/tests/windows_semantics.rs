// SPDX-License-Identifier: MIT

//! Linux behavior Fahras's file operations rely on, checked on Windows.

#![cfg(windows)]

use std::{
    fs,
    path::{Path, PathBuf},
    time::{Duration, UNIX_EPOCH},
};

use rustix_compat::fs::{
    Access, AtFlags, CWD, Mode, OFlags, Timespec, Timestamps, accessat, fsync, open, renameat,
    unlinkat, utimensat,
};

fn scratch(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("fahras-compat-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).expect("scratch folder");
    path
}

fn set_readonly(path: &Path, readonly: bool) {
    let mut permissions = fs::metadata(path).expect("metadata").permissions();
    permissions.set_readonly(readonly);
    fs::set_permissions(path, permissions).expect("permissions");
}

#[test]
fn a_file_is_created_through_a_read_only_descriptor() {
    let folder = scratch("create");
    let path = folder.join("claimed.txt");
    let fd = open(
        &path,
        OFlags::RDONLY | OFlags::CREATE | OFlags::EXCL,
        Mode::RUSR | Mode::WUSR,
    )
    .expect("Linux creates through O_RDONLY | O_CREAT | O_EXCL");
    drop(fd);
    assert!(path.is_file());
    assert!(
        open(
            &path,
            OFlags::RDONLY | OFlags::CREATE | OFlags::EXCL,
            Mode::RUSR
        )
        .is_err()
    );
}

#[test]
fn a_read_only_handle_flushes_without_error() {
    let folder = scratch("flush");
    let path = folder.join("locked.txt");
    fs::write(&path, b"data").expect("file");
    set_readonly(&path, true);
    let fd = open(&path, OFlags::RDONLY, Mode::empty()).expect("read-only open");
    fsync(&fd).expect("a read-only file's data is already on disk");
    set_readonly(&path, false);
}

#[test]
fn folders_with_the_read_only_attribute_stay_writable() {
    let folder = scratch("access");
    let customized = folder.join("Documents");
    fs::create_dir(&customized).expect("folder");
    set_readonly(&customized, true);
    accessat(CWD, &customized, Access::WRITE_OK, AtFlags::empty())
        .expect("Windows ignores read-only on folders");
    let file = folder.join("locked.txt");
    fs::write(&file, b"data").expect("file");
    set_readonly(&file, true);
    assert!(accessat(CWD, &file, Access::WRITE_OK, AtFlags::empty()).is_err());
    set_readonly(&file, false);
    set_readonly(&customized, false);
}

#[test]
fn renaming_replaces_a_read_only_file() {
    let folder = scratch("rename");
    let from = folder.join("new.txt");
    let to = folder.join("old.txt");
    fs::write(&from, b"new").expect("source");
    fs::write(&to, b"old").expect("target");
    set_readonly(&to, true);
    renameat(CWD, &from, CWD, &to).expect("Linux replaces a read-only file");
    assert_eq!(fs::read(&to).expect("replaced"), b"new");
    assert!(!from.exists());
}

#[test]
fn a_link_gets_its_own_times_and_its_target_keeps_theirs() {
    let folder = scratch("times");
    let target = folder.join("target.txt");
    fs::write(&target, b"data").expect("target");
    let link = folder.join("link.txt");
    if std::os::windows::fs::symlink_file(&target, &link).is_err() {
        eprintln!("skipping: creating symlinks needs Developer Mode or administrator rights");
        return;
    }
    let before = fs::metadata(&target)
        .expect("target")
        .modified()
        .expect("mtime");
    let stamp = Timespec {
        tv_sec: 1_000_000_000,
        tv_nsec: 0,
    };
    utimensat(
        CWD,
        &link,
        &Timestamps {
            last_access: stamp,
            last_modification: stamp,
        },
        AtFlags::SYMLINK_NOFOLLOW,
    )
    .expect("link times");
    assert_eq!(
        fs::metadata(&target)
            .expect("target")
            .modified()
            .expect("mtime"),
        before
    );
    assert_eq!(
        fs::symlink_metadata(&link)
            .expect("link")
            .modified()
            .expect("mtime"),
        UNIX_EPOCH + Duration::from_secs(1_000_000_000)
    );
}

#[test]
fn a_broken_folder_link_is_removed() {
    let folder = scratch("unlink");
    let gone = folder.join("gone");
    fs::create_dir(&gone).expect("target");
    let link = folder.join("link");
    if std::os::windows::fs::symlink_dir(&gone, &link).is_err() {
        eprintln!("skipping: creating symlinks needs Developer Mode or administrator rights");
        return;
    }
    fs::remove_dir(&gone).expect("break the link");
    unlinkat(CWD, &link, AtFlags::empty()).expect("a dangling folder link is unlinked");
    assert!(fs::symlink_metadata(&link).is_err());
}
