//! Opening pages and programs, desktop refresh, setting-change broadcasts,
//! the Recycle Bin and notification beeps.

use std::path::Path;

use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::System::Diagnostics::Debug::MessageBeep;
use windows::Win32::UI::Shell::{
    SHCNE_ASSOCCHANGED, SHCNF_IDLIST, SHChangeNotify, SHERB_NOCONFIRMATION, SHERB_NOPROGRESSUI, SHERB_NOSOUND,
    SHEmptyRecycleBinW, SHQUERYRBINFO, SHQueryRecycleBinW, ShellExecuteW,
};
use windows::Win32::UI::WindowsAndMessaging::{
    HWND_BROADCAST, MB_ICONASTERISK, MB_ICONEXCLAMATION, MB_ICONHAND, MB_OK, SMTO_ABORTIFHUNG, SW_HIDE, SW_SHOWNORMAL,
    SendMessageTimeoutW, WM_SETTINGCHANGE,
};
use windows::core::{PCWSTR, w};

use super::wide;
use crate::alarms::Tone;
use crate::shortcuts::Target;
use crate::tweaks::Refresh;
use crate::{Error, Result};

pub(crate) fn shell_execute(verb: PCWSTR, file: &str, args: Option<&str>, hidden: bool) -> Result<()> {
    let file_w = wide(file);
    let args_w = args.map(wide);
    let result = unsafe {
        ShellExecuteW(
            None,
            verb,
            PCWSTR(file_w.as_ptr()),
            args_w.as_ref().map(|a| PCWSTR(a.as_ptr())).unwrap_or(PCWSTR::null()),
            PCWSTR::null(),
            if hidden { SW_HIDE } else { SW_SHOWNORMAL },
        )
    };
    // Values above 32 mean success.
    if result.0 as isize > 32 { Ok(()) } else { Err(Error::msg(format!("Windows couldn't open {file}."))) }
}

fn quote(arg: &str) -> String {
    format!("\"{}\"", arg.replace('"', ""))
}

pub fn open_target(target: Target) -> Result<()> {
    match target {
        Target::Uri(uri) => shell_execute(w!("open"), uri, None, false),
        Target::Program(exe, args) => {
            let joined = args
                .iter()
                .map(|a| if a.contains(' ') { quote(a) } else { (*a).to_owned() })
                .collect::<Vec<_>>()
                .join(" ");
            shell_execute(w!("open"), exe, (!joined.is_empty()).then_some(joined.as_str()), false)
        }
        Target::TempFolder => open_folder(&std::env::temp_dir()),
    }
}

pub fn open_folder(path: &Path) -> Result<()> {
    if !path.is_dir() {
        return Err(Error::NotFound(path.display().to_string()));
    }
    shell_execute(w!("open"), "explorer.exe", Some(&quote(&path.to_string_lossy())), false)
}

/// Starts a program the registry pointed us at (an installed app or browser).
pub fn launch(exe: &Path, args: &[&str]) -> Result<()> {
    if !exe.is_file() {
        return Err(Error::NotFound(exe.display().to_string()));
    }
    let joined = args.iter().map(|a| quote(a)).collect::<Vec<_>>().join(" ");
    shell_execute(w!("open"), &exe.to_string_lossy(), (!joined.is_empty()).then_some(joined.as_str()), false)
}

pub fn refresh_desktop() {
    unsafe { SHChangeNotify(SHCNE_ASSOCCHANGED, SHCNF_IDLIST, None, None) };
}

fn broadcast(area: Option<&str>) {
    let area_w = area.map(wide);
    let lparam = area_w.as_ref().map(|a| LPARAM(a.as_ptr() as isize)).unwrap_or(LPARAM(0));
    unsafe {
        let _ = SendMessageTimeoutW(HWND_BROADCAST, WM_SETTINGCHANGE, WPARAM(0), lparam, SMTO_ABORTIFHUNG, 1000, None);
    }
}

pub fn notify_changed(refresh: Refresh) {
    match refresh {
        Refresh::Theme => broadcast(Some("ImmersiveColorSet")),
        Refresh::Taskbar => broadcast(Some("TraySettings")),
        Refresh::Explorer => {
            broadcast(None);
            refresh_desktop();
        }
    }
}

pub fn recycle_bin() -> (u64, u64) {
    let mut info = SHQUERYRBINFO { cbSize: std::mem::size_of::<SHQUERYRBINFO>() as u32, ..Default::default() };
    match unsafe { SHQueryRecycleBinW(PCWSTR::null(), &mut info) } {
        Ok(()) => (info.i64NumItems.max(0) as u64, info.i64Size.max(0) as u64),
        Err(_) => (0, 0),
    }
}

pub fn empty_recycle_bin() -> Result<()> {
    unsafe {
        SHEmptyRecycleBinW(None::<HWND>, PCWSTR::null(), SHERB_NOCONFIRMATION | SHERB_NOPROGRESSUI | SHERB_NOSOUND)
    }
    .map_err(|e| Error::msg(format!("Could not empty the Recycle Bin: {}", e.message())))
}

pub fn beep(tone: Tone) {
    let style = match tone {
        Tone::Default => MB_OK,
        Tone::Information => MB_ICONASTERISK,
        Tone::Warning => MB_ICONEXCLAMATION,
        Tone::Critical => MB_ICONHAND,
    };
    unsafe {
        let _ = MessageBeep(style);
    }
}
