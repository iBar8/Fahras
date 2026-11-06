// SPDX-License-Identifier: MIT

//! The window as File Explorer lays it out, shown while an Explorer look is
//! on: the system title bar, an address row with back, forward and up, a
//! boxed address bar and a search box, Windows 11's command bar or Windows
//! 10's ribbon, and a status bar with item and selection counts. Fahras's own
//! header and footer return when the look is turned off.

use std::rc::Rc;

use gtk::{glib, prelude::*};

use super::{FooterBinding, Header, build_appearance_menu};
use crate::{
    assets,
    fahras::look::ExplorerSkin,
    ui::{browser::BrowserView, preferences::PreferenceManager, preview::PreviewDrawer},
};

const COMMAND_ICON_PX: i32 = 16;
const RIBBON_ICON_PX: i32 = 24;

pub(in crate::ui::window) struct ExplorerParts<'a> {
    pub(in crate::ui::window) window: &'a gtk::ApplicationWindow,
    pub(in crate::ui::window) root: &'a gtk::Box,
    pub(in crate::ui::window) header: &'a Header,
    pub(in crate::ui::window) browser: &'a BrowserView,
    pub(in crate::ui::window) preview: &'a PreviewDrawer,
    pub(in crate::ui::window) footer: &'a FooterBinding,
    pub(in crate::ui::window) preferences: &'a Rc<PreferenceManager>,
}

/// One Explorer layout per tab; it follows the look live.
pub(in crate::ui::window) fn install(parts: &ExplorerParts) {
    let ribbon = ribbon(parts);
    let (address_row, address_slot) = address_row(parts.browser);
    let command_bar = command_bar(parts);
    let chrome = gtk::Box::new(gtk::Orientation::Vertical, 0);
    chrome.add_css_class("explorer-chrome");
    chrome.append(&ribbon);
    chrome.append(&address_row);
    chrome.append(&command_bar);
    parts
        .root
        .insert_child_after(&chrome, Some(&parts.header.widget));
    let status = status_bar(parts.browser);
    parts.root.append(&status);

    let layout = Layout {
        root: parts.root.downgrade(),
        window: parts.window.downgrade(),
        header: parts.header.widget.downgrade(),
        header_content: parts.header.content.downgrade(),
        sidebar_toggle: parts
            .header
            .sidebar_toggle
            .clone()
            .upcast::<gtk::Widget>()
            .downgrade(),
        location: parts.browser.location_widget().downgrade(),
        address_slot: address_slot.downgrade(),
        footer: parts
            .footer
            .shortcuts
            .widget()
            .clone()
            .upcast::<gtk::Widget>()
            .downgrade(),
        status: status.downgrade(),
        ribbon: ribbon.downgrade(),
        command_bar: command_bar.downgrade(),
    };
    parts.preferences.bind_preference(
        &chrome,
        PreferenceManager::explorer_skin,
        move |chrome, skin| layout.apply(chrome, skin),
    );
}

struct Layout {
    root: glib::WeakRef<gtk::Box>,
    window: glib::WeakRef<gtk::ApplicationWindow>,
    header: glib::WeakRef<gtk::HeaderBar>,
    header_content: glib::WeakRef<gtk::Box>,
    sidebar_toggle: glib::WeakRef<gtk::Widget>,
    location: glib::WeakRef<gtk::Widget>,
    address_slot: glib::WeakRef<gtk::Box>,
    footer: glib::WeakRef<gtk::Widget>,
    status: glib::WeakRef<gtk::Box>,
    ribbon: glib::WeakRef<gtk::Box>,
    command_bar: glib::WeakRef<gtk::Box>,
}

impl Layout {
    fn apply(&self, chrome: &gtk::Widget, skin: ExplorerSkin) {
        let explorer = skin != ExplorerSkin::Off;
        chrome.set_visible(explorer);
        set_visible(&self.header, !explorer);
        self.place_footer(explorer);
        set_visible(&self.status, explorer);
        set_visible(&self.ribbon, skin == ExplorerSkin::Windows10);
        set_visible(&self.command_bar, skin == ExplorerSkin::Windows11);
        self.place_location(explorer);
        if let Some(window) = self.window.upgrade() {
            use_system_title_bar(&window, explorer);
        }
    }

    /// The footer shows itself whenever its status changes, so the Explorer
    /// layout takes it out rather than hiding it; it goes back above the
    /// Explorer status bar.
    fn place_footer(&self, explorer: bool) {
        let (Some(root), Some(footer), Some(status)) = (
            self.root.upgrade(),
            self.footer.upgrade(),
            self.status.upgrade(),
        ) else {
            return;
        };
        let placed = footer.parent().is_some();
        if explorer && placed {
            root.remove(&footer);
        } else if !explorer && !placed {
            root.insert_child_after(&footer, status.prev_sibling().as_ref());
        }
    }

    /// The location bar is one widget, so it moves between the two layouts.
    fn place_location(&self, explorer: bool) {
        let (Some(location), Some(slot), Some(content), Some(toggle)) = (
            self.location.upgrade(),
            self.address_slot.upgrade(),
            self.header_content.upgrade(),
            self.sidebar_toggle.upgrade(),
        ) else {
            return;
        };
        let target: gtk::Widget = if explorer {
            slot.clone().upcast()
        } else {
            content.clone().upcast()
        };
        if location.parent().as_ref() == Some(&target) {
            return;
        }
        if let Some(parent) = location.parent().and_downcast::<gtk::Box>() {
            parent.remove(&location);
        }
        if explorer {
            slot.append(&location);
        } else {
            content.insert_child_after(&location, Some(&toggle));
        }
    }
}

fn set_visible<W: IsA<gtk::Widget>>(widget: &glib::WeakRef<W>, visible: bool) {
    if let Some(widget) = widget.upgrade() {
        widget.set_visible(visible);
    }
}

/// Explorer uses the system's own title bar with minimize, maximize and close;
/// Fahras's look draws its own header instead. A frame only changes while the
/// window is unrealized.
fn use_system_title_bar(window: &gtk::ApplicationWindow, system: bool) {
    if system == window.titlebar().is_none() {
        return;
    }
    let shown = window.is_visible();
    if window.is_realized() {
        window.set_visible(false);
        WidgetExt::unrealize(window);
    }
    if system {
        window.set_titlebar(None::<&gtk::Widget>);
    } else {
        crate::ui::decoration::claim_titlebar(window);
    }
    if shown {
        window.present();
    }
}

fn address_row(browser: &BrowserView) -> (gtk::Box, gtk::Box) {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 4);
    row.add_css_class("explorer-address-row");
    let controller = browser.browser();
    let rtl = gtk::Widget::default_direction() == gtk::TextDirection::Rtl;
    let (back_icon, forward_icon) = if rtl {
        (
            "fahras-fluent-forward-symbolic",
            "fahras-fluent-back-symbolic",
        )
    } else {
        (
            "fahras-fluent-back-symbolic",
            "fahras-fluent-forward-symbolic",
        )
    };
    let back = nav_button(back_icon, "Back");
    let forward = nav_button(forward_icon, "Forward");
    let up = nav_button("fahras-fluent-up-symbolic", "Up");
    let refresh = nav_button("fahras-fluent-refresh-symbolic", "Refresh");
    {
        let controller = controller.clone();
        back.connect_clicked(move |_| controller.back());
    }
    {
        let controller = controller.clone();
        forward.connect_clicked(move |_| controller.forward());
    }
    {
        let view = browser.clone();
        up.connect_clicked(move |_| view.navigate_up());
    }
    {
        let view = browser.clone();
        refresh.connect_clicked(move |_| view.refresh());
    }
    let address = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    address.add_css_class("explorer-address");
    address.set_hexpand(true);
    let search = gtk::SearchEntry::new();
    search.add_css_class("explorer-search");
    crate::ui::accessibility::set_label(&search, &crate::i18n::tr("Search"));
    {
        let view = browser.clone();
        search.connect_search_changed(move |entry| {
            let query = entry.text();
            if query.is_empty() {
                view.clear_listing_filter();
            } else {
                view.set_listing_filter(&query);
            }
        });
    }
    for button in [&back, &forward, &up, &refresh] {
        row.append(button);
    }
    row.append(&address);
    row.append(&search);
    follow_navigation(&controller, &back, &forward, &up, &search);
    (row, address)
}

fn follow_navigation(
    controller: &Rc<crate::app::Browser>,
    back: &gtk::Button,
    forward: &gtk::Button,
    up: &gtk::Button,
    search: &gtk::SearchEntry,
) {
    let update = {
        let weak = Rc::downgrade(controller);
        let (back, forward, up, search) = (
            back.downgrade(),
            forward.downgrade(),
            up.downgrade(),
            search.downgrade(),
        );
        move || {
            let Some(controller) = weak.upgrade() else {
                return;
            };
            if let Some(back) = back.upgrade() {
                back.set_sensitive(controller.can_go_back());
            }
            if let Some(forward) = forward.upgrade() {
                forward.set_sensitive(controller.can_go_forward());
            }
            if let Some(up) = up.upgrade() {
                up.set_sensitive(controller.can_go_parent());
            }
            if let Some(search) = search.upgrade() {
                let folder = controller
                    .active_location()
                    .map(|location| location.display_name())
                    .unwrap_or_default();
                search.set_placeholder_text(Some(&rust_i18n::t!(
                    "fahras.explorer.search_in",
                    folder = folder
                )));
            }
        }
    };
    update();
    controller.observe(move |_| update());
}

fn nav_button(icon: &str, label: &str) -> gtk::Button {
    let label = crate::i18n::tr(label);
    let button = gtk::Button::new();
    button.set_child(Some(&assets::text_icon(icon, COMMAND_ICON_PX)));
    button.add_css_class("explorer-nav");
    button.set_tooltip_text(Some(&label));
    crate::ui::accessibility::set_label(&button, &label);
    button
}

/// A command with its label beside the icon (Windows 11) or under it
/// (the Windows 10 ribbon). Icon-only commands take their label as tooltip.
fn command_content(icon: &str, label: Option<&str>, ribbon: bool) -> gtk::Box {
    let orientation = if ribbon {
        gtk::Orientation::Vertical
    } else {
        gtk::Orientation::Horizontal
    };
    let content = gtk::Box::new(orientation, if ribbon { 2 } else { 6 });
    let size = if ribbon {
        RIBBON_ICON_PX
    } else {
        COMMAND_ICON_PX
    };
    content.append(&assets::text_icon(icon, size));
    if let Some(label) = label {
        content.append(&gtk::Label::new(Some(&crate::i18n::tr(label))));
    }
    content
}

fn command(icon: &str, label: &str, show_label: bool, ribbon: bool) -> gtk::Button {
    let button = gtk::Button::new();
    button.set_child(Some(&command_content(
        icon,
        show_label.then_some(label),
        ribbon,
    )));
    button.add_css_class(if ribbon {
        "explorer-ribbon-command"
    } else {
        "explorer-command"
    });
    let label = crate::i18n::tr(label);
    if !show_label {
        button.set_tooltip_text(Some(&label));
    }
    crate::ui::accessibility::set_label(&button, &label);
    button
}

fn menu_command(icon: &str, label: &str, ribbon: bool) -> gtk::MenuButton {
    let button = gtk::MenuButton::new();
    button.set_child(Some(&command_content(icon, Some(label), ribbon)));
    button.set_always_show_arrow(!ribbon);
    button.add_css_class(if ribbon {
        "explorer-ribbon-command"
    } else {
        "explorer-command"
    });
    crate::ui::accessibility::set_label(&button, &crate::i18n::tr(label));
    button
}

/// A labelled menu entry and what it runs.
type MenuItem<'a> = (&'a str, Rc<dyn Fn()>);

fn popover_with(items: &[MenuItem]) -> gtk::Popover {
    let menu = gtk::Box::new(gtk::Orientation::Vertical, 2);
    menu.add_css_class("column-menu");
    let popover = gtk::Popover::builder()
        .child(&menu)
        .has_arrow(false)
        .build();
    popover.add_css_class("column-popover");
    for (label, run) in items {
        let item = gtk::Button::with_label(&crate::i18n::tr(label));
        item.add_css_class("column-menu-option");
        let run = run.clone();
        let popover = popover.downgrade();
        item.connect_clicked(move |_| {
            if let Some(popover) = popover.upgrade() {
                popover.popdown();
            }
            run();
        });
        menu.append(&item);
    }
    popover
}

struct Commands {
    new: gtk::MenuButton,
    cut: gtk::Button,
    copy: gtk::Button,
    paste: gtk::Button,
    rename: gtk::Button,
    delete: gtk::Button,
    sort: gtk::MenuButton,
    view: gtk::MenuButton,
    more: gtk::MenuButton,
}

fn commands(parts: &ExplorerParts, ribbon: bool) -> Commands {
    let labelled = ribbon;
    let view_ref = parts.browser;
    let new = menu_command("fahras-fluent-new-symbolic", "New", ribbon);
    let new_folder: Rc<dyn Fn()> = {
        let view = view_ref.clone();
        Rc::new(move || view.create_new_folder())
    };
    new.set_popover(Some(&popover_with(&[("Folder", new_folder)])));
    let cut = command("fahras-fluent-cut-symbolic", "Cut", labelled, ribbon);
    let copy = command("fahras-fluent-copy-symbolic", "Copy", labelled, ribbon);
    let paste = command("fahras-fluent-paste-symbolic", "Paste", labelled, ribbon);
    let rename = command("fahras-fluent-rename-symbolic", "Rename", labelled, ribbon);
    let delete = command("fahras-fluent-delete-symbolic", "Delete", labelled, ribbon);
    {
        let view = view_ref.clone();
        cut.connect_clicked(move |_| {
            view.cut_selection();
        });
    }
    {
        let view = view_ref.clone();
        copy.connect_clicked(move |_| {
            view.copy_selection();
        });
    }
    {
        let view = view_ref.clone();
        paste.connect_clicked(move |_| view.paste());
    }
    {
        let view = view_ref.clone();
        rename.connect_clicked(move |_| {
            view.begin_rename();
        });
    }
    {
        let view = view_ref.clone();
        delete.connect_clicked(move |_| {
            view.confirm_delete(false);
        });
    }
    let sort = menu_command("fahras-fluent-sort-symbolic", "Sort", ribbon);
    {
        let controller = view_ref.browser();
        sort.set_create_popup_func(move |button| {
            let depth = controller.active_depth().unwrap_or_default();
            let source = crate::ui::browser::list_sort_menu(&controller, depth);
            if let Some(popover) = source.popover() {
                source.set_popover(None::<&gtk::Popover>);
                button.set_popover(Some(&popover));
            }
        });
    }
    let view = build_appearance_menu(
        parts.browser,
        &parts.browser.browser(),
        parts.preferences.clone(),
        parts.preview,
    );
    view.set_child(Some(&command_content(
        "fahras-fluent-view-symbolic",
        Some("View"),
        ribbon,
    )));
    view.set_always_show_arrow(!ribbon);
    view.add_css_class(if ribbon {
        "explorer-ribbon-command"
    } else {
        "explorer-command"
    });
    crate::ui::accessibility::set_label(&view, &crate::i18n::tr("View"));
    let more = gtk::MenuButton::new();
    more.set_child(Some(&assets::text_icon(
        "fahras-fluent-more-symbolic",
        COMMAND_ICON_PX,
    )));
    more.add_css_class(if ribbon {
        "explorer-ribbon-command"
    } else {
        "explorer-command"
    });
    let more_label = crate::i18n::tr("More options");
    more.set_tooltip_text(Some(&more_label));
    crate::ui::accessibility::set_label(&more, &more_label);
    let new_tab: Rc<dyn Fn()> = {
        let window = parts.window.downgrade();
        Rc::new(move || {
            if let Some(window) = window.upgrade() {
                let _ = WidgetExt::activate_action(&window, "win.new-tab", None);
            }
        })
    };
    let settings: Rc<dyn Fn()> = {
        let button = parts.header.settings.downgrade();
        Rc::new(move || {
            if let Some(button) = button.upgrade() {
                button.emit_clicked();
            }
        })
    };
    more.set_popover(Some(&popover_with(&[
        ("New tab", new_tab),
        ("Settings", settings),
    ])));
    Commands {
        new,
        cut,
        copy,
        paste,
        rename,
        delete,
        sort,
        view,
        more,
    }
}

fn separator() -> gtk::Separator {
    let separator = gtk::Separator::new(gtk::Orientation::Vertical);
    separator.add_css_class("explorer-separator");
    separator
}

fn command_bar(parts: &ExplorerParts) -> gtk::Box {
    let commands = commands(parts, false);
    let bar = gtk::Box::new(gtk::Orientation::Horizontal, 2);
    bar.add_css_class("explorer-command-bar");
    bar.append(&commands.new);
    bar.append(&separator());
    for button in [
        &commands.cut,
        &commands.copy,
        &commands.paste,
        &commands.rename,
        &commands.delete,
    ] {
        bar.append(button);
    }
    bar.append(&separator());
    bar.append(&commands.sort);
    bar.append(&commands.view);
    bar.append(&separator());
    bar.append(&commands.more);
    bar
}

fn ribbon_group(label: &str, buttons: &[&gtk::Widget]) -> gtk::Box {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 2);
    for button in buttons {
        row.append(*button);
    }
    let title = gtk::Label::new(Some(&crate::i18n::tr(label)));
    title.add_css_class("explorer-ribbon-group-label");
    let group = gtk::Box::new(gtk::Orientation::Vertical, 2);
    group.add_css_class("explorer-ribbon-group");
    group.append(&row);
    group.append(&title);
    group
}

fn ribbon(parts: &ExplorerParts) -> gtk::Box {
    let commands = commands(parts, true);
    let ribbon = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    ribbon.add_css_class("explorer-ribbon");
    let groups = [
        ribbon_group(
            "Clipboard",
            &[
                commands.copy.upcast_ref(),
                commands.paste.upcast_ref(),
                commands.cut.upcast_ref(),
            ],
        ),
        ribbon_group(
            "Organize",
            &[commands.delete.upcast_ref(), commands.rename.upcast_ref()],
        ),
        ribbon_group("New", &[commands.new.upcast_ref()]),
        ribbon_group(
            "View",
            &[
                commands.sort.upcast_ref(),
                commands.view.upcast_ref(),
                commands.more.upcast_ref(),
            ],
        ),
    ];
    for (index, group) in groups.iter().enumerate() {
        if index > 0 {
            ribbon.append(&separator());
        }
        ribbon.append(group);
    }
    ribbon
}

/// Explorer's status bar: the folder's item count, then the selection.
fn status_bar(browser: &BrowserView) -> gtk::Box {
    let bar = gtk::Box::new(gtk::Orientation::Horizontal, 24);
    bar.add_css_class("explorer-status");
    let items = gtk::Label::new(None);
    let selected = gtk::Label::new(None);
    bar.append(&items);
    bar.append(&selected);
    let controller = browser.browser();
    let update = {
        let weak = Rc::downgrade(&controller);
        let (items, selected) = (items.downgrade(), selected.downgrade());
        move || {
            let (Some(controller), Some(items), Some(selected)) =
                (weak.upgrade(), items.upgrade(), selected.upgrade())
            else {
                return;
            };
            let total = controller
                .active_depth()
                .and_then(|depth| controller.column_entry_counts(depth))
                .map_or(0, |counts| counts.total);
            items.set_label(&crate::i18n::count("items", total));
            let chosen = controller.selected_count();
            selected.set_visible(chosen > 0);
            selected.set_label(&rust_i18n::t!(
                "%{items} selected",
                items = crate::i18n::count("items", chosen)
            ));
        }
    };
    update();
    controller.observe(move |_| update());
    bar
}
