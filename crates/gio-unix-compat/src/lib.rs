// SPDX-License-Identifier: MIT

//! The parts of [gio-unix](https://docs.rs/gio-unix) Fahras uses, for Windows.
//! Mount changes come from GIO's cross-platform volume monitor; Unix mount
//! tables and `.desktop` application entries do not exist and come back empty.

use std::{cell::RefCell, collections::HashMap, path::PathBuf, rc::Rc};

use gio::prelude::*;
use glib::SignalHandlerId;

pub mod functions {
    use std::path::Path;

    /// Windows has no system-internal mount points like `/proc` or `/sys`.
    pub fn is_mount_path_system_internal(_mount_path: impl AsRef<Path>) -> bool {
        false
    }
}

fn raw(handler: &SignalHandlerId) -> std::ffi::c_ulong {
    // SAFETY: the raw id only serves as a map key and is never passed back to GLib.
    unsafe { handler.as_raw() }
}

/// Reports mount changes through GIO's volume monitor; one handler id covers
/// the added, removed and changed signals it connects.
#[derive(Clone)]
pub struct MountMonitor {
    monitor: gio::VolumeMonitor,
    companions: Rc<RefCell<HashMap<std::ffi::c_ulong, Vec<SignalHandlerId>>>>,
}

impl MountMonitor {
    pub fn get() -> Self {
        Self {
            monitor: gio::VolumeMonitor::get(),
            companions: Rc::default(),
        }
    }

    pub fn connect_mounts_changed<F: Fn(&Self) + 'static>(&self, f: F) -> SignalHandlerId {
        let callback = Rc::new(f);
        let notify = |monitor: &Self| {
            let monitor = monitor.clone();
            let callback = callback.clone();
            move |_: &gio::VolumeMonitor, _: &gio::Mount| callback(&monitor)
        };
        let added = self.monitor.connect_mount_added(notify(self));
        let removed = self.monitor.connect_mount_removed(notify(self));
        let changed = self.monitor.connect_mount_changed(notify(self));
        self.companions
            .borrow_mut()
            .insert(raw(&added), vec![removed, changed]);
        added
    }

    pub fn disconnect(&self, handler: SignalHandlerId) {
        if let Some(companions) = self.companions.borrow_mut().remove(&raw(&handler)) {
            for companion in companions {
                self.monitor.disconnect(companion);
            }
        }
        self.monitor.disconnect(handler);
    }
}

/// `.desktop` application entries are a freedesktop concept; none exist on Windows.
#[derive(Clone, Debug)]
pub struct DesktopAppInfo(());

impl DesktopAppInfo {
    pub fn new(_desktop_id: &str) -> Option<Self> {
        None
    }

    pub fn from_filename(_filename: impl AsRef<std::path::Path>) -> Option<Self> {
        None
    }

    pub fn name(&self) -> glib::GString {
        glib::GString::new()
    }

    pub fn executable(&self) -> PathBuf {
        PathBuf::new()
    }

    pub fn boolean(&self, _key: &str) -> bool {
        false
    }
}

/// Entries of `/etc/fstab`, which Windows does not have.
#[derive(Clone, Debug)]
pub struct MountPoint(());

impl MountPoint {
    pub fn mount_points() -> (Vec<MountPoint>, u64) {
        (Vec::new(), 0)
    }

    pub fn device_path(&self) -> PathBuf {
        PathBuf::new()
    }

    pub fn fs_type(&self) -> glib::GString {
        glib::GString::new()
    }
}
