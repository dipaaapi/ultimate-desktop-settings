//! Running processes. Apps are closed by asking their windows to close
//! (like clicking X), never by force-killing a guessed process name.

use std::collections::HashSet;

use windows::Win32::Foundation::{CloseHandle, HWND, LPARAM, WPARAM};
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW, TH32CS_SNAPPROCESS,
};
use windows::Win32::System::Threading::{
    OpenProcess, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION, QueryFullProcessImageNameW,
};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GW_OWNER, GetWindow, GetWindowThreadProcessId, IsWindowVisible, PostMessageW, WM_CLOSE,
};
use windows::core::{BOOL, PWSTR};

pub fn process_image_path(pid: u32) -> Option<String> {
    if pid == 0 {
        return None;
    }
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let mut buf = vec![0u16; 1024];
        let mut len = buf.len() as u32;
        let ok = QueryFullProcessImageNameW(handle, PROCESS_NAME_WIN32, PWSTR(buf.as_mut_ptr()), &mut len).is_ok();
        let _ = CloseHandle(handle);
        ok.then(|| String::from_utf16_lossy(&buf[..len as usize]))
    }
}

/// `(pid, full image path)` for every process this user can inspect.
pub fn running() -> Vec<(u32, String)> {
    let mut out = Vec::new();
    unsafe {
        let Ok(snapshot) = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) else { return out };
        let mut entry = PROCESSENTRY32W { dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32, ..Default::default() };
        let mut has = Process32FirstW(snapshot, &mut entry).is_ok();
        while has {
            if let Some(path) = process_image_path(entry.th32ProcessID) {
                out.push((entry.th32ProcessID, path));
            }
            has = Process32NextW(snapshot, &mut entry).is_ok();
        }
        let _ = CloseHandle(snapshot);
    }
    out
}

struct CloseRequest {
    pids: HashSet<u32>,
    closed: usize,
}

unsafe extern "system" fn close_window(hwnd: HWND, data: LPARAM) -> BOOL {
    unsafe {
        let request = &mut *(data.0 as *mut CloseRequest);
        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        let is_top_level = GetWindow(hwnd, GW_OWNER).map(|owner| owner.is_invalid()).unwrap_or(true);
        if request.pids.contains(&pid)
            && IsWindowVisible(hwnd).as_bool()
            && is_top_level
            && PostMessageW(Some(hwnd), WM_CLOSE, WPARAM(0), LPARAM(0)).is_ok()
        {
            request.closed += 1;
        }
    }
    BOOL(1)
}

/// Asks the visible windows of these processes to close. Returns how many
/// windows were asked; apps may still show their own "save changes?" prompt.
pub fn close_windows(pids: &[u32]) -> usize {
    let mut request = CloseRequest { pids: pids.iter().copied().collect(), closed: 0 };
    unsafe {
        let _ = EnumWindows(Some(close_window), LPARAM(&mut request as *mut _ as isize));
    }
    request.closed
}

#[cfg(test)]
mod tests {
    #[test]
    fn finds_own_process() {
        let me = std::process::id();
        let path = super::process_image_path(me).expect("own image path");
        assert!(path.to_lowercase().ends_with(".exe"));
        assert!(super::running().iter().any(|(pid, _)| *pid == me));
    }
}
