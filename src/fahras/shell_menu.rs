// SPDX-License-Identifier: MIT

//! What the Windows shell's own right-click menu can be shown for.

use std::path::PathBuf;

/// The shell menu binds to one folder, so files from several, such as search
/// results, keep Fahras's own menu.
#[cfg_attr(
    all(not(windows), not(test)),
    expect(dead_code, reason = "only Windows shows the shell menu")
)]
pub(crate) fn share_one_folder(paths: &[PathBuf]) -> bool {
    let mut parents = paths.iter().map(|path| path.parent());
    match parents.next() {
        Some(Some(first)) => parents.all(|parent| parent == Some(first)),
        _ => false,
    }
}

/// `(x, y)` in `widget` as a point in its window's client area, in device
/// pixels, where Windows places a menu opened from the keyboard.
#[cfg_attr(
    all(not(windows), not(test)),
    expect(dead_code, reason = "only Windows shows the shell menu")
)]
pub(crate) fn window_point(widget: &gtk::Widget, x: f64, y: f64) -> Option<(i32, i32)> {
    use gtk::prelude::*;
    let root = widget.root()?;
    let point = widget.compute_point(&root, &gtk::graphene::Point::new(x as f32, y as f32))?;
    let scale = f64::from(widget.scale_factor());
    Some((
        (f64::from(point.x()) * scale).round() as i32,
        (f64::from(point.y()) * scale).round() as i32,
    ))
}

#[cfg(test)]
mod tests;
