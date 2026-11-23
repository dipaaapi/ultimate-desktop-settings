//! Power plans and their settings (screen, sleep, lid, buttons, battery
//! saver, brightness), through the same power API Control Panel uses.

use windows::Win32::Foundation::{ERROR_SUCCESS, HLOCAL, LocalFree, WIN32_ERROR};
use windows::Win32::System::Power::{
    ACCESS_SCHEME, GetSystemPowerStatus, PowerEnumerate, PowerGetActiveScheme, PowerReadACValueIndex,
    PowerReadDCValueIndex, PowerReadFriendlyName, PowerSetActiveScheme, PowerWriteACValueIndex, PowerWriteDCValueIndex,
    SYSTEM_POWER_STATUS,
};
use windows::core::GUID;

use crate::system::{BatteryStatus, PowerPlan};
use crate::{Error, Result};

const SUB_VIDEO: GUID = GUID::from_u128(0x7516b95f_f776_4464_8c53_06167f40cc99);
const VIDEO_IDLE: GUID = GUID::from_u128(0x3c0bc021_c8a8_4e07_a973_6b14cbcb2b7e);
const VIDEO_BRIGHTNESS: GUID = GUID::from_u128(0xaded5e82_b909_4619_9949_f5d71dac0bcb);
const SUB_SLEEP: GUID = GUID::from_u128(0x238c9fa8_0aad_41ed_83f4_97be242c8f20);
const STANDBY_IDLE: GUID = GUID::from_u128(0x29f6c1db_86da_48c5_9fdb_f2b67b1f44da);
const SUB_BUTTONS: GUID = GUID::from_u128(0x4f971e89_eebd_4455_a8de_9e59040e7347);
const LID_ACTION: GUID = GUID::from_u128(0x5ca83367_6e45_459f_a27b_476b1d01c936);
const POWER_BUTTON: GUID = GUID::from_u128(0x7648efa3_dd9c_4e3e_b566_50f929386280);
const SUB_ENERGY_SAVER: GUID = GUID::from_u128(0xde830923_a562_41af_a086_e3a2c6bad2da);
const SAVER_THRESHOLD: GUID = GUID::from_u128(0xe69653ca_cf7f_4f05_aa73_cb833fa90ad4);

#[derive(Clone, Copy, PartialEq)]
enum Source {
    Plugged,
    Battery,
    /// Whichever the PC is on right now.
    Current,
}

fn setting(id: &str) -> Option<(GUID, GUID, Source)> {
    Some(match id {
        "brightness" => (SUB_VIDEO, VIDEO_BRIGHTNESS, Source::Current),
        "screen-off-ac" => (SUB_VIDEO, VIDEO_IDLE, Source::Plugged),
        "screen-off-dc" => (SUB_VIDEO, VIDEO_IDLE, Source::Battery),
        "sleep-ac" => (SUB_SLEEP, STANDBY_IDLE, Source::Plugged),
        "sleep-dc" => (SUB_SLEEP, STANDBY_IDLE, Source::Battery),
        "lid-ac" => (SUB_BUTTONS, LID_ACTION, Source::Plugged),
        "lid-dc" => (SUB_BUTTONS, LID_ACTION, Source::Battery),
        "power-button-ac" => (SUB_BUTTONS, POWER_BUTTON, Source::Plugged),
        "power-button-dc" => (SUB_BUTTONS, POWER_BUTTON, Source::Battery),
        "battery-saver-at" => (SUB_ENERGY_SAVER, SAVER_THRESHOLD, Source::Battery),
        _ => return None,
    })
}

fn check(code: WIN32_ERROR, what: &str) -> Result<()> {
    if code == ERROR_SUCCESS {
        Ok(())
    } else if code == windows::Win32::Foundation::ERROR_ACCESS_DENIED {
        Err(Error::AccessDenied)
    } else {
        Err(Error::msg(format!("Could not change {what}: {}", windows::core::Error::from(code.to_hresult()).message())))
    }
}

pub fn active_scheme() -> Option<GUID> {
    let mut ptr: *mut GUID = std::ptr::null_mut();
    unsafe {
        if PowerGetActiveScheme(None, &mut ptr) != ERROR_SUCCESS || ptr.is_null() {
            return None;
        }
        let guid = *ptr;
        let _ = LocalFree(Some(HLOCAL(ptr.cast())));
        Some(guid)
    }
}

fn friendly_name(scheme: &GUID) -> String {
    let mut size = 0u32;
    unsafe {
        let _ = PowerReadFriendlyName(None, Some(scheme), None, None, None, &mut size);
        if size == 0 {
            return String::new();
        }
        let mut buf = vec![0u8; size as usize];
        if PowerReadFriendlyName(None, Some(scheme), None, None, Some(buf.as_mut_ptr()), &mut size) != ERROR_SUCCESS {
            return String::new();
        }
        let wide: Vec<u16> = buf.chunks_exact(2).map(|c| u16::from_le_bytes([c[0], c[1]])).collect();
        let end = wide.iter().position(|c| *c == 0).unwrap_or(wide.len());
        String::from_utf16_lossy(&wide[..end])
    }
}

pub fn plans() -> Vec<PowerPlan> {
    let active = active_scheme();
    let mut out = Vec::new();
    for index in 0..64 {
        let mut guid = GUID::zeroed();
        let mut size = std::mem::size_of::<GUID>() as u32;
        let code = unsafe {
            PowerEnumerate(None, None, None, ACCESS_SCHEME, index, Some((&mut guid as *mut GUID).cast()), &mut size)
        };
        if code != ERROR_SUCCESS {
            break;
        }
        out.push(PowerPlan {
            id: format!("{guid:?}").to_lowercase(),
            name: friendly_name(&guid),
            active: Some(guid) == active,
        });
    }
    out
}

pub fn set_plan(id: &str) -> Result<()> {
    let guid = plans()
        .into_iter()
        .find(|p| p.id == id)
        .map(|p| GUID::try_from(p.id.as_str()))
        .ok_or_else(|| Error::NotFound(format!("power plan {id}")))?
        .map_err(|_| Error::Invalid(id.to_owned()))?;
    check(unsafe { PowerSetActiveScheme(None, Some(&guid)) }, "the power plan")
}

pub fn battery() -> BatteryStatus {
    let mut status = SYSTEM_POWER_STATUS::default();
    if unsafe { GetSystemPowerStatus(&mut status) }.is_err() {
        return BatteryStatus::default();
    }
    // 128 = no system battery, 255 = unknown.
    let has_battery = status.BatteryFlag != 128 && status.BatteryFlag != 255;
    BatteryStatus {
        has_battery,
        percent: (status.BatteryLifePercent <= 100).then_some(status.BatteryLifePercent),
        plugged_in: status.ACLineStatus == 1,
        charging: status.BatteryFlag & 8 != 0,
        saver_on: status.SystemStatusFlag == 1,
    }
}

fn on_battery() -> bool {
    let b = battery();
    b.has_battery && !b.plugged_in
}

fn resolve(source: Source) -> Source {
    match source {
        Source::Current if on_battery() => Source::Battery,
        Source::Current => Source::Plugged,
        other => other,
    }
}

pub fn get(id: &str) -> Option<i32> {
    let (sub, setting_id, source) = setting(id)?;
    let scheme = active_scheme()?;
    let mut value = 0u32;
    let code = unsafe {
        match resolve(source) {
            Source::Battery => {
                WIN32_ERROR(PowerReadDCValueIndex(None, Some(&scheme), Some(&sub), Some(&setting_id), &mut value))
            }
            _ => PowerReadACValueIndex(None, Some(&scheme), Some(&sub), Some(&setting_id), &mut value),
        }
    };
    if code != ERROR_SUCCESS {
        return None;
    }
    i32::try_from(value).ok()
}

pub fn set(id: &str, value: i32) -> Result<()> {
    let (sub, setting_id, source) = setting(id).ok_or_else(|| Error::NotFound(format!("setting {id}")))?;
    let scheme = active_scheme().ok_or_else(|| Error::msg("Windows didn't report an active power plan."))?;
    let value = u32::try_from(value).map_err(|_| Error::Invalid(value.to_string()))?;
    let code = unsafe {
        match resolve(source) {
            Source::Battery => WIN32_ERROR(PowerWriteDCValueIndex(None, &scheme, Some(&sub), Some(&setting_id), value)),
            _ => PowerWriteACValueIndex(None, &scheme, Some(&sub), Some(&setting_id), value),
        }
    };
    check(code, "the power setting")?;
    // Re-applying the plan makes Windows pick up the new value right away.
    check(unsafe { PowerSetActiveScheme(None, Some(&scheme)) }, "the power plan")
}
