// SPDX-License-Identifier: MIT

//! Type names for files Windows has no description for.

/// GLib on Windows names a type it cannot describe "<extension> filetype",
/// in English. File Explorer calls such a file "CSV File"; so does Fahras,
/// in the interface language.
pub(crate) fn readable(content_type: &str, description: &str) -> String {
    let extension = content_type.strip_prefix('.').unwrap_or_default();
    if extension.is_empty() || description != format!("{content_type} filetype") {
        return description.to_owned();
    }
    rust_i18n::t!(
        "fahras.file_type.extension",
        extension = extension.to_uppercase()
    )
    .into_owned()
}

#[cfg(test)]
mod tests;
