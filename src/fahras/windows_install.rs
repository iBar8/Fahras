// SPDX-License-Identifier: MIT

//! Installs a Fahras update on Windows by running the signed installer, which
//! closes Fahras, replaces it and opens it again.

use std::{process::Command, sync::mpsc::Sender};

use super::{
    InstallCancel, InstallRequest, InstallStop, UpdateInstall, download_to_file_bounded,
    fetch_signed_release, installed_executable,
};

const UNINSTALLER: &str = "unins000.exe";
const STAGING_PREFIX: &str = "fahras-update-";
const INSTALLER_ARGUMENTS: [&str; 7] = [
    "/SILENT",
    "/SP-",
    "/SUPPRESSMSGBOXES",
    "/NORESTART",
    "/CLOSEAPPLICATIONS",
    "/FORCECLOSEAPPLICATIONS",
    "/RELAUNCH=1",
];

pub(super) fn install(
    request: &InstallRequest,
    download_url: &str,
    cancel: &InstallCancel,
    progress: &Sender<UpdateInstall>,
) -> Result<(), InstallStop> {
    ensure_installed_by_setup()?;
    remove_earlier_downloads();
    let _sent = progress.send(UpdateInstall::Verifying);
    let release = fetch_signed_release(request, cancel)?;
    let workdir = tempfile::Builder::new()
        .prefix(STAGING_PREFIX)
        .tempdir()
        .map_err(|error| {
            rust_i18n::t!(
                "Could not stage the update: %{error}",
                error = crate::services::io_error_detail(&error)
            )
            .into_owned()
        })?;
    let setup = workdir.path().join(&request.asset_name);
    download_to_file_bounded(
        download_url,
        &setup,
        release.archive_size(),
        cancel,
        progress,
    )?;
    let _sent = progress.send(UpdateInstall::Verifying);
    release.verify_archive(&setup, cancel)?;

    let _sent = progress.send(UpdateInstall::Installing);
    if !cancel.try_commit() {
        return Err(InstallStop::Cancelled);
    }
    let _sent = progress.send(UpdateInstall::Finalizing);
    Command::new(&setup)
        .args(INSTALLER_ARGUMENTS)
        .spawn()
        .map_err(|error| {
            rust_i18n::t!(
                "fahras.update.start_failed",
                error = crate::services::io_error_detail(&error)
            )
            .into_owned()
        })?;
    // The installer runs from this directory; the next update removes it.
    let _kept = workdir.keep();
    Ok(())
}

/// A copy unpacked from the zip has no uninstaller, and running the installer
/// would create a second copy instead of updating it.
fn ensure_installed_by_setup() -> Result<(), String> {
    let installed = installed_executable()
        .ok()
        .and_then(|executable| Some(executable.parent()?.parent()?.join(UNINSTALLER)))
        .is_some_and(|uninstaller| uninstaller.is_file());
    if installed {
        Ok(())
    } else {
        Err(crate::i18n::tr("fahras.update.portable"))
    }
}

fn remove_earlier_downloads() {
    let Ok(entries) = std::fs::read_dir(std::env::temp_dir()) else {
        return;
    };
    for entry in entries.flatten() {
        if entry
            .file_name()
            .to_string_lossy()
            .starts_with(STAGING_PREFIX)
        {
            let _removed = std::fs::remove_dir_all(entry.path());
        }
    }
}
