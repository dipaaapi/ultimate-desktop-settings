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
        Refresh::None | Refresh::ExplorerRestart => {}
        Refresh::International => broadcast(Some("intl")),
        Refresh::Wallpaper => {
            if let Some(current) = wallpaper() {
                let _ = set_wallpaper(Path::new(&current));
            }
        }
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

/// Runs this app again with `args` through the UAC prompt and waits for it.
/// Used for the few changes that need admin rights.
pub fn run_self_elevated(args: &str) -> Result<()> {
    use windows::Win32::Foundation::{CloseHandle, ERROR_CANCELLED, WAIT_OBJECT_0};
    use windows::Win32::System::Threading::{GetExitCodeProcess, INFINITE, WaitForSingleObject};
    use windows::Win32::UI::Shell::{SEE_MASK_NOASYNC, SEE_MASK_NOCLOSEPROCESS, SHELLEXECUTEINFOW, ShellExecuteExW};

    let exe = std::env::current_exe()?;
    let exe_w = wide(&exe.to_string_lossy());
    let args_w = wide(args);
    let mut info = SHELLEXECUTEINFOW {
        cbSize: std::mem::size_of::<SHELLEXECUTEINFOW>() as u32,
        fMask: SEE_MASK_NOCLOSEPROCESS | SEE_MASK_NOASYNC,
        lpVerb: w!("runas"),
        lpFile: PCWSTR(exe_w.as_ptr()),
        lpParameters: PCWSTR(args_w.as_ptr()),
        nShow: SW_HIDE.0,
        ..Default::default()
    };
    if let Err(e) = unsafe { ShellExecuteExW(&mut info) } {
        return Err(if e.code() == ERROR_CANCELLED.to_hresult() {
            Error::msg("The change was cancelled at the admin prompt.")
        } else {
            Error::msg(format!("Windows didn't allow the change: {}", e.message()))
        });
    }
    if info.hProcess.is_invalid() {
        return Err(Error::msg("Windows didn't start the admin helper."));
    }
    let mut code = 1u32;
    unsafe {
        if WaitForSingleObject(info.hProcess, INFINITE) == WAIT_OBJECT_0 {
            let _ = GetExitCodeProcess(info.hProcess, &mut code);
        }
        let _ = CloseHandle(info.hProcess);
    }
    if code == 0 { Ok(()) } else { Err(Error::msg("Windows didn't allow the change, even with admin rights.")) }
}

/// Wallpaper: the current image path, and setting a new one.
pub fn wallpaper() -> Option<String> {
    use windows::Win32::UI::WindowsAndMessaging::{
        SPI_GETDESKWALLPAPER, SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS, SystemParametersInfoW,
    };
    let mut buf = [0u16; 1024];
    unsafe {
        SystemParametersInfoW(
            SPI_GETDESKWALLPAPER,
            buf.len() as u32,
            Some(buf.as_mut_ptr().cast()),
            SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
        )
        .ok()?;
    }
    let end = buf.iter().position(|c| *c == 0).unwrap_or(buf.len());
    let path = String::from_utf16_lossy(&buf[..end]);
    (!path.is_empty()).then_some(path)
}

pub fn set_wallpaper(path: &Path) -> Result<()> {
    use windows::Win32::UI::WindowsAndMessaging::{
        SPI_SETDESKWALLPAPER, SPIF_SENDCHANGE, SPIF_UPDATEINIFILE, SystemParametersInfoW,
    };
    if !path.is_file() {
        return Err(Error::NotFound(path.display().to_string()));
    }
    let mut path_w = wide(&path.to_string_lossy());
    unsafe {
        SystemParametersInfoW(
            SPI_SETDESKWALLPAPER,
            0,
            Some(path_w.as_mut_ptr().cast()),
            SPIF_UPDATEINIFILE | SPIF_SENDCHANGE,
        )
    }
    .map_err(|e| Error::msg(format!("Could not set the wallpaper: {}", e.message())))
}

/// Taskbar auto-hide, read and set live through the app bar API.
pub fn taskbar_autohide() -> bool {
    use windows::Win32::UI::Shell::{ABM_GETSTATE, ABS_AUTOHIDE, APPBARDATA, SHAppBarMessage};
    let mut data = APPBARDATA { cbSize: std::mem::size_of::<APPBARDATA>() as u32, ..Default::default() };
    let state = unsafe { SHAppBarMessage(ABM_GETSTATE, &mut data) } as u32;
    state & ABS_AUTOHIDE != 0
}

pub fn set_taskbar_autohide(enabled: bool) {
    use windows::Win32::UI::Shell::{ABM_SETSTATE, ABS_ALWAYSONTOP, ABS_AUTOHIDE, APPBARDATA, SHAppBarMessage};
    use windows::Win32::UI::WindowsAndMessaging::FindWindowW;
    let tray = unsafe { FindWindowW(w!("Shell_TrayWnd"), PCWSTR::null()) }.unwrap_or_default();
    let mut data = APPBARDATA {
        cbSize: std::mem::size_of::<APPBARDATA>() as u32,
        hWnd: tray,
        lParam: LPARAM((if enabled { ABS_AUTOHIDE } else { ABS_ALWAYSONTOP }) as isize),
        ..Default::default()
    };
    unsafe { SHAppBarMessage(ABM_SETSTATE, &mut data) };
}

/// Restarts Explorer gracefully so settings it only reads at start apply.
/// Explorer is asked to exit the way it does on sign-out, then started again.
pub fn restart_explorer() -> Result<()> {
    use windows::Win32::UI::WindowsAndMessaging::{FindWindowW, PostMessageW, WM_USER};
    let tray = unsafe { FindWindowW(w!("Shell_TrayWnd"), PCWSTR::null()) }
        .map_err(|_| Error::msg("Explorer isn't running."))?;
    // 0x5B4 is the documented-by-convention "exit Explorer" tray message.
    unsafe { PostMessageW(Some(tray), WM_USER + 436, WPARAM(0), LPARAM(0)) }
        .map_err(|e| Error::msg(format!("Could not restart Explorer: {}", e.message())))?;
    // Wait for the taskbar to go away, then start Explorer again.
    for _ in 0..50 {
        std::thread::sleep(std::time::Duration::from_millis(100));
        if unsafe { FindWindowW(w!("Shell_TrayWnd"), PCWSTR::null()) }.is_err() {
            break;
        }
    }
    std::thread::sleep(std::time::Duration::from_millis(500));
    shell_execute(w!("open"), "explorer.exe", None, false)
}
