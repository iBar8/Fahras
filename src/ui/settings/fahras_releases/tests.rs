// SPDX-License-Identifier: MIT

use super::*;

fn texts(root: &gtk::Widget) -> Vec<String> {
    let mut found = Vec::new();
    let mut pending = vec![root.clone()];
    while let Some(widget) = pending.pop() {
        if let Some(label) = widget.downcast_ref::<gtk::Label>() {
            found.push(label.text().to_string());
        }
        let mut child = widget.first_child();
        while let Some(current) = child {
            child = current.next_sibling();
            pending.push(current);
        }
    }
    found
}

#[test]
fn linux_builds_point_to_the_releases_page_instead_of_updating() {
    crate::test_support::gtk_test(
        "ui::settings::fahras_releases::tests::linux_builds_point_to_the_releases_page_instead_of_updating",
        || {
            assert!(
                crate::build_info::updater_enabled(),
                "Windows keeps updating itself"
            );
            crate::fahras::updates::act_as_linux_release_for_test();
            assert!(
                !crate::build_info::updater_enabled(),
                "no checks, downloads or installs"
            );
            assert!(
                super::super::navigation::page_shown("updates"),
                "the Updates page stays"
            );
            for (locale, description, button) in [
                (
                    "en",
                    "On Linux, Fahras does not update itself. New versions come from the Fahras Releases page.",
                    "Open Releases Page",
                ),
                (
                    "ar",
                    "على لينكس، لا يحدّث فهرس نفسه. تتوفر الإصدارات الجديدة في صفحة إصدارات فهرس.",
                    "فتح صفحة الإصدارات",
                ),
            ] {
                rust_i18n::set_locale(locale);
                let page = releases_page();
                let shown = texts(&page);
                assert!(
                    shown.iter().any(|text| text == description),
                    "{locale}: {shown:?}"
                );
                assert!(
                    shown.iter().any(|text| text == button),
                    "{locale}: {shown:?}"
                );
            }
        },
    );
}
