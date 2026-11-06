// SPDX-License-Identifier: MIT

//! Fahras in the Windows shell: the native right-click menu for files, with
//! the entries other programs add, and opening folders in Fahras by default.

#![expect(unsafe_code, reason = "the Windows shell is a COM and Win32 API")]

use std::{ffi::OsStr, io, os::windows::ffi::OsStrExt, path::PathBuf};

use windows::{
    Win32::{
        Foundation::{HWND, POINT},
        System::Com::{COINIT_APARTMENTTHREADED, CoInitializeEx, CoTaskMemFree},
        UI::{
            Shell::{
                CMF_EXPLORE, CMF_NORMAL, CMIC_MASK_PTINVOKE, CMINVOKECOMMANDINFO,
                CMINVOKECOMMANDINFOEX, Common::ITEMIDLIST, IContextMenu, ILFindLastID,
                IShellFolder, SHBindToParent, SHParseDisplayName,
            },
            WindowsAndMessaging::{
                CreatePopupMenu, DestroyMenu, GetCursorPos, GetForegroundWindow, SW_SHOWNORMAL,
                SetForegroundWindow, TPM_RETURNCMD, TPM_RIGHTBUTTON, TrackPopupMenuEx,
            },
        },
    },
    core::{HSTRING, PCSTR, PCWSTR},
};
use windows_sys::Win32::{
    Foundation::ERROR_SUCCESS,
    System::Registry::{
        HKEY_CURRENT_USER, REG_SZ, RRF_RT_REG_SZ, RegDeleteKeyValueW, RegGetValueW, RegSetKeyValueW,
    },
};

/// Unicode verbs and paths in `CMINVOKECOMMANDINFOEX`; missing from the bindings.
const CMIC_MASK_UNICODE: u32 = 0x4000;
/// The verb the installer and the default-folder setting add.
const VERB: &str = "Fahras";
/// Folders and drives: the shell classes Explorer opens by their default verb.
const FOLDER_CLASSES: [&str; 2] = [r"Software\Classes\Directory", r"Software\Classes\Drive"];

/// Shows the shell's own menu for `paths`, which share one folder, at the
/// pointer, and runs the command chosen from it.
pub(crate) fn show_context_menu(paths: &[PathBuf]) -> bool {
    match context_menu(paths) {
        Ok(()) => true,
        Err(error) => {
            tracing::warn!(%error, "could not show the Windows context menu");
            false
        }
    }
}

struct Pidls(Vec<*mut ITEMIDLIST>);

impl Drop for Pidls {
    fn drop(&mut self) {
        for pidl in &self.0 {
            unsafe { CoTaskMemFree(Some(pidl.cast_const().cast())) };
        }
    }
}

fn context_menu(paths: &[PathBuf]) -> windows::core::Result<()> {
    unsafe {
        // GTK has already set up COM on this thread; a second call only counts it.
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        let mut pidls = Pidls(Vec::with_capacity(paths.len()));
        for path in paths {
            let mut pidl = std::ptr::null_mut();
            SHParseDisplayName(&HSTRING::from(path.as_os_str()), None, &mut pidl, 0, None)?;
            pidls.0.push(pidl);
        }
        let Some(&first) = pidls.0.first() else {
            return Ok(());
        };
        let folder: IShellFolder = SHBindToParent(first, None)?;
        let children: Vec<*const ITEMIDLIST> = pidls
            .0
            .iter()
            .map(|pidl| ILFindLastID(*pidl).cast_const())
            .collect();
        let window: HWND = GetForegroundWindow();
        let menu: IContextMenu = folder.GetUIObjectOf(window, &children, None)?;
        let popup = CreatePopupMenu()?;
        let shown = (|| {
            menu.QueryContextMenu(popup, 0, 1, 0x7fff, CMF_NORMAL | CMF_EXPLORE)
                .ok()?;
            let mut point = POINT::default();
            GetCursorPos(&mut point)?;
            let _ = SetForegroundWindow(window);
            let chosen = TrackPopupMenuEx(
                popup,
                (TPM_RETURNCMD | TPM_RIGHTBUTTON).0,
                point.x,
                point.y,
                window,
                None,
            );
            if chosen.0 > 0 {
                let offset = (chosen.0 - 1) as usize;
                let info = CMINVOKECOMMANDINFOEX {
                    cbSize: size_of::<CMINVOKECOMMANDINFOEX>() as u32,
                    fMask: CMIC_MASK_UNICODE | CMIC_MASK_PTINVOKE,
                    hwnd: window,
                    lpVerb: PCSTR(offset as *const u8),
                    lpVerbW: PCWSTR(offset as *const u16),
                    nShow: SW_SHOWNORMAL.0,
                    ptInvoke: point,
                    ..Default::default()
                };
                menu.InvokeCommand(std::ptr::from_ref(&info).cast::<CMINVOKECOMMANDINFO>())?;
            }
            Ok(())
        })();
        let _ = DestroyMenu(popup);
        shown
    }
}

fn wide(text: impl AsRef<OsStr>) -> Vec<u16> {
    text.as_ref().encode_wide().chain([0]).collect()
}

fn registry_error(status: u32) -> io::Result<()> {
    if status == ERROR_SUCCESS {
        Ok(())
    } else {
        Err(io::Error::from_raw_os_error(status as i32))
    }
}

fn set_string(key: &str, name: Option<&str>, value: &str) -> io::Result<()> {
    let key = wide(key);
    let name = name.map(wide);
    let value = wide(value);
    let status = unsafe {
        RegSetKeyValueW(
            HKEY_CURRENT_USER,
            key.as_ptr(),
            name.as_ref().map_or(std::ptr::null(), |name| name.as_ptr()),
            REG_SZ,
            value.as_ptr().cast(),
            (value.len() * size_of::<u16>()) as u32,
        )
    };
    registry_error(status)
}

fn string(key: &str) -> Option<String> {
    let key = wide(key);
    let mut buffer = vec![0_u16; 512];
    let mut size = (buffer.len() * size_of::<u16>()) as u32;
    let status = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            key.as_ptr(),
            std::ptr::null(),
            RRF_RT_REG_SZ,
            std::ptr::null_mut(),
            buffer.as_mut_ptr().cast(),
            &mut size,
        )
    };
    if status != ERROR_SUCCESS {
        return None;
    }
    let length = (size as usize / size_of::<u16>()).saturating_sub(1);
    Some(String::from_utf16_lossy(&buffer[..length]))
}

/// Whether double-clicking a folder in Windows opens it in Fahras.
pub(crate) fn opens_folders() -> bool {
    FOLDER_CLASSES
        .iter()
        .all(|class| string(&format!(r"{class}\shell")).as_deref() == Some(VERB))
}

/// Makes Fahras the program that opens folders and drives, or gives them back
/// to File Explorer. Only this user's settings change.
pub(crate) fn set_opens_folders(enabled: bool) -> io::Result<()> {
    let executable = std::env::current_exe()?;
    for class in FOLDER_CLASSES {
        let shell = format!(r"{class}\shell");
        if enabled {
            let verb = format!(r"{shell}\{VERB}");
            set_string(&verb, None, &crate::i18n::tr("Open in Fahras"))?;
            set_string(&verb, Some("Icon"), &executable.display().to_string())?;
            set_string(
                &format!(r"{verb}\command"),
                None,
                &format!("\"{}\" \"%1\"", executable.display()),
            )?;
            set_string(&shell, None, VERB)?;
        } else if string(&shell).as_deref() == Some(VERB) {
            let shell = wide(&shell);
            registry_error(unsafe {
                RegDeleteKeyValueW(HKEY_CURRENT_USER, shell.as_ptr(), std::ptr::null())
            })?;
        }
    }
    Ok(())
}

/// With an Explorer look on, right-clicking files shows the Windows menu in
/// place of Fahras's own, as File Explorer does.
pub(crate) fn replaces_item_menu(entries: &[crate::model::FileEntry]) -> bool {
    if crate::ui::preferences::PreferenceManager::shared().explorer_skin()
        == crate::fahras::look::ExplorerSkin::Off
    {
        return false;
    }
    let paths: Vec<PathBuf> = entries
        .iter()
        .filter_map(|entry| {
            entry
                .location
                .native_path()
                .map(std::path::Path::to_path_buf)
        })
        .collect();
    !paths.is_empty() && paths.len() == entries.len() && show_context_menu(&paths)
}
