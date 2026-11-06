// SPDX-License-Identifier: MIT

//! Telling a sandbox output folder in use from one a closed process left behind,
//! where folders cannot be locked.

use std::path::Path;

/// Whether the process that named the folder `<prefix><pid>-<sequence>` has
/// ended. A name without a process id counts as still in use.
pub(crate) fn owner_has_exited(folder: &Path, prefix: &str) -> bool {
    let pid = folder
        .file_name()
        .and_then(|name| name.to_str())
        .and_then(|name| name.strip_prefix(prefix))
        .and_then(|rest| rest.split_once('-'))
        .and_then(|(pid, _)| pid.parse::<i32>().ok())
        .and_then(rustix::process::Pid::from_raw);
    pid.is_some_and(|pid| rustix::process::test_kill_process(pid) == Err(rustix::io::Errno::SRCH))
}

#[cfg(test)]
mod tests;
