// SPDX-License-Identifier: MIT

use super::*;

#[test]
fn undescribed_windows_types_are_named_after_their_extension() {
    crate::test_support::gtk_test(
        "fahras::file_types::tests::undescribed_windows_types_are_named_after_their_extension",
        || {
            for (locale, named) in [("en", "CSV File"), ("ar", "ملف CSV"), ("de", "CSV-Datei")] {
                rust_i18n::set_locale(locale);
                assert_eq!(readable(".csv", ".csv filetype"), named);
            }
            rust_i18n::set_locale("en");
            assert_eq!(readable(".txt", "Text Document"), "Text Document");
            assert_eq!(readable("text/csv", "CSV document"), "CSV document");
            assert_eq!(readable(".", ". filetype"), ". filetype");
        },
    );
}
