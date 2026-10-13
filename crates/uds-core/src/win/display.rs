//! Monitors and the screen-wide Night Light (display gamma ramp).

use windows::Win32::Foundation::{LPARAM, RECT};
use windows::Win32::Graphics::Gdi::{
    CreateDCW, DEVMODEW, DISPLAY_DEVICEW, DeleteDC, ENUM_CURRENT_SETTINGS, EnumDisplayDevicesW, EnumDisplayMonitors,
    EnumDisplaySettingsW, GetMonitorInfoW, HDC, HMONITOR, MONITORINFO, MONITORINFOEXW,
};
use windows::Win32::UI::ColorSystem::SetDeviceGammaRamp;
use windows::Win32::UI::HiDpi::{GetDpiForMonitor, MDT_EFFECTIVE_DPI};
use windows::core::{BOOL, PCWSTR};

use super::wide;
use crate::nightlight::gamma_ramp;
use crate::system::DisplayInfo;
use crate::{Error, Result};

const MONITORINFOF_PRIMARY: u32 = 1;

fn utf16_to_string(buf: &[u16]) -> String {
    let end = buf.iter().position(|c| *c == 0).unwrap_or(buf.len());
    String::from_utf16_lossy(&buf[..end])
}

unsafe extern "system" fn collect(monitor: HMONITOR, _: HDC, _: *mut RECT, data: LPARAM) -> BOOL {
    unsafe {
        let list = &mut *(data.0 as *mut Vec<HMONITOR>);
        list.push(monitor);
    }
    BOOL(1)
}

fn monitors() -> Vec<HMONITOR> {
    let mut list: Vec<HMONITOR> = Vec::new();
    unsafe {
        let _ = EnumDisplayMonitors(None, None, Some(collect), LPARAM(&mut list as *mut _ as isize));
    }
    list
}

fn monitor_device(monitor: HMONITOR) -> Option<(String, MONITORINFOEXW)> {
    let mut info = MONITORINFOEXW::default();
    info.monitorInfo.cbSize = std::mem::size_of::<MONITORINFOEXW>() as u32;
    unsafe { GetMonitorInfoW(monitor, &mut info as *mut _ as *mut MONITORINFO) }
        .as_bool()
        .then(|| (utf16_to_string(&info.szDevice), info))
}

pub fn list() -> Vec<DisplayInfo> {
    let mut out = Vec::new();
    for (index, monitor) in monitors().into_iter().enumerate() {
        let Some((device, info)) = monitor_device(monitor) else { continue };
        let device_w = wide(&device);
        let mut mode = DEVMODEW { dmSize: std::mem::size_of::<DEVMODEW>() as u16, ..Default::default() };
        let has_mode =
            unsafe { EnumDisplaySettingsW(PCWSTR(device_w.as_ptr()), ENUM_CURRENT_SETTINGS, &mut mode) }.as_bool();
        let mut adapter = DISPLAY_DEVICEW { cb: std::mem::size_of::<DISPLAY_DEVICEW>() as u32, ..Default::default() };
        let mut name = format!("Display {}", index + 1);
        if unsafe { EnumDisplayDevicesW(PCWSTR(device_w.as_ptr()), 0, &mut adapter, 0) }.as_bool() {
            let monitor_name = utf16_to_string(&adapter.DeviceString);
            if !monitor_name.is_empty() {
                name = monitor_name;
            }
        }
        let (mut dpi_x, mut dpi_y) = (96u32, 96u32);
        let _ = unsafe { GetDpiForMonitor(monitor, MDT_EFFECTIVE_DPI, &mut dpi_x, &mut dpi_y) };
        let rect = info.monitorInfo.rcMonitor;
        out.push(DisplayInfo {
            name,
            device,
            width: if has_mode { mode.dmPelsWidth } else { (rect.right - rect.left) as u32 },
            height: if has_mode { mode.dmPelsHeight } else { (rect.bottom - rect.top) as u32 },
            refresh_hz: if has_mode { mode.dmDisplayFrequency } else { 0 },
            scale_percent: dpi_x * 100 / 96,
            primary: info.monitorInfo.dwFlags & MONITORINFOF_PRIMARY != 0,
        });
    }
    out.sort_by_key(|d| std::cmp::Reverse(d.primary));
    out
}

/// Applies the warm filter to every monitor. Strength 0 restores normal colors.
pub fn apply_night_light(strength: u8) -> Result<()> {
    let ramp = gamma_ramp(strength);
    let mut applied = 0;
    for monitor in monitors() {
        let Some((device, _)) = monitor_device(monitor) else { continue };
        let device_w = wide(&device);
        unsafe {
            let hdc = CreateDCW(PCWSTR::null(), PCWSTR(device_w.as_ptr()), PCWSTR::null(), None);
            if hdc.is_invalid() {
                continue;
            }
            if SetDeviceGammaRamp(hdc, ramp.as_ptr() as *const _).as_bool() {
                applied += 1;
            }
            let _ = DeleteDC(hdc);
        }
    }
    if applied == 0 && strength > 0 {
        return Err(Error::msg(
            "Windows didn't accept the color change on any display. Try a lower strength, or use Windows' own Night light.",
        ));
    }
    Ok(())
}
