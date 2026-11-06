// SPDX-License-Identifier: MIT

use std::ffi::OsString;

use super::*;
use crate::model::{EntryKind, Location};

fn entry(kind: EntryKind, size: u64) -> FileEntry {
    FileEntry {
        location: Location::local("/tmp/entry"),
        native_name: OsString::from("entry"),
        thumbnail_path: None,
        display_name: "entry".to_owned(),
        kind,
        size: MetadataValue::Known(size),
        modified_unix_seconds: MetadataValue::Unknown,
        created_unix_seconds: MetadataValue::Unknown,
        mode: MetadataValue::Unknown,
        recent_unix_seconds: MetadataValue::Unknown,
        is_hidden: false,
        image_dimensions: MetadataValue::Unknown,
        child_count: MetadataValue::Known(0),
        duration_seconds: MetadataValue::Unknown,
        listable: MetadataValue::Unknown,
        recent_uri: None,
    }
}

#[test]
fn details_columns_read_like_file_explorer_while_its_look_is_on() {
    crate::test_support::gtk_test(
        "fahras::explorer_details::tests::details_columns_read_like_file_explorer_while_its_look_is_on",
        || {
            let file = entry(EntryKind::File, 1);
            let folder = entry(EntryKind::Directory, 0);
            let local = gtk::glib::DateTime::from_local(2026, 11, 6, 15, 7, 0.0).expect("date");
            let seconds = local.to_unix();
            set_polished(false);
            assert_eq!(size(&file), None, "Fahras's own look keeps its columns");
            assert_eq!(kind(&folder), None);
            assert_eq!(date(seconds), None);

            set_polished(true);
            rust_i18n::set_locale("en");
            assert_eq!(size(&file).as_deref(), Some("1 KB"));
            assert_eq!(size(&entry(EntryKind::File, 0)).as_deref(), Some("0 KB"));
            assert_eq!(size(&entry(EntryKind::File, 1025)).as_deref(), Some("2 KB"));
            assert_eq!(
                size(&entry(EntryKind::File, 1_500_000)).as_deref(),
                Some("1,465 KB")
            );
            assert_eq!(size(&folder).as_deref(), Some(""), "folders show no size");
            assert_eq!(kind(&folder).as_deref(), Some("File folder"));
            assert_eq!(kind(&file), None, "files keep their own type");
            assert_eq!(date(seconds).as_deref(), Some("11/6/2026 3:07 PM"));

            rust_i18n::set_locale("ar");
            assert_eq!(kind(&folder).as_deref(), Some("مجلد ملفات"));
            assert!(
                size(&file).is_some_and(|text| text.ends_with("كيلوبايت")),
                "{:?}",
                size(&file)
            );
        },
    );
}
