//! Windows desktop settings that are plain per-user registry values.
//!
//! Only the values in [`CATALOG`] can be changed, so the UI can never write an
//! arbitrary registry path.

use serde::Serialize;

use crate::registry::{Hive, RegValue, Registry};
use crate::{Error, Result};

const PERSONALIZE: &str = r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize";
const ADVANCED: &str = r"Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced";
const DEV_SETTINGS: &str = r"Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced\TaskbarDeveloperSettings";

/// What Windows needs to hear about after a value changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Refresh {
    /// Broadcast `ImmersiveColorSet` so apps repaint in the new theme.
    Theme,
    /// Tell Explorer windows to re-read their view settings.
    Explorer,
    /// Tell the taskbar to re-read its settings.
    Taskbar,
}

#[derive(Debug, Clone, Copy)]
pub struct TweakDef {
    pub id: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    pub group: &'static str,
    pub path: &'static str,
    pub value: &'static str,
    pub on: u32,
    pub off: u32,
    /// State Windows assumes when the value doesn't exist yet.
    pub default_on: bool,
    pub refresh: Refresh,
}

pub const CATALOG: &[TweakDef] = &[
    TweakDef {
        id: "apps-dark",
        title: "Dark mode for apps",
        description: "Apps that follow the Windows theme use dark colors.",
        group: "Theme",
        path: PERSONALIZE,
        value: "AppsUseLightTheme",
        on: 0,
        off: 1,
        default_on: false,
        refresh: Refresh::Theme,
    },
    TweakDef {
        id: "system-dark",
        title: "Dark mode for Windows",
        description: "Taskbar, Start menu and notifications use dark colors.",
        group: "Theme",
        path: PERSONALIZE,
        value: "SystemUsesLightTheme",
        on: 0,
        off: 1,
        default_on: false,
        refresh: Refresh::Theme,
    },
    TweakDef {
        id: "transparency",
        title: "Transparency effects",
        description: "Windows and surfaces appear slightly see-through.",
        group: "Theme",
        path: PERSONALIZE,
        value: "EnableTransparency",
        on: 1,
        off: 0,
        default_on: true,
        refresh: Refresh::Theme,
    },
    TweakDef {
        id: "file-extensions",
        title: "Show file extensions",
        description: "File Explorer shows endings like .exe, .pdf and .txt.",
        group: "File Explorer",
        path: ADVANCED,
        value: "HideFileExt",
        on: 0,
        off: 1,
        default_on: false,
        refresh: Refresh::Explorer,
    },
    TweakDef {
        id: "hidden-files",
        title: "Show hidden files",
        description: "File Explorer shows files and folders marked hidden.",
        group: "File Explorer",
        path: ADVANCED,
        value: "Hidden",
        on: 1,
        off: 2,
        default_on: false,
        refresh: Refresh::Explorer,
    },
    TweakDef {
        id: "open-this-pc",
        title: "Open File Explorer to This PC",
        description: "New Explorer windows start at This PC instead of Home.",
        group: "File Explorer",
        path: ADVANCED,
        value: "LaunchTo",
        on: 1,
        off: 2,
        default_on: false,
        refresh: Refresh::Explorer,
    },
    TweakDef {
        id: "compact-view",
        title: "Compact view",
        description: "Tighter spacing between files in File Explorer.",
        group: "File Explorer",
        path: ADVANCED,
        value: "UseCompactMode",
        on: 1,
        off: 0,
        default_on: false,
        refresh: Refresh::Explorer,
    },
    TweakDef {
        id: "taskbar-left",
        title: "Align taskbar to the left",
        description: "Start and pinned apps sit on the left like Windows 10.",
        group: "Taskbar",
        path: ADVANCED,
        value: "TaskbarAl",
        on: 0,
        off: 1,
        default_on: false,
        refresh: Refresh::Taskbar,
    },
    TweakDef {
        id: "task-view-button",
        title: "Task View button",
        description: "Shows the Task View button on the taskbar.",
        group: "Taskbar",
        path: ADVANCED,
        value: "ShowTaskViewButton",
        on: 1,
        off: 0,
        default_on: true,
        refresh: Refresh::Taskbar,
    },
    TweakDef {
        id: "clock-seconds",
        title: "Show seconds in the clock",
        description: "The taskbar clock shows seconds (Windows 11).",
        group: "Taskbar",
        path: ADVANCED,
        value: "ShowSecondsInSystemClock",
        on: 1,
        off: 0,
        default_on: false,
        refresh: Refresh::Taskbar,
    },
    TweakDef {
        id: "end-task",
        title: "End task on right-click",
        description: "Right-clicking a taskbar app offers End task (Windows 11).",
        group: "Taskbar",
        path: DEV_SETTINGS,
        value: "TaskbarEndTask",
        on: 1,
        off: 0,
        default_on: false,
        refresh: Refresh::Taskbar,
    },
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TweakState {
    pub id: String,
    pub title: String,
    pub description: String,
    pub group: String,
    pub enabled: bool,
}

pub fn find(id: &str) -> Result<&'static TweakDef> {
    CATALOG.iter().find(|t| t.id == id).ok_or_else(|| Error::NotFound(format!("setting {id}")))
}

pub fn is_on(reg: &dyn Registry, def: &TweakDef) -> bool {
    match reg.get_dword(Hive::CurrentUser, def.path, def.value) {
        Some(v) if v == def.on => true,
        Some(v) if v == def.off => false,
        // Unknown or missing values: Windows falls back to its default.
        _ => def.default_on,
    }
}

pub fn list(reg: &dyn Registry) -> Vec<TweakState> {
    CATALOG
        .iter()
        .map(|def| TweakState {
            id: def.id.into(),
            title: def.title.into(),
            description: def.description.into(),
            group: def.group.into(),
            enabled: is_on(reg, def),
        })
        .collect()
}

pub fn set(reg: &dyn Registry, id: &str, enabled: bool) -> Result<Refresh> {
    let def = find(id)?;
    let value = if enabled { def.on } else { def.off };
    reg.set(Hive::CurrentUser, def.path, def.value, RegValue::Dword(value))?;
    Ok(def.refresh)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::MemoryRegistry;

    #[test]
    fn catalog_ids_are_unique() {
        let mut ids: Vec<_> = CATALOG.iter().map(|t| t.id).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), CATALOG.len());
        assert!(CATALOG.iter().all(|t| t.on != t.off));
    }

    #[test]
    fn defaults_apply_when_missing() {
        let reg = MemoryRegistry::new();
        let states = list(&reg);
        let transparency = states.iter().find(|s| s.id == "transparency").unwrap();
        assert!(transparency.enabled);
        let ext = states.iter().find(|s| s.id == "file-extensions").unwrap();
        assert!(!ext.enabled);
    }

    #[test]
    fn set_writes_the_documented_values() {
        let reg = MemoryRegistry::new();
        assert_eq!(set(&reg, "apps-dark", true).unwrap(), Refresh::Theme);
        assert_eq!(reg.get_dword(Hive::CurrentUser, PERSONALIZE, "AppsUseLightTheme"), Some(0));
        set(&reg, "hidden-files", true).unwrap();
        assert_eq!(reg.get_dword(Hive::CurrentUser, ADVANCED, "Hidden"), Some(1));
        set(&reg, "hidden-files", false).unwrap();
        assert_eq!(reg.get_dword(Hive::CurrentUser, ADVANCED, "Hidden"), Some(2));
        assert!(!list(&reg).iter().find(|s| s.id == "hidden-files").unwrap().enabled);
    }

    #[test]
    fn unknown_ids_are_rejected() {
        let reg = MemoryRegistry::new();
        assert!(set(&reg, "anything-else", true).is_err());
    }
}
