//! The floating rod window: no title bar, no background, sized to the rod.

use std::sync::Mutex;

use tauri::{PhysicalPosition, PhysicalSize, WebviewWindow};
use uds_core::window_layout::{self, Mode, Placement, Rect};

/// The side a board is open on (`None` while only the rod shows) and the
/// app size the user picked with the size slider.
pub struct WindowState {
    open: Mutex<Option<Placement>>,
    ui_scale: Mutex<f64>,
}

impl WindowState {
    pub fn new(ui_scale: f64) -> Self {
        Self { open: Mutex::new(None), ui_scale: Mutex::new(ui_scale) }
    }

    fn ui_scale(&self) -> f64 {
        *self.ui_scale.lock().unwrap_or_else(|e| e.into_inner())
    }
}

fn rects(window: &WebviewWindow, ui_scale: f64) -> tauri::Result<Option<(Rect, Rect, f64)>> {
    let Some(monitor) = window.current_monitor()?.or(window.primary_monitor()?) else {
        return Ok(None);
    };
    let pos = window.outer_position()?;
    let size = window.outer_size()?;
    let area = monitor.work_area();
    let current = Rect { x: pos.x, y: pos.y, width: size.width as i32, height: size.height as i32 };
    let area =
        Rect { x: area.position.x, y: area.position.y, width: area.size.width as i32, height: area.size.height as i32 };
    // The page is zoomed by the app size, so the window grows with it.
    Ok(Some((current, area, monitor.scale_factor() * ui_scale)))
}

/// Resizes the window for `mode` and returns which side boards open on.
pub fn apply(window: &WebviewWindow, state: &WindowState, mode: Mode) -> tauri::Result<Placement> {
    let mut open = state.open.lock().unwrap_or_else(|e| e.into_inner());
    let Some((current, area, scale)) = rects(window, state.ui_scale())? else {
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
    let open = *state.open.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(placement) = open {
        return Ok(placement);
    }
    let Some((current, area, scale)) = rects(window, state.ui_scale())? else {
        return Ok(Placement::Below);
    };
    Ok(window_layout::placement_for(window_layout::rod_rect(current, area, scale, None), area))
}

/// Applies a new app size: zooms the page and resizes the window around the
/// rod, keeping a board open if one is.
pub fn set_ui_scale(window: &WebviewWindow, state: &WindowState, ui_scale: f64) -> tauri::Result<Placement> {
    // Measure the rod at the old size before the size changes.
    let mode = if state.open.lock().unwrap_or_else(|e| e.into_inner()).is_some() { Mode::Board } else { Mode::Rod };
    let old = state.ui_scale();
    if let Some((current, area, scale)) = rects(window, old)? {
        let open = *state.open.lock().unwrap_or_else(|e| e.into_inner());
        let rod = window_layout::rod_rect(current, area, scale, open);
        // Re-anchor the window on the rod so the new size grows from it.
        window.set_size(PhysicalSize::new(rod.width as u32, rod.height as u32))?;
        window.set_position(PhysicalPosition::new(rod.x, rod.y))?;
        *state.open.lock().unwrap_or_else(|e| e.into_inner()) = None;
    }
    *state.ui_scale.lock().unwrap_or_else(|e| e.into_inner()) = ui_scale;
    window.set_zoom(ui_scale)?;
    apply(window, state, mode)
}
