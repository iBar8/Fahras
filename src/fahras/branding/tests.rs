// SPDX-License-Identifier: MIT

use super::rebranded;

#[test]
fn catalog_text_names_the_product_people_see() {
    for (locale, text, expected) in [
        (
            "en",
            "Restart Strata to apply the change.",
            "Restart Fahras to apply the change.",
        ),
        ("de", "Die Strata-Programmdatei", "Die Fahras-Programmdatei"),
        ("ar", "أعد تشغيل Strata.", "أعد تشغيل فهرس."),
        ("en", "Copy", "Copy"),
        ("en", "~/.config/strata", "~/.config/strata"),
    ] {
        assert_eq!(rebranded(locale, text), expected, "{locale}: {text}");
    }
}
