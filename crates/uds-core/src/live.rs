//! Settings Windows applies through system calls rather than plain registry
//! values: mouse, keyboard, visual effects, taskbar auto-hide and power. They
//! take effect at once and are saved the way Windows' own Settings app saves
//! them. Reading and writing happens in [`crate::system::live_get`] and
//! [`crate::system::live_set`]; this module holds the catalog and checks.

use crate::tweaks::{Page, RangeSpec, TweakState};
use crate::{Error, Result};

#[derive(Debug, Clone, Copy)]
pub enum Kind {
    Toggle,
    Range { min: i32, max: i32, step: i32, unit: &'static str },
    Choice(&'static [(&'static str, i32)]),
}

#[derive(Debug, Clone, Copy)]
pub struct LiveDef {
    pub id: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    pub page: Page,
    pub group: &'static str,
    pub kind: Kind,
}

const MINUTES: &[(&str, i32)] = &[
    ("Never", 0),
    ("1 min", 60),
    ("2 min", 120),
    ("3 min", 180),
    ("5 min", 300),
    ("10 min", 600),
    ("15 min", 900),
    ("20 min", 1200),
    ("30 min", 1800),
    ("45 min", 2700),
    ("1 hour", 3600),
    ("2 hours", 7200),
    ("3 hours", 10800),
    ("5 hours", 18000),
];
const LID: &[(&str, i32)] = &[("Do nothing", 0), ("Sleep", 1), ("Hibernate", 2), ("Shut down", 3)];
const BUTTON: &[(&str, i32)] =
    &[("Do nothing", 0), ("Sleep", 1), ("Hibernate", 2), ("Shut down", 3), ("Turn off the display", 4)];
const SAVER: &[(&str, i32)] =
    &[("Never", 0), ("10%", 10), ("20%", 20), ("30%", 30), ("40%", 40), ("50%", 50), ("Always", 100)];

const fn def(
    id: &'static str,
    page: Page,
    group: &'static str,
    title: &'static str,
    description: &'static str,
    kind: Kind,
) -> LiveDef {
    LiveDef { id, title, description, page, group, kind }
}

const fn range(min: i32, max: i32, step: i32, unit: &'static str) -> Kind {
    Kind::Range { min, max, step, unit }
}

pub const CATALOG: &[LiveDef] = &[
    // Mouse & keyboard
    def(
        "mouse-speed",
        Page::Input,
        "Mouse",
        "Pointer speed",
        "How far the pointer moves when you move the mouse.",
        range(1, 20, 1, ""),
    ),
    def(
        "pointer-precision",
        Page::Input,
        "Mouse",
        "Enhance pointer precision",
        "Pointer acceleration: faster moves go further.",
        Kind::Toggle,
    ),
    def(
        "swap-buttons",
        Page::Input,
        "Mouse",
        "Swap primary button",
        "Use the right button as the main button (left-handed).",
        Kind::Toggle,
    ),
    def(
        "scroll-lines",
        Page::Input,
        "Mouse",
        "Lines to scroll",
        "How many lines one notch of the wheel scrolls.",
        range(1, 20, 1, " lines"),
    ),
    def(
        "double-click",
        Page::Input,
        "Mouse",
        "Double-click speed",
        "The longest gap between two clicks of a double-click.",
        range(200, 900, 50, " ms"),
    ),
    def(
        "mouse-sonar",
        Page::Input,
        "Mouse",
        "Find the pointer with Ctrl",
        "Pressing Ctrl shows rings around the pointer.",
        Kind::Toggle,
    ),
    def(
        "snap-to-default",
        Page::Input,
        "Mouse",
        "Jump to the default button",
        "The pointer moves to the default button in dialogs.",
        Kind::Toggle,
    ),
    def(
        "hover-activate",
        Page::Input,
        "Mouse",
        "Activate windows by hovering",
        "A window comes to the front when the pointer rests on it.",
        Kind::Toggle,
    ),
    def("cursor-shadow", Page::Input, "Mouse", "Pointer shadow", "A soft shadow under the pointer.", Kind::Toggle),
    def(
        "repeat-delay",
        Page::Input,
        "Keyboard",
        "Repeat delay",
        "How long to hold a key before it repeats (short to long).",
        range(0, 3, 1, ""),
    ),
    def(
        "repeat-rate",
        Page::Input,
        "Keyboard",
        "Repeat rate",
        "How fast a held key repeats (slow to fast).",
        range(0, 31, 1, ""),
    ),
    def(
        "caret-blink",
        Page::Input,
        "Keyboard",
        "Cursor blink rate",
        "Time between blinks of the text cursor.",
        range(200, 1200, 50, " ms"),
    ),
    // Visual effects
    def(
        "animations",
        Page::Look,
        "Effects",
        "Animation effects",
        "Windows and controls animate. Turn off for a snappier feel.",
        Kind::Toggle,
    ),
    def(
        "window-animation",
        Page::Look,
        "Effects",
        "Animate minimize and maximize",
        "Windows zoom when they minimize and maximize.",
        Kind::Toggle,
    ),
    def("menu-animation", Page::Look, "Effects", "Fade or slide menus", "Menus animate when they open.", Kind::Toggle),
    def(
        "drag-full-windows",
        Page::Look,
        "Effects",
        "Show window contents while dragging",
        "Off shows only an outline while you move a window.",
        Kind::Toggle,
    ),
    def("font-smoothing", Page::Look, "Effects", "Smooth screen fonts", "ClearType edges on text.", Kind::Toggle),
    def(
        "snap-windows",
        Page::System,
        "Multitasking",
        "Snap windows",
        "Drag windows to an edge or corner to arrange them.",
        Kind::Toggle,
    ),
    // Taskbar
    def(
        "taskbar-autohide",
        Page::Taskbar,
        "Taskbar",
        "Automatically hide the taskbar",
        "The taskbar slides away until you point at the screen edge.",
        Kind::Toggle,
    ),
    // Power (active power plan; plugged in / on battery)
    def(
        "brightness",
        Page::Power,
        "Display",
        "Brightness",
        "Built-in screen brightness for the current power source.",
        range(0, 100, 5, "%"),
    ),
    def(
        "screen-off-ac",
        Page::Power,
        "Screen and sleep",
        "Turn off the screen (plugged in)",
        "After this long without use.",
        Kind::Choice(MINUTES),
    ),
    def(
        "screen-off-dc",
        Page::Power,
        "Screen and sleep",
        "Turn off the screen (on battery)",
        "After this long without use.",
        Kind::Choice(MINUTES),
    ),
    def(
        "sleep-ac",
        Page::Power,
        "Screen and sleep",
        "Sleep (plugged in)",
        "After this long without use.",
        Kind::Choice(MINUTES),
    ),
    def(
        "sleep-dc",
        Page::Power,
        "Screen and sleep",
        "Sleep (on battery)",
        "After this long without use.",
        Kind::Choice(MINUTES),
    ),
    def(
        "lid-ac",
        Page::Power,
        "Buttons and lid",
        "Closing the lid (plugged in)",
        "What happens when you close the lid.",
        Kind::Choice(LID),
    ),
    def(
        "lid-dc",
        Page::Power,
        "Buttons and lid",
        "Closing the lid (on battery)",
        "What happens when you close the lid.",
        Kind::Choice(LID),
    ),
    def(
        "power-button-ac",
        Page::Power,
        "Buttons and lid",
        "Power button (plugged in)",
        "What the power button does.",
        Kind::Choice(BUTTON),
    ),
    def(
        "power-button-dc",
        Page::Power,
        "Buttons and lid",
        "Power button (on battery)",
        "What the power button does.",
        Kind::Choice(BUTTON),
    ),
    def(
        "battery-saver-at",
        Page::Power,
        "Battery",
        "Turn on battery saver at",
        "Battery saver starts at this charge.",
        Kind::Choice(SAVER),
    ),
];

pub fn find(id: &str) -> Option<&'static LiveDef> {
    CATALOG.iter().find(|d| d.id == id)
}

/// Checks a new value against the setting's kind before it reaches Windows.
pub fn validate(def: &LiveDef, value: i32) -> Result<i32> {
    let ok = match def.kind {
        Kind::Toggle => value == 0 || value == 1,
        Kind::Range { min, max, .. } => (min..=max).contains(&value),
        Kind::Choice(options) => options.iter().any(|(_, v)| *v == value),
    };
    if ok { Ok(value) } else { Err(Error::Invalid(format!("{} can't be {value}", def.title))) }
}

/// The UI shape shared with registry settings. `value` is `None` when
/// Windows couldn't report it (or off Windows).
pub fn state(def: &LiveDef, value: Option<i32>) -> TweakState {
    let (toggle, options, selected, range) = match def.kind {
        Kind::Toggle => (true, vec!["Off".to_owned(), "On".to_owned()], value.map(|v| usize::from(v != 0)), None),
        Kind::Range { min, max, step, unit } => (false, Vec::new(), None, Some(RangeSpec { min, max, step, unit })),
        Kind::Choice(options) => (
            false,
            options.iter().map(|(label, _)| (*label).to_owned()).collect(),
            value.and_then(|v| options.iter().position(|(_, o)| *o == v)),
            None,
        ),
    };
    TweakState {
        id: def.id.into(),
        title: def.title.into(),
        description: def.description.into(),
        page: def.page,
        group: def.group.into(),
        toggle,
        options,
        enabled: toggle && value.is_some_and(|v| v != 0),
        selected,
        needs_restart: false,
        range,
        value,
    }
}

/// For choices the UI sends an option index; this turns it into the value.
pub fn value_for(def: &LiveDef, input: i32) -> Result<i32> {
    match def.kind {
        Kind::Choice(options) => usize::try_from(input)
            .ok()
            .and_then(|i| options.get(i))
            .map(|(_, v)| *v)
            .ok_or_else(|| Error::Invalid(format!("option {input} of {}", def.id))),
        _ => validate(def, input),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_unique_and_do_not_clash_with_registry_settings() {
        let mut ids: Vec<_> = CATALOG.iter().map(|d| d.id).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), CATALOG.len());
        for d in CATALOG {
            assert!(crate::tweaks::find(d.id).is_err(), "{} is also a registry setting", d.id);
        }
    }

    #[test]
    fn preview_sample_lists_every_setting() {
        // The browser preview shows the same settings as the app.
        let sample = include_str!("../../../src/lib/mockSettings.ts");
        let ids = crate::tweaks::CATALOG.iter().map(|t| t.id).chain(CATALOG.iter().map(|d| d.id));
        for id in ids {
            assert!(sample.contains(&format!("id: \"{id}\"")), "src/lib/mockSettings.ts is missing {id}");
        }
    }

    #[test]
    fn values_are_validated() {
        let speed = find("mouse-speed").unwrap();
        assert!(validate(speed, 10).is_ok());
        assert!(validate(speed, 0).is_err());
        assert!(validate(speed, 21).is_err());
        let toggle = find("animations").unwrap();
        assert!(validate(toggle, 2).is_err());
        let lid = find("lid-dc").unwrap();
        assert_eq!(value_for(lid, 1).unwrap(), 1);
        assert!(value_for(lid, 9).is_err());
        assert!(value_for(lid, -1).is_err());
        let screen = find("screen-off-ac").unwrap();
        assert_eq!(value_for(screen, 4).unwrap(), 300);
        assert!(validate(screen, 301).is_err());
    }

    #[test]
    fn state_matches_the_ui_shape() {
        let s = state(find("swap-buttons").unwrap(), Some(1));
        assert!(s.toggle && s.enabled && s.selected == Some(1));
        let s = state(find("screen-off-ac").unwrap(), Some(600));
        assert_eq!(s.selected, Some(5));
        assert_eq!(s.options[5], "10 min");
        let s = state(find("screen-off-ac").unwrap(), Some(420));
        assert_eq!(s.selected, None, "a custom time from Control Panel isn't mislabelled");
        let s = state(find("brightness").unwrap(), None);
        assert_eq!(s.range.unwrap().max, 100);
        assert_eq!(s.value, None);
    }
}
