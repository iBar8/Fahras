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
                        && names.last().map(String::as_str) == Some("Network")
                },
                || format!("{:?}", sidebar_names(&open)),
            );
            assert!(
                location
                    .parent()
                    .is_some_and(|slot| slot.has_css_class("explorer-address"))
            );
            assert!(open.window.titlebar().is_none(), "the system title bar");
            assert!(
                open.content.footer().widget().parent().is_none(),
                "Fahras's footer is out"
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

            manager.set_explorer_skin(ExplorerSkin::Off);
            settle();
            assert!(shown(&open, "browser-header"));
            assert!(!shown(&open, "explorer-chrome"));
            assert!(location.ancestor(gtk::HeaderBar::static_type()).is_some());
            assert!(open.window.titlebar().is_some());
            assert!(open.content.footer().widget().parent().is_some());
            open.window.destroy();
        },
    );
}
