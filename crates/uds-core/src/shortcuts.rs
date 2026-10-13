//! Every page, tool and folder the app can open. The UI asks for an id from
//! this list; it can never ask Windows to run an arbitrary command.

use serde::Serialize;

use crate::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    /// A URI handled by Windows, such as `ms-settings:display`.
    Uri(&'static str),
    /// A program in the Windows folder, with fixed arguments.
    Program(&'static str, &'static [&'static str]),
    /// The current user's temp folder.
    TempFolder,
}

#[derive(Debug, Clone, Copy)]
pub struct Shortcut {
    pub id: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    pub group: &'static str,
    pub target: Target,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShortcutInfo {
    pub id: String,
    pub title: String,
    pub description: String,
    pub group: String,
}

const SETTINGS: &str = "Windows Settings";
const TOOLS: &str = "System Tools";
const FOLDERS: &str = "Folders & Apps";

macro_rules! settings {
    ($id:literal, $title:literal, $desc:literal, $uri:literal) => {
        Shortcut { id: $id, title: $title, description: $desc, group: SETTINGS, target: Target::Uri($uri) }
    };
}

pub const CATALOG: &[Shortcut] = &[
    settings!("display", "Display", "Resolution, scale, brightness and multiple monitors.", "ms-settings:display"),
    settings!("night-light", "Night light", "Windows' own night light schedule.", "ms-settings:nightlight"),
    settings!("sound", "Sound", "Output and input devices and system sounds.", "ms-settings:sound"),
    settings!(
        "app-volume",
        "App volume and device preferences",
        "Pick an output device per app.",
        "ms-settings:apps-volume"
    ),
    settings!("bluetooth", "Bluetooth & devices", "Pair and manage Bluetooth devices.", "ms-settings:bluetooth"),
    settings!("network", "Network & internet", "Wi-Fi, Ethernet, VPN and DNS.", "ms-settings:network"),
    settings!(
        "personalization",
        "Personalization",
        "Background, colors, themes and lock screen.",
        "ms-settings:personalization"
    ),
    settings!("taskbar", "Taskbar", "Taskbar items, behaviors and alignment.", "ms-settings:taskbar"),
    settings!(
        "notifications",
        "Notifications",
        "Notification and Do Not Disturb settings.",
        "ms-settings:notifications"
    ),
    settings!("apps", "Installed apps", "Modify or uninstall programs.", "ms-settings:appsfeatures"),
    settings!("default-apps", "Default apps", "Choose the default browser and other apps.", "ms-settings:defaultapps"),
    settings!("startup-apps", "Startup apps", "Windows' own startup apps page.", "ms-settings:startupapps"),
    settings!("date-time", "Date & time", "Time zone, clock sync and formats.", "ms-settings:dateandtime"),
    settings!("windows-update", "Windows Update", "Updates, update history and restarts.", "ms-settings:windowsupdate"),
    settings!(
        "privacy",
        "Privacy & security",
        "App permissions, Windows Security and firewall.",
        "ms-settings:privacy"
    ),
    settings!("about", "About this PC", "Processor, memory and Windows version.", "ms-settings:about"),
    settings!("troubleshoot", "Troubleshoot", "Windows troubleshooters, including audio.", "ms-settings:troubleshoot"),
    Shortcut {
        id: "control-panel",
        title: "Control Panel",
        description: "The classic Control Panel.",
        group: TOOLS,
        target: Target::Program("control.exe", &[]),
    },
    Shortcut {
        id: "task-manager",
        title: "Task Manager",
        description: "Running apps, performance and startup.",
        group: TOOLS,
        target: Target::Program("taskmgr.exe", &[]),
    },
    Shortcut {
        id: "device-manager",
        title: "Device Manager",
        description: "Hardware and drivers.",
        group: TOOLS,
        target: Target::Program("mmc.exe", &["devmgmt.msc"]),
    },
    Shortcut {
        id: "services",
        title: "Services",
        description: "Start, stop and configure Windows services.",
        group: TOOLS,
        target: Target::Program("mmc.exe", &["services.msc"]),
    },
    Shortcut {
        id: "system-info",
        title: "System Information",
        description: "Detailed hardware and software report.",
        group: TOOLS,
        target: Target::Program("msinfo32.exe", &[]),
    },
    Shortcut {
        id: "disk-cleanup",
        title: "Disk Cleanup",
        description: "Free up space on a drive.",
        group: TOOLS,
        target: Target::Program("cleanmgr.exe", &[]),
    },
    Shortcut {
        id: "command-prompt",
        title: "Command Prompt",
        description: "Open a command prompt window.",
        group: TOOLS,
        target: Target::Program("cmd.exe", &[]),
    },
    Shortcut {
        id: "sound-devices",
        title: "Sound devices",
        description: "Classic playback and recording devices panel.",
        group: TOOLS,
        target: Target::Program("control.exe", &["mmsys.cpl"]),
    },
    Shortcut {
        id: "volume-mixer",
        title: "Volume Mixer",
        description: "Classic per-app volume mixer.",
        group: TOOLS,
        target: Target::Program("sndvol.exe", &[]),
    },
    Shortcut {
        id: "bluetooth-pair",
        title: "Add a Bluetooth device",
        description: "Windows pairing wizard.",
        group: TOOLS,
        target: Target::Program("DevicePairingWizard.exe", &[]),
    },
    Shortcut {
        id: "bluetooth-files",
        title: "Send or receive files",
        description: "Bluetooth file transfer.",
        group: TOOLS,
        target: Target::Program("fsquirt.exe", &[]),
    },
    Shortcut {
        id: "recycle-bin",
        title: "Recycle Bin",
        description: "Open the Recycle Bin folder.",
        group: FOLDERS,
        target: Target::Program("explorer.exe", &["shell:RecycleBinFolder"]),
    },
    Shortcut {
        id: "temp-folder",
        title: "Temp folder",
        description: "Open your %TEMP% folder.",
        group: FOLDERS,
        target: Target::TempFolder,
    },
    Shortcut {
        id: "startup-folder",
        title: "Startup folder",
        description: "Shortcuts here start when you sign in.",
        group: FOLDERS,
        target: Target::Program("explorer.exe", &["shell:startup"]),
    },
    Shortcut {
        id: "sticky-notes",
        title: "Sticky Notes",
        description: "Microsoft Sticky Notes.",
        group: FOLDERS,
        target: Target::Program(
            "explorer.exe",
            &[r"shell:appsfolder\Microsoft.MicrosoftStickyNotes_8wekyb3d8bbwe!App"],
        ),
    },
    Shortcut {
        id: "notepad",
        title: "Notepad",
        description: "Windows Notepad.",
        group: FOLDERS,
        target: Target::Program("notepad.exe", &[]),
    },
    Shortcut {
        id: "calculator",
        title: "Calculator",
        description: "Windows Calculator.",
        group: FOLDERS,
        target: Target::Uri("calculator:"),
    },
];

pub fn find(id: &str) -> Result<&'static Shortcut> {
    CATALOG.iter().find(|s| s.id == id).ok_or_else(|| Error::NotFound(format!("shortcut {id}")))
}

pub fn list() -> Vec<ShortcutInfo> {
    CATALOG
        .iter()
        .map(|s| ShortcutInfo {
            id: s.id.into(),
            title: s.title.into(),
            description: s.description.into(),
            group: s.group.into(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_unique_and_targets_are_safe() {
        let mut ids: Vec<_> = CATALOG.iter().map(|s| s.id).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), CATALOG.len());
        for s in CATALOG {
            match s.target {
                Target::Uri(uri) => assert!(uri.ends_with(':') || uri.starts_with("ms-settings:"), "{uri}"),
                Target::Program(exe, _) => assert!(!exe.contains('\\') && exe.ends_with(".exe"), "{exe}"),
                Target::TempFolder => {}
            }
        }
        assert!(find("display").is_ok());
        assert!(find("rm -rf").is_err());
    }
}
