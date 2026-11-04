// SPDX-License-Identifier: MIT

use std::rc::Rc;

use gtk::{pango, prelude::*};

use crate::{
    assets::icons,
    fahras::arabic_fonts::{self, FONTS},
    ui::{accessibility, preferences::PreferenceManager},
};

/// Lists the bundled Arabic families, each with a sample drawn in its own typeface.
pub(super) fn arabic_font_choice(preferences: &Rc<PreferenceManager>) -> gtk::MenuButton {
    let list = font_list();
    let scroll = gtk::ScrolledWindow::builder()
        .child(&list)
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vscrollbar_policy(gtk::PolicyType::Automatic)
        .propagate_natural_height(true)
        .propagate_natural_width(true)
        .max_content_height(380)
        .build();
    scroll.add_css_class("context-menu-scroll");
    let popover = gtk::Popover::builder()
        .child(&scroll)
        .has_arrow(false)
        .build();
    popover.add_css_class("column-popover");
    let button = gtk::MenuButton::builder()
        .popover(&popover)
        .always_show_arrow(true)
        .valign(gtk::Align::Center)
        .build();
    button.add_css_class("form-control");
    button.add_css_class("settings-choice");
    accessibility::set_label(&button, &crate::i18n::tr("Arabic font"));
    bind_choice_label(preferences, &button);
    bind_checks(preferences, &list);
    connect_choice(preferences, &list, &button);
    button
}

fn font_list() -> gtk::ListBox {
    let list = gtk::ListBox::new();
    list.set_selection_mode(gtk::SelectionMode::None);
    list.set_activate_on_single_click(true);
    list.add_css_class("column-menu");
    list.add_css_class("font-choice-list");
    accessibility::set_label(&list, &crate::i18n::tr("Arabic font"));
    for font in FONTS {
        list.append(&font_row(font.family));
    }
    list
}

fn font_row(family: &'static str) -> gtk::ListBoxRow {
    let name = gtk::Label::new(Some(family));
    name.set_xalign(0.0);
    name.add_css_class("settings-option-description");
    let sample = gtk::Label::new(Some(arabic_fonts::SAMPLE));
    sample.set_attributes(Some(&family_attributes(family)));
    sample.set_single_line_mode(true);
    sample.set_halign(gtk::Align::Start);
    let copy = gtk::Box::new(gtk::Orientation::Vertical, 2);
    copy.set_hexpand(true);
    copy.append(&sample);
    copy.append(&name);
    let check = crate::assets::primary_icon(icons::CHECK, 16);
    check.set_valign(gtk::Align::Center);
    let content = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    content.append(&copy);
    content.append(&check);
    let row = gtk::ListBoxRow::new();
    row.set_child(Some(&content));
    accessibility::set_label(&row, family);
    row
}

fn family_attributes(family: &str) -> pango::AttrList {
    let attributes = pango::AttrList::new();
    attributes.insert(pango::AttrString::new_family(family));
    attributes.insert(pango::AttrFloat::new_scale(1.2));
    attributes
}

fn row_family(row: &gtk::ListBoxRow) -> Option<&'static str> {
    usize::try_from(row.index())
        .ok()
        .and_then(|index| FONTS.get(index))
        .map(|font| font.family)
}

fn rows(list: &gtk::ListBox) -> impl Iterator<Item = gtk::ListBoxRow> + '_ {
    (0..).map_while(|index| list.row_at_index(index))
}

fn bind_choice_label(preferences: &Rc<PreferenceManager>, button: &gtk::MenuButton) {
    preferences.bind_preference(button, PreferenceManager::arabic_font, |widget, choice| {
        if let Some(button) = widget.downcast_ref::<gtk::MenuButton>() {
            super::bindings::show_choice_value(
                button,
                arabic_fonts::resolved_family(choice.as_deref()),
            );
        }
    });
}

fn bind_checks(preferences: &Rc<PreferenceManager>, list: &gtk::ListBox) {
    preferences.bind_preference(list, PreferenceManager::arabic_font, |widget, choice| {
        let Some(list) = widget.downcast_ref::<gtk::ListBox>() else {
            return;
        };
        let chosen = arabic_fonts::resolved_family(choice.as_deref());
        for row in rows(list) {
            if let Some(check) = row.child().and_then(|content| content.last_child()) {
                check.set_visible(row_family(&row) == Some(chosen));
            }
        }
    });
}

fn connect_choice(
    preferences: &Rc<PreferenceManager>,
    list: &gtk::ListBox,
    button: &gtk::MenuButton,
) {
    let preferences = preferences.clone();
    let button = button.downgrade();
    list.connect_row_activated(move |_, row| {
        if let Some(family) = row_family(row) {
            preferences.set_arabic_font(Some(family.to_owned()));
        }
        if let Some(button) = button.upgrade() {
            button.popdown();
        }
    });
}

#[cfg(test)]
mod tests;
