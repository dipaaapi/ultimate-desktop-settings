//! The floating rod window: no title bar, no background, sized to the rod.

use std::sync::Mutex;

use tauri::{PhysicalPosition, PhysicalSize, WebviewWindow};
use uds_core::window_layout::{self, Mode, Placement, Rect};

/// The side a board is open on, or `None` while only the rod shows.
#[derive(Default)]
pub struct WindowState(Mutex<Option<Placement>>);

fn rects(window: &WebviewWindow) -> tauri::Result<Option<(Rect, Rect, f64)>> {
    let Some(monitor) = window.current_monitor()?.or(window.primary_monitor()?) else {
        return Ok(None);
    };
    let pos = window.outer_position()?;
    let size = window.outer_size()?;
    let area = monitor.work_area();
    let current = Rect { x: pos.x, y: pos.y, width: size.width as i32, height: size.height as i32 };
    let area =
        Rect { x: area.position.x, y: area.position.y, width: area.size.width as i32, height: area.size.height as i32 };
    Ok(Some((current, area, monitor.scale_factor())))
}

/// Resizes the window for `mode` and returns which side boards open on.
pub fn apply(window: &WebviewWindow, state: &WindowState, mode: Mode) -> tauri::Result<Placement> {
    let mut open = state.0.lock().unwrap_or_else(|e| e.into_inner());
    let Some((current, area, scale)) = rects(window)? else {
        return Ok(Placement::Below);
    };
    let rod = window_layout::rod_rect(current, area, scale, *open);
    let (rect, placement) = window_layout::window_rect(rod, area, scale, mode);
    window.set_size(PhysicalSize::new(rect.width as u32, rect.height as u32))?;
    window.set_position(PhysicalPosition::new(rect.x, rect.y))?;
    *open = (mode == Mode::Board).then_some(placement);
    Ok(placement)
}

/// Which side boards would open on right now (labels follow it too).
pub fn placement(window: &WebviewWindow, state: &WindowState) -> tauri::Result<Placement> {
    let open = *state.0.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(placement) = open {
        return Ok(placement);
    }
    let Some((current, area, scale)) = rects(window)? else {
        return Ok(Placement::Below);
    };
    Ok(window_layout::placement_for(window_layout::rod_rect(current, area, scale, None), area))
}
