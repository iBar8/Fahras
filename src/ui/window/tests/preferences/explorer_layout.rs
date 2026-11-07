// SPDX-License-Identifier: MIT

use super::*;
use crate::fahras::look::ExplorerSkin;
use crate::ui::preferences::ListColumnId;

fn descendants(root: &gtk::Widget) -> Vec<gtk::Widget> {
    let mut found = Vec::new();
    let mut pending = vec![root.clone()];
    while let Some(widget) = pending.pop() {
        let mut child = widget.first_child();
        while let Some(current) = child {
            child = current.next_sibling();
            pending.push(current);
        }
        found.push(widget);
    }
    found
}

fn with_class(open: &OpenWindow, class: &str) -> Vec<gtk::Widget> {
    descendants(open.window.upcast_ref())
        .into_iter()
        .filter(|widget| widget.has_css_class(class))
        .collect()
}

fn shown(open: &OpenWindow, class: &str) -> bool {
    with_class(open, class)
        .iter()
        .any(|widget| widget.is_visible() && widget.is_mapped())
}

/// A row's labels, leaving out the menus attached to it.
fn row_labels(widget: &gtk::Widget, labels: &mut Vec<String>) {
    if widget.is::<gtk::Popover>() {
        return;
    }
    if let Some(label) = widget.downcast_ref::<gtk::Label>() {
        labels.push(label.text().to_string());
    }
    let mut child = widget.first_child();
    while let Some(current) = child {
        row_labels(&current, labels);
        child = current.next_sibling();
    }
}

/// The row's name, not its one-letter key hint.
fn row_name(row: &gtk::Widget) -> Option<String> {
    let mut labels = Vec::new();
    row_labels(row, &mut labels);
    labels.into_iter().max_by_key(String::len)
}

/// The sidebar rows' names, top to bottom.
fn sidebar_names(open: &OpenWindow) -> Vec<String> {
    let mut names = Vec::new();
    let mut child = open.content.sidebar.state.widget.first_child();
    while let Some(row) = child {
        child = row.next_sibling();
        if row.has_css_class("sidebar-row")
            && row.is_visible()
            && let Some(name) = row_name(&row)
        {
            names.push(name);
        }
    }
    names
}

fn command_sensitive(open: &OpenWindow, name: &str) -> bool {
    with_class(open, "explorer-command")
        .into_iter()
        .filter(|widget| widget.is::<gtk::Button>() && widget.is_mapped())
        .find(|button| button.tooltip_text().as_deref() == Some(name))
        .is_some_and(|button| button.is_sensitive())
}

fn sidebar_width(open: &OpenWindow) -> i32 {
    with_class(open, "sidebar-scroll")
        .first()
        .map_or(0, |scroller| scroller.width_request())
}

/// The footer row's first widget, and whether its shortcuts button shows.
fn footer_start(open: &OpenWindow) -> (gtk::Widget, bool) {
    let status = with_class(open, "explorer-status")
        .into_iter()
        .next()
        .expect("status bar");
    let row = status.parent().expect("footer row");
    let shortcuts = std::iter::successors(row.first_child(), gtk::Widget::next_sibling)
        .find(|widget| {
            widget.is::<gtk::ToggleButton>() && widget.has_css_class("shortcut-footer-button")
        })
        .is_some_and(|button| button.get_visible());
    (row.first_child().expect("first"), shortcuts)
}

/// The icon the command bar's New shows, by name.
fn new_icon(open: &OpenWindow) -> Option<String> {
    with_class(open, "explorer-command-bar")
        .iter()
        .flat_map(descendants)
        .filter_map(|widget| widget.downcast::<gtk::MenuButton>().ok())
        .find(|button| row_name(button.upcast_ref()).as_deref() == Some("New"))
        .and_then(|button| button.child())
        .and_then(|content| content.first_child())
        .map(|icon| icon.widget_name().to_string())
}

fn separators(open: &OpenWindow) -> Vec<String> {
    with_class(open, "breadcrumb-separator")
        .iter()
        .filter_map(|widget| widget.downcast_ref::<gtk::Label>())
        .map(|label| label.text().to_string())
        .collect()
}

/// The address bar's crumbs, ancestors and the current folder.
fn crumb_names(open: &OpenWindow) -> Vec<String> {
    with_class(open, "breadcrumb")
        .iter()
        .filter(|widget| widget.is::<gtk::Button>() || widget.has_css_class("current"))
        .filter_map(row_name)
        .collect()
}

fn status_text(open: &OpenWindow) -> Vec<String> {
    with_class(open, "explorer-status")
        .iter()
        .flat_map(descendants)
        .filter_map(|widget| widget.downcast::<gtk::Label>().ok())
        .filter(|label| label.is_visible())
        .map(|label| label.text().to_string())
        .collect()
}

#[test]
fn explorer_looks_swap_the_window_layout_live() {
    gtk_test(
        "ui::window::tests::preferences::explorer_layout::explorer_looks_swap_the_window_layout_live",
        || {
            let manager = PreferenceManager::shared();
            manager.set_explorer_skin(ExplorerSkin::Off);
            manager.set_browser_mode(BrowserMode::Columns);
            let open = OpenWindow::open();
            let directory = load_folder(&open);
            let location = open.content.browser.location_widget();
            assert!(shown(&open, "browser-header"));
            assert!(!shown(&open, "explorer-chrome"));
            assert!(
                open.window.titlebar().is_some(),
                "Fahras draws its own header"
            );

            manager.set_explorer_skin(ExplorerSkin::Windows11);
            settle();
            assert_eq!(manager.browser_mode(), BrowserMode::List, "Details view");
            assert_eq!(
                manager.list_visible_columns(),
                [
                    ListColumnId::Name,
                    ListColumnId::Modified,
                    ListColumnId::Type,
                    ListColumnId::Size
                ]
            );
            assert!(!shown(&open, "browser-header"));
            assert!(shown(&open, "explorer-command-bar"));
            assert!(!shown(&open, "explorer-ribbon"));
            wait_until_or(
                || {
                    let names = sidebar_names(&open);
                    names.first().map(String::as_str) == Some("Home")
                        && names.ends_with(&["Network".to_owned(), "Trash".to_owned()])
                },
                || format!("{:?}", sidebar_names(&open)),
            );
            assert!(
                location
                    .parent()
                    .is_some_and(|slot| slot.has_css_class("explorer-address"))
            );
            assert!(
                open.window.titlebar().is_some(),
                "a shown window keeps its frame"
            );
            assert!(
                shown(&open, "explorer-window-controls"),
                "it keeps minimize, maximize and close"
            );
            let reopened = OpenWindow::open();
            assert!(reopened.window.titlebar().is_none(), "the system title bar");
            assert!(!shown(&reopened, "explorer-window-controls"));
            manager.set_explorer_skin(ExplorerSkin::Off);
            settle();
            assert!(shown(&reopened, "browser-header"));
            assert!(
                !with_class(&reopened, "fahras-window-button")
                    .iter()
                    .any(gtk::Widget::is_visible),
                "the system title bar keeps the only window buttons"
            );
            assert!(shown(&open, "fahras-window-button"));
            manager.set_explorer_skin(ExplorerSkin::Windows11);
            settle();
            reopened.window.destroy();
            assert!(
                open.content.footer().widget().parent().is_some(),
                "the footer keeps job progress and prompts"
            );
            let view = with_class(&open, "explorer-command-bar")
                .iter()
                .flat_map(descendants)
                .filter_map(|widget| widget.downcast::<gtk::MenuButton>().ok())
                .find(|button| row_name(button.upcast_ref()).as_deref() == Some("View"))
                .expect("view command");
            assert_eq!(
                view.tooltip_text(),
                None,
                "a labelled command has no tooltip"
            );
            let search = with_class(&open, "explorer-search")
                .into_iter()
                .find_map(|widget| widget.downcast::<gtk::SearchEntry>().ok())
                .expect("search box");
            search.grab_focus();
            search.set_text("a");
            settle_for(std::time::Duration::from_millis(400));
            search.set_text("");
            settle_for(std::time::Duration::from_millis(400));
            let search_widget: gtk::Widget = search.clone().upcast();
            assert!(
                gtk::prelude::GtkWindowExt::focus(&open.window).is_some_and(|focus| focus
                    == search_widget
                    || focus.is_ancestor(&search_widget)),
                "clearing the search keeps focus in the box"
            );
            let browser = open.content.browser.browser();
            browser.clear_active_selection();
            wait_until_or(
                || status_text(&open) == ["2 items"],
                || format!("{:?}", status_text(&open)),
            );
            browser.select(0, 0);
            wait_until_or(
                || status_text(&open).contains(&"1 item selected".to_owned()),
                || format!("{:?}", status_text(&open)),
            );

            let up = with_class(&open, "explorer-nav")
                .into_iter()
                .filter_map(|widget| widget.downcast::<gtk::Button>().ok())
                .find(|button| button.tooltip_text().as_deref() == Some("Up"))
                .expect("up button");
            up.emit_clicked();
            let parent = directory
                .path()
                .parent()
                .expect("parent folder")
                .to_path_buf();
            wait_until(|| {
                open.content.browser.browser().active_location()
                    == Some(crate::model::Location::local(&parent))
            });

            manager.set_explorer_skin(ExplorerSkin::Windows10);
            settle();
            assert!(shown(&open, "explorer-ribbon"));
            assert!(!shown(&open, "explorer-command-bar"));
            manager.set_explorer_skin(ExplorerSkin::Windows11Legacy);
            settle();
            assert!(
                shown(&open, "explorer-command-bar"),
                "legacy keeps the layout"
            );
            assert!(!crate::fahras::explorer_details::is_polished());
            assert!(!shown(&open, "explorer-details-pane"));
            assert!(!shown(&open, "explorer-view-switches"));
            assert_eq!(
                new_icon(&open).as_deref(),
                Some("fahras-fluent-new-symbolic")
            );
            open.content.browser.browser().clear_active_selection();
            wait_until(|| command_sensitive(&open, "Copy"));
            assert_eq!(sidebar_width(&open), 201, "legacy keeps Fahras's sidebar");
            let (first, _) = footer_start(&open);
            assert!(
                !first.has_css_class("explorer-status"),
                "legacy keeps Fahras's footer"
            );
            wait_until_or(
                || !sidebar_names(&open).contains(&"Gallery".to_owned()),
                || format!("{:?}", sidebar_names(&open)),
            );
            assert!(!shown(&open, "explorer-pin"));
            assert!(!shown(&open, "explorer-tree-row"));
            assert!(!shown(&open, "explorer-this-pc"));
            wait_until_or(
                || separators(&open).iter().all(|text| text == "/"),
                || format!("{:?}", separators(&open)),
            );
            manager.set_explorer_skin(ExplorerSkin::Windows11);
            settle();
            assert!(crate::fahras::explorer_details::is_polished());
            assert!(shown(&open, "explorer-details-pane"));
            assert!(shown(&open, "explorer-view-switches"));
            assert_eq!(new_icon(&open).as_deref(), Some("fahras-fluent-new-color"));
            let browser = open.content.browser.browser();
            browser.clear_active_selection();
            wait_until(|| !command_sensitive(&open, "Copy"));
            assert!(!command_sensitive(&open, "Delete"), "nothing to act on");
            browser.select(0, 0);
            wait_until(|| command_sensitive(&open, "Copy"));
            browser.clear_active_selection();
            assert_eq!(
                sidebar_width(&open),
                158,
                "Explorer's navigation pane width"
            );
            let (first, shortcuts_shown) = footer_start(&open);
            assert!(
                first.has_css_class("explorer-status"),
                "the count comes first"
            );
            assert!(!shortcuts_shown, "no shortcuts button, as in Explorer");
            // Gallery shows the Pictures folder, so it needs one.
            let has_pictures = glib::user_special_dir(glib::UserDirectory::Pictures).is_some();
            wait_until_or(
                || {
                    let names = sidebar_names(&open);
                    let gallery_second = names.get(1).map(String::as_str) == Some("Gallery");
                    gallery_second == has_pictures && names.iter().any(|name| name == "Home")
                },
                || format!("{:?}", sidebar_names(&open)),
            );
            assert!(shown(&open, "explorer-pin"), "user folders are pinned");
            assert!(
                shown(&open, "explorer-tree-row"),
                "Network is a tree row beside This PC"
            );
            assert!(
                with_class(&open, "sidebar-heading-toggle")
                    .iter()
                    .filter(|heading| row_name(heading).as_deref() == Some("This PC"))
                    .all(|heading| heading.has_css_class("explorer-this-pc")),
                "drives sit under a This PC tree row"
            );
            let switch = |name: &str| {
                with_class(&open, "explorer-view-switch")
                    .into_iter()
                    .filter_map(|widget| widget.downcast::<gtk::ToggleButton>().ok())
                    .find(|button| button.tooltip_text().as_deref() == Some(name))
                    .expect("view switch")
            };
            let (details, large_icons) = (switch("Details"), switch("Large icons"));
            assert!(details.is_active(), "Details view is on");
            large_icons.emit_clicked();
            settle();
            assert_eq!(manager.browser_mode(), BrowserMode::Icons);
            assert!(large_icons.is_active() && !details.is_active());
            details.emit_clicked();
            settle();
            assert_eq!(manager.browser_mode(), BrowserMode::List);
            let child = directory.path().join("child");
            open.content
                .browser
                .navigate_location(crate::model::Location::local(&child));
            wait_until_or(
                || {
                    let separators = separators(&open);
                    !separators.is_empty() && separators.iter().all(|text| text == "›")
                },
                || format!("{:?}", separators(&open)),
            );
            assert_eq!(
                crumb_names(&open).len(),
                child.components().count(),
                "the whole path, not from the home folder: {:?}",
                crumb_names(&open)
            );

            manager.set_explorer_skin(ExplorerSkin::Off);
            settle();
            assert!(shown(&open, "browser-header"));
            assert!(!shown(&open, "explorer-chrome"));
            assert!(location.ancestor(gtk::HeaderBar::static_type()).is_some());
            assert!(open.window.titlebar().is_some());
            let reopened = OpenWindow::open();
            assert!(
                reopened.window.titlebar().is_some(),
                "Fahras's header again"
            );
            reopened.window.destroy();
            assert!(open.content.footer().widget().parent().is_some());
            open.window.destroy();
        },
    );
}
