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

#[cfg(test)]
mod tests;
