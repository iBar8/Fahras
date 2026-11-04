// SPDX-License-Identifier: MIT

use super::{DEFAULT_FAMILY, FONTS, materialize, resolved_family};

#[test]
fn saved_choices_resolve_to_bundled_families() {
    for (choice, expected) in [
        (None, DEFAULT_FAMILY),
        (Some("Amiri"), "Amiri"),
        (Some("Noto Kufi Arabic"), "Noto Kufi Arabic"),
        (Some("Uninstalled Family"), DEFAULT_FAMILY),
        (Some(""), DEFAULT_FAMILY),
    ] {
        assert_eq!(resolved_family(choice), expected, "{choice:?}");
    }
}

#[test]
fn every_bundled_font_is_written_once_for_registration() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let written = std::cell::RefCell::new(Vec::new());
    let paths = materialize(directory.path(), |path, contents| {
        written.borrow_mut().push(path.to_owned());
        std::fs::write(path, contents)
    })
    .expect("materialize fonts");
    let expected: usize = FONTS.iter().map(|font| font.files.len()).sum();
    assert_eq!(paths.len(), expected);
    assert_eq!(*written.borrow(), paths);
    for path in &paths {
        assert!(path.starts_with(directory.path().join("arabic")));
        assert!(std::fs::metadata(path).expect("font file").len() > 0);
    }
}
