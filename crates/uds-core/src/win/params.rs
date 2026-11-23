//! Mouse, keyboard and visual-effect settings through `SystemParametersInfo`.
//! Every change is saved to the user profile and broadcast, like Settings does.

use windows::Win32::UI::Input::KeyboardAndMouse::{GetDoubleClickTime, SwapMouseButton};
use windows::Win32::UI::WindowsAndMessaging::{
    ANIMATIONINFO, GetCaretBlinkTime, GetSystemMetrics, SM_SWAPBUTTON, SPI_GETACTIVEWINDOWTRACKING, SPI_GETANIMATION,
    SPI_GETCLIENTAREAANIMATION, SPI_GETCURSORSHADOW, SPI_GETDRAGFULLWINDOWS, SPI_GETFONTSMOOTHING,
    SPI_GETKEYBOARDDELAY, SPI_GETKEYBOARDSPEED, SPI_GETMENUANIMATION, SPI_GETMOUSE, SPI_GETMOUSESONAR,
    SPI_GETMOUSESPEED, SPI_GETSNAPTODEFBUTTON, SPI_GETWHEELSCROLLLINES, SPI_GETWINARRANGING,
    SPI_SETACTIVEWINDOWTRACKING, SPI_SETANIMATION, SPI_SETCLIENTAREAANIMATION, SPI_SETCURSORSHADOW,
    SPI_SETDOUBLECLICKTIME, SPI_SETDRAGFULLWINDOWS, SPI_SETFONTSMOOTHING, SPI_SETKEYBOARDDELAY, SPI_SETKEYBOARDSPEED,
    SPI_SETMENUANIMATION, SPI_SETMOUSE, SPI_SETMOUSEBUTTONSWAP, SPI_SETMOUSESONAR, SPI_SETMOUSESPEED,
    SPI_SETSNAPTODEFBUTTON, SPI_SETWHEELSCROLLLINES, SPI_SETWINARRANGING, SPIF_SENDCHANGE, SPIF_UPDATEINIFILE,
    SYSTEM_PARAMETERS_INFO_ACTION, SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS, SetCaretBlinkTime, SystemParametersInfoW,
};

use crate::registry::{Hive, RegValue, Registry};
use crate::{Error, Result};

const SAVE: SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS =
    SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(SPIF_UPDATEINIFILE.0 | SPIF_SENDCHANGE.0);
const NONE: SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS = SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0);

/// How a setter passes its value: in `uiParam`, or as the `pvParam` pointer itself.
#[derive(Clone, Copy)]
enum Pass {
    Ui,
    Pv,
}

fn get_u32(action: SYSTEM_PARAMETERS_INFO_ACTION) -> Option<i32> {
    let mut value: u32 = 0;
    unsafe { SystemParametersInfoW(action, 0, Some((&mut value as *mut u32).cast()), NONE) }.ok()?;
    i32::try_from(value).ok()
}

fn set(action: SYSTEM_PARAMETERS_INFO_ACTION, value: i32, pass: Pass) -> Result<()> {
    let result = unsafe {
        match pass {
            Pass::Ui => SystemParametersInfoW(action, value as u32, None, SAVE),
            Pass::Pv => SystemParametersInfoW(action, 0, Some(value as isize as *mut core::ffi::c_void), SAVE),
        }
    };
    result.map_err(|e| Error::msg(format!("Windows didn't accept the change: {}", e.message())))
}

/// Getter and setter actions for the settings that follow the plain pattern.
fn simple(id: &str) -> Option<(SYSTEM_PARAMETERS_INFO_ACTION, SYSTEM_PARAMETERS_INFO_ACTION, Pass)> {
    Some(match id {
        "mouse-speed" => (SPI_GETMOUSESPEED, SPI_SETMOUSESPEED, Pass::Pv),
        "scroll-lines" => (SPI_GETWHEELSCROLLLINES, SPI_SETWHEELSCROLLLINES, Pass::Ui),
        "mouse-sonar" => (SPI_GETMOUSESONAR, SPI_SETMOUSESONAR, Pass::Pv),
        "snap-to-default" => (SPI_GETSNAPTODEFBUTTON, SPI_SETSNAPTODEFBUTTON, Pass::Ui),
        "hover-activate" => (SPI_GETACTIVEWINDOWTRACKING, SPI_SETACTIVEWINDOWTRACKING, Pass::Pv),
        "cursor-shadow" => (SPI_GETCURSORSHADOW, SPI_SETCURSORSHADOW, Pass::Pv),
        "repeat-delay" => (SPI_GETKEYBOARDDELAY, SPI_SETKEYBOARDDELAY, Pass::Ui),
        "repeat-rate" => (SPI_GETKEYBOARDSPEED, SPI_SETKEYBOARDSPEED, Pass::Ui),
        "animations" => (SPI_GETCLIENTAREAANIMATION, SPI_SETCLIENTAREAANIMATION, Pass::Pv),
        "menu-animation" => (SPI_GETMENUANIMATION, SPI_SETMENUANIMATION, Pass::Pv),
        "drag-full-windows" => (SPI_GETDRAGFULLWINDOWS, SPI_SETDRAGFULLWINDOWS, Pass::Ui),
        "font-smoothing" => (SPI_GETFONTSMOOTHING, SPI_SETFONTSMOOTHING, Pass::Ui),
        "snap-windows" => (SPI_GETWINARRANGING, SPI_SETWINARRANGING, Pass::Pv),
        _ => return None,
    })
}

fn animation_info() -> Option<ANIMATIONINFO> {
    let mut info = ANIMATIONINFO { cbSize: std::mem::size_of::<ANIMATIONINFO>() as u32, iMinAnimate: 0 };
    unsafe {
        SystemParametersInfoW(SPI_GETANIMATION, info.cbSize, Some((&mut info as *mut ANIMATIONINFO).cast()), NONE)
    }
    .ok()?;
    Some(info)
}

pub fn get(id: &str) -> Option<i32> {
    if let Some((getter, _, _)) = simple(id) {
        return get_u32(getter).map(|v| if v != 0 && is_toggle(id) { 1 } else { v });
    }
    match id {
        "pointer-precision" => {
            let mut mouse = [0i32; 3];
            unsafe { SystemParametersInfoW(SPI_GETMOUSE, 0, Some(mouse.as_mut_ptr().cast()), NONE) }.ok()?;
            Some(i32::from(mouse[2] != 0))
        }
        "swap-buttons" => Some(i32::from(unsafe { GetSystemMetrics(SM_SWAPBUTTON) } != 0)),
        "double-click" => i32::try_from(unsafe { GetDoubleClickTime() }).ok(),
        "caret-blink" => i32::try_from(unsafe { GetCaretBlinkTime() }).ok(),
        "window-animation" => animation_info().map(|i| i32::from(i.iMinAnimate != 0)),
        _ => None,
    }
}

fn is_toggle(id: &str) -> bool {
    crate::live::find(id).is_some_and(|d| matches!(d.kind, crate::live::Kind::Toggle))
}

pub fn set_value(reg: &dyn Registry, id: &str, value: i32) -> Result<()> {
    if let Some((_, setter, pass)) = simple(id) {
        return set(setter, value, pass);
    }
    match id {
        "pointer-precision" => {
            // Windows' defaults for the two thresholds when acceleration is on.
            let mut mouse = if value != 0 { [6i32, 10, 1] } else { [0i32, 0, 0] };
            unsafe { SystemParametersInfoW(SPI_SETMOUSE, 0, Some(mouse.as_mut_ptr().cast()), SAVE) }
                .map_err(|e| Error::msg(e.message()))
        }
        "swap-buttons" => {
            unsafe {
                let _ = SwapMouseButton(value != 0);
            }
            set(SPI_SETMOUSEBUTTONSWAP, value, Pass::Ui)
        }
        "double-click" => set(SPI_SETDOUBLECLICKTIME, value, Pass::Ui),
        "caret-blink" => {
            unsafe { SetCaretBlinkTime(value as u32) }.map_err(|e| Error::msg(e.message()))?;
            // SetCaretBlinkTime lasts until sign-out; the registry keeps it.
            reg.set(Hive::CurrentUser, r"Control Panel\Desktop", "CursorBlinkRate", RegValue::String(value.to_string()))
        }
        "window-animation" => {
            let mut info = ANIMATIONINFO { cbSize: std::mem::size_of::<ANIMATIONINFO>() as u32, iMinAnimate: value };
            unsafe {
                SystemParametersInfoW(
                    SPI_SETANIMATION,
                    info.cbSize,
                    Some((&mut info as *mut ANIMATIONINFO).cast()),
                    SAVE,
                )
            }
            .map_err(|e| Error::msg(e.message()))
        }
        _ => Err(Error::NotFound(format!("setting {id}"))),
    }
}
