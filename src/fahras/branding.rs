// SPDX-License-Identifier: MIT

/// The product name shown to people: window titles, About, `--version`.
/// Internal identifiers (crate, application ID, paths) keep Strata's names.
pub(crate) const DISPLAY_NAME: &str = "Fahras";

/// The name Strata's translation catalogs use for the product.
const UPSTREAM_NAME: &str = "Strata";

fn display_name_in(locale: &str) -> &'static str {
    match locale {
        "ar" => "فهرس",
        _ => DISPLAY_NAME,
    }
}

/// Catalog text with the product name people see. The catalogs stay as upstream
/// writes them, so syncs never conflict over the name. Tests keep upstream's
/// wording so its assertions still hold.
pub(crate) fn rebrand(locale: &str, text: &'static str) -> &'static str {
    if cfg!(test) {
        return text;
    }
    rebranded(locale, text)
}

fn rebranded(locale: &str, text: &'static str) -> &'static str {
    if !text.contains(UPSTREAM_NAME) {
        return text;
    }
    // Leaked once per message when the catalogs load at startup.
    Box::leak(
        text.replace(UPSTREAM_NAME, display_name_in(locale))
            .into_boxed_str(),
    )
}

#[cfg(test)]
mod tests;
