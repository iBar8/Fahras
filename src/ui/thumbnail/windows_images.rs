// SPDX-License-Identifier: MIT

//! Windows' own icons and thumbnails in the polished Explorer looks, read on a
//! worker thread and cached by file type, or by file for pictures and programs.

use std::{
    cell::RefCell,
    collections::HashMap,
    path::{Path, PathBuf},
    sync::mpsc,
};

use gtk::{gdk, glib, prelude::*};
use shell_images::Kind;

use super::ThumbnailSlot;

/// Below this size Explorer shows the type's icon, as in Details view.
const THUMBNAIL_MIN_PX: u32 = 48;

/// Extensions whose files carry their own icon.
const OWN_ICON: [&str; 6] = ["exe", "lnk", "ico", "url", "msc", "cpl"];

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct Key {
    source: String,
    pixels: u32,
    kind: Kind,
}

struct Job {
    key: Key,
    path: PathBuf,
}

thread_local! {
    static CACHE: RefCell<HashMap<Key, Option<gdk::Texture>>> = RefCell::new(HashMap::new());
    static WAITING: RefCell<HashMap<Key, Vec<glib::WeakRef<ThumbnailSlot>>>> =
        RefCell::new(HashMap::new());
    /// The file each slot shows, so a late image never lands on a reused row.
    static SHOWN: RefCell<HashMap<usize, PathBuf>> = RefCell::new(HashMap::new());
    static WORKER: RefCell<Option<mpsc::Sender<Job>>> = const { RefCell::new(None) };
}

fn key_for(path: &Path, pixels: u32, kind: Kind) -> Key {
    let extension = path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_lowercase);
    let per_file = path.is_dir()
        || kind == Kind::ThumbnailOrIcon
        || extension
            .as_deref()
            .is_none_or(|extension| OWN_ICON.contains(&extension));
    let source = if per_file {
        path.to_string_lossy().into_owned()
    } else {
        format!("*.{}", extension.unwrap_or_default())
    };
    Key {
        source,
        pixels,
        kind,
    }
}

fn texture(image: shell_images::Image) -> gdk::Texture {
    let stride = image.width as usize * 4;
    gdk::MemoryTexture::new(
        image.width as i32,
        image.height as i32,
        gdk::MemoryFormat::B8g8r8a8Premultiplied,
        &glib::Bytes::from_owned(image.bgra),
        stride,
    )
    .upcast()
}

fn worker() -> mpsc::Sender<Job> {
    WORKER.with_borrow_mut(|worker| {
        worker
            .get_or_insert_with(|| {
                let (jobs, receive_jobs) = mpsc::channel::<Job>();
                let main = glib::MainContext::default();
                std::thread::Builder::new()
                    .name("windows-images".into())
                    .spawn(move || {
                        shell_images::initialize_thread();
                        for job in receive_jobs {
                            let image =
                                shell_images::image(&job.path, job.key.pixels, job.key.kind).ok();
                            let key = job.key;
                            main.invoke(move || deliver(key, image.map(texture)));
                        }
                    })
                    .expect("windows image thread");
                jobs
            })
            .clone()
    })
}

fn deliver(key: Key, texture: Option<gdk::Texture>) {
    CACHE.with_borrow_mut(|cache| cache.insert(key.clone(), texture.clone()));
    let slots = WAITING.with_borrow_mut(|waiting| waiting.remove(&key).unwrap_or_default());
    let Some(texture) = texture else {
        return;
    };
    for slot in slots.iter().filter_map(glib::WeakRef::upgrade) {
        let still_shown = SHOWN.with_borrow(|shown| {
            shown
                .get(&(slot.as_ptr() as usize))
                .is_some_and(|path| key_for(path, key.pixels, key.kind) == key)
        });
        if still_shown && let Some(icon) = slot.fallback_icon() {
            slot.set_fallback(&icon, Some(&texture));
        }
    }
}

/// Replaces the icon Fahras just drew for `path` with Windows' own, once it is
/// read. `path` of `None` means the slot shows something else now.
pub(super) fn show(slot: &ThumbnailSlot, path: Option<&Path>, size: i32) {
    let id = slot.as_ptr() as usize;
    let wanted = path.filter(|_| crate::fahras::explorer_details::is_polished());
    let Some(path) = wanted else {
        SHOWN.with_borrow_mut(|shown| shown.remove(&id));
        return;
    };
    SHOWN.with_borrow_mut(|shown| shown.insert(id, path.to_path_buf()));
    let pixels = (size.max(16) * slot.scale_factor().max(1)) as u32;
    let kind = if pixels >= THUMBNAIL_MIN_PX * slot.scale_factor().max(1) as u32 {
        Kind::ThumbnailOrIcon
    } else {
        Kind::Icon
    };
    let key = key_for(path, pixels, kind);
    if let Some(cached) = CACHE.with_borrow(|cache| cache.get(&key).cloned()) {
        if let (Some(texture), Some(icon)) = (cached, slot.fallback_icon()) {
            slot.set_fallback(&icon, Some(&texture));
        }
        return;
    }
    let first = WAITING.with_borrow_mut(|waiting| {
        let slots = waiting.entry(key.clone()).or_default();
        slots.push(slot.downgrade());
        slots.len() == 1
    });
    if first {
        let _ = worker().send(Job {
            key,
            path: path.to_path_buf(),
        });
    }
}
