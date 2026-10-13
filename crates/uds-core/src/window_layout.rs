//! Where the floating rod window sits on screen.
//!
//! The window is only as big as the rod. Opening a board grows it towards the
//! roomier side of the screen: down when the rod is in the top half, up when
//! it is in the bottom half. The rod itself stays where the user dragged it.

use serde::{Deserialize, Serialize};

/// Size of the rod strip and of a board, in logical pixels.
pub const ROD_WIDTH: f64 = 1180.0;
pub const ROD_HEIGHT: f64 = 280.0;
pub const BOARD_HEIGHT: f64 = 620.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Mode {
    Rod,
    Board,
}

/// Which side of the rod boards and labels open on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Placement {
    Below,
    Above,
}

/// A rectangle in physical pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl Rect {
    fn bottom(&self) -> i32 {
        self.y + self.height
    }
    fn right(&self) -> i32 {
        self.x + self.width
    }
}

fn clamp(value: i32, min: i32, max: i32) -> i32 {
    value.max(min).min(max.max(min))
}

/// Finds the rod strip inside the current window. `open` is the side a board
/// is open on, or `None` when the window is just the rod.
pub fn rod_rect(window: Rect, area: Rect, scale: f64, open: Option<Placement>) -> Rect {
    let width = ((ROD_WIDTH * scale).round() as i32).min(area.width);
    let height = ((ROD_HEIGHT * scale).round() as i32).min(area.height);
    let y = match open {
        Some(Placement::Above) => window.bottom() - height,
        _ => window.y,
    };
    Rect {
        x: clamp(window.x, area.x, area.right() - width),
        y: clamp(y, area.y, area.bottom() - height),
        width,
        height,
    }
}

/// Boards open towards the larger free side of the screen.
pub fn placement_for(rod: Rect, area: Rect) -> Placement {
    let center = rod.y + rod.height / 2;
    if center < area.y + area.height / 2 { Placement::Below } else { Placement::Above }
}

/// The window rectangle for `mode`, given the rod's current strip.
pub fn window_rect(rod: Rect, area: Rect, scale: f64, mode: Mode) -> (Rect, Placement) {
    let placement = placement_for(rod, area);
    if mode == Mode::Rod {
        return (rod, placement);
    }
    let board = ((BOARD_HEIGHT * scale).round() as i32).min(area.height - rod.height).max(0);
    let height = rod.height + board;
    let y = match placement {
        Placement::Below => clamp(rod.y, area.y, area.bottom() - height),
        Placement::Above => clamp(rod.bottom() - height, area.y, area.bottom() - height),
    };
    (Rect { y, height, ..rod }, placement)
}

#[cfg(test)]
mod tests {
    use super::*;

    const AREA: Rect = Rect { x: 0, y: 0, width: 1920, height: 1040 };

    #[test]
    fn opens_down_from_the_top_and_up_from_the_bottom() {
        let top = rod_rect(Rect { x: 300, y: 20, width: 1180, height: 280 }, AREA, 1.0, None);
        let (rect, placement) = window_rect(top, AREA, 1.0, Mode::Board);
        assert_eq!(placement, Placement::Below);
        assert_eq!((rect.y, rect.height), (20, 900));

        let bottom = rod_rect(Rect { x: 300, y: 740, width: 1180, height: 280 }, AREA, 1.0, None);
        let (rect, placement) = window_rect(bottom, AREA, 1.0, Mode::Board);
        assert_eq!(placement, Placement::Above);
        assert_eq!((rect.y, rect.bottom()), (120, 1020));
    }

    #[test]
    fn rod_stays_put_after_a_board_closes() {
        let rod = Rect { x: 300, y: 740, width: 1180, height: 280 };
        let (open, placement) = window_rect(rod, AREA, 1.0, Mode::Board);
        assert_eq!(rod_rect(open, AREA, 1.0, Some(placement)), rod);
    }

    #[test]
    fn stays_on_small_and_scaled_screens() {
        let small = Rect { x: 0, y: 0, width: 1024, height: 700 };
        let rod = rod_rect(Rect { x: 900, y: 650, width: 1180, height: 280 }, small, 1.25, None);
        assert_eq!(rod.width, 1024);
        assert_eq!(rod.height, 350);
        assert!(rod.bottom() <= 700 && rod.x == 0);
        let (open, _) = window_rect(rod, small, 1.25, Mode::Board);
        assert!(open.y >= 0 && open.bottom() <= 700);
    }

    #[test]
    fn serializes_for_the_ui() {
        assert_eq!(serde_json::to_string(&Placement::Below).unwrap(), "\"below\"");
        assert_eq!(serde_json::from_str::<Mode>("\"board\"").unwrap(), Mode::Board);
    }
}
