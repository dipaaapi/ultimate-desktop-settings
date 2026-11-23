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

const TOOLS: &str = "System Tools";
const FOLDERS: &str = "Folders & Apps";

macro_rules! page {
    ($group:literal, $id:literal, $title:literal, $desc:literal, $uri:literal) => {
        Shortcut { id: $id, title: $title, description: $desc, group: $group, target: Target::Uri($uri) }
    };
}

pub const CATALOG: &[Shortcut] = &[
    page!(
        "System",
        "display",
        "Display",
        "Resolution, scale, brightness and multiple monitors.",
        "ms-settings:display"
    ),
    page!("System", "night-light", "Night light", "Windows' own night light schedule.", "ms-settings:nightlight"),
    page!("System", "hdr", "HDR", "High dynamic range for supported screens.", "ms-settings:display-hdr"),
    page!(
        "System",
        "graphics",
        "Graphics",
        "GPU preference per app and graphics features.",
        "ms-settings:display-advancedgraphics"
    ),
    page!("System", "sound", "Sound", "Output and input devices and system sounds.", "ms-settings:sound"),
    page!(
        "System",
        "app-volume",
        "App volume and device preferences",
        "Pick an output device per app.",
        "ms-settings:apps-volume"
    ),
    page!(
        "System",
        "notifications",
        "Notifications",
        "Notification and Do Not Disturb settings.",
        "ms-settings:notifications"
    ),
    page!("System", "focus", "Focus", "Focus sessions and distraction blocking.", "ms-settings:focus"),
    page!(
        "System",
        "power",
        "Power & battery",
        "Power mode, battery saver and battery usage.",
        "ms-settings:powersleep"
    ),
    page!(
        "System",
        "battery-usage",
        "Battery usage",
        "Which apps used the battery.",
        "ms-settings:batterysaver-usagedetails"
    ),
    page!("System", "storage", "Storage", "Disk space, Storage Sense and cleanup.", "ms-settings:storagesense"),
    page!(
        "System",
        "storage-locations",
        "Where new content is saved",
        "Default drives for apps, documents and media.",
        "ms-settings:savelocations"
    ),
    page!("System", "nearby-sharing", "Nearby sharing", "Share files with nearby PCs.", "ms-settings:crossdevice"),
    page!("System", "multitasking", "Multitasking", "Snap, Alt+Tab and virtual desktops.", "ms-settings:multitasking"),
    page!(
        "System",
        "for-developers",
        "For developers",
        "Developer Mode, sudo and terminal options.",
        "ms-settings:developers"
    ),
    page!("System", "activation", "Activation", "Windows edition and product key.", "ms-settings:activation"),
    page!(
        "System",
        "troubleshoot",
        "Troubleshoot",
        "Windows troubleshooters, including audio.",
        "ms-settings:troubleshoot"
    ),
    page!("System", "recovery", "Recovery", "Reset this PC, go back and advanced startup.", "ms-settings:recovery"),
    page!(
        "System",
        "projecting",
        "Projecting to this PC",
        "Let phones and PCs cast to this screen.",
        "ms-settings:project"
    ),
    page!(
        "System",
        "remote-desktop",
        "Remote Desktop",
        "Allow remote connections to this PC.",
        "ms-settings:remotedesktop"
    ),
    page!("System", "clipboard", "Clipboard", "Clipboard history and sync across devices.", "ms-settings:clipboard"),
    page!("System", "about", "About this PC", "Processor, memory, Windows version and PC name.", "ms-settings:about"),
    page!(
        "Bluetooth & devices",
        "bluetooth",
        "Bluetooth & devices",
        "Pair and manage Bluetooth devices.",
        "ms-settings:bluetooth"
    ),
    page!(
        "Bluetooth & devices",
        "printers",
        "Printers & scanners",
        "Add printers and set the default one.",
        "ms-settings:printers"
    ),
    page!(
        "Bluetooth & devices",
        "phone-link",
        "Mobile devices",
        "Phone Link and using your phone from this PC.",
        "ms-settings:mobile-devices"
    ),
    page!("Bluetooth & devices", "cameras", "Cameras", "Webcam settings and default camera.", "ms-settings:camera"),
    page!(
        "Bluetooth & devices",
        "mouse",
        "Mouse",
        "Buttons, scrolling and pointer options.",
        "ms-settings:mousetouchpad"
    ),
    page!(
        "Bluetooth & devices",
        "touchpad",
        "Touchpad",
        "Gestures, taps and scrolling.",
        "ms-settings:devices-touchpad"
    ),
    page!("Bluetooth & devices", "pen", "Pen & Windows Ink", "Pen buttons and handwriting.", "ms-settings:pen"),
    page!(
        "Bluetooth & devices",
        "autoplay",
        "AutoPlay",
        "What happens when you plug in drives and phones.",
        "ms-settings:autoplay"
    ),
    page!("Bluetooth & devices", "usb", "USB", "USB notifications and battery saver for USB.", "ms-settings:usb"),
    page!(
        "Network & internet",
        "network",
        "Network & internet",
        "Status and network properties.",
        "ms-settings:network"
    ),
    page!(
        "Network & internet",
        "wifi",
        "Wi-Fi",
        "Networks, known networks and hardware properties.",
        "ms-settings:network-wifi"
    ),
    page!(
        "Network & internet",
        "ethernet",
        "Ethernet",
        "Wired connection, IP and DNS.",
        "ms-settings:network-ethernet"
    ),
    page!("Network & internet", "vpn", "VPN", "Add and connect to VPNs.", "ms-settings:network-vpn"),
    page!(
        "Network & internet",
        "hotspot",
        "Mobile hotspot",
        "Share this PC's internet.",
        "ms-settings:network-mobilehotspot"
    ),
    page!(
        "Network & internet",
        "airplane",
        "Airplane mode",
        "Turn all wireless off.",
        "ms-settings:network-airplanemode"
    ),
    page!("Network & internet", "proxy", "Proxy", "Proxy server settings.", "ms-settings:network-proxy"),
    page!(
        "Network & internet",
        "data-usage",
        "Data usage",
        "Data used per app and data limits.",
        "ms-settings:datausage"
    ),
    page!(
        "Network & internet",
        "advanced-network",
        "Advanced network settings",
        "Adapters, network reset and sharing.",
        "ms-settings:network-advancedsettings"
    ),
    page!(
        "Personalization",
        "personalization",
        "Personalization",
        "Background, colors, themes and lock screen.",
        "ms-settings:personalization"
    ),
    page!(
        "Personalization",
        "background",
        "Background",
        "Desktop picture, slideshow or solid color.",
        "ms-settings:personalization-background"
    ),
    page!("Personalization", "colors", "Colors", "Light or dark mode and accent color.", "ms-settings:colors"),
    page!("Personalization", "themes", "Themes", "Themes, desktop icons and mouse pointers.", "ms-settings:themes"),
    page!("Personalization", "lock-screen", "Lock screen", "Lock screen picture and status.", "ms-settings:lockscreen"),
    page!(
        "Personalization",
        "text-input",
        "Text input",
        "Touch keyboard and emoji panel themes.",
        "ms-settings:personalization-textinput"
    ),
    page!(
        "Personalization",
        "start",
        "Start",
        "Start layout, recommendations and folders.",
        "ms-settings:personalization-start"
    ),
    page!("Personalization", "taskbar", "Taskbar", "Taskbar items, behaviors and alignment.", "ms-settings:taskbar"),
    page!("Personalization", "fonts", "Fonts", "Installed fonts and adding new ones.", "ms-settings:fonts"),
    page!(
        "Personalization",
        "device-usage",
        "Device usage",
        "Tell Windows what you use this PC for.",
        "ms-settings:deviceusage"
    ),
    page!("Apps", "apps", "Installed apps", "Modify or uninstall programs.", "ms-settings:appsfeatures"),
    page!(
        "Apps",
        "advanced-apps",
        "Advanced app settings",
        "Where to get apps, app archiving and sharing.",
        "ms-settings:advanced-apps"
    ),
    page!(
        "Apps",
        "default-apps",
        "Default apps",
        "Choose the default browser and other apps.",
        "ms-settings:defaultapps"
    ),
    page!("Apps", "offline-maps", "Offline maps", "Download maps for offline use.", "ms-settings:maps"),
    page!(
        "Apps",
        "optional-features",
        "Optional features",
        "Add or remove Windows features.",
        "ms-settings:optionalfeatures"
    ),
    page!("Apps", "websites-apps", "Apps for websites", "Links that open in apps.", "ms-settings:appsforwebsites"),
    page!(
        "Apps",
        "video-playback",
        "Video playback",
        "HDR video and battery options for video.",
        "ms-settings:videoplayback"
    ),
    page!("Apps", "startup-apps", "Startup apps", "Windows' own startup apps page.", "ms-settings:startupapps"),
    page!("Accounts", "your-info", "Your info", "Your account and picture.", "ms-settings:yourinfo"),
    page!(
        "Accounts",
        "sign-in",
        "Sign-in options",
        "PIN, Windows Hello, password and dynamic lock.",
        "ms-settings:signinoptions"
    ),
    page!(
        "Accounts",
        "email-accounts",
        "Email & accounts",
        "Accounts used by Mail, Calendar and other apps.",
        "ms-settings:emailandaccounts"
    ),
    page!("Accounts", "family", "Family & other users", "Add people and set up a kiosk.", "ms-settings:otherusers"),
    page!("Accounts", "backup", "Windows Backup", "Back up folders, apps and settings.", "ms-settings:backup"),
    page!(
        "Accounts",
        "work-access",
        "Access work or school",
        "Connect to a work or school account.",
        "ms-settings:workplace"
    ),
    page!(
        "Time & language",
        "date-time",
        "Date & time",
        "Time zone, clock sync and formats.",
        "ms-settings:dateandtime"
    ),
    page!(
        "Time & language",
        "language",
        "Language & region",
        "Display language, keyboards and regional format.",
        "ms-settings:regionlanguage"
    ),
    page!("Time & language", "typing", "Typing", "Autocorrect, suggestions and touch keyboard.", "ms-settings:typing"),
    page!("Time & language", "speech", "Speech", "Voice typing language and voices.", "ms-settings:speech"),
    page!(
        "Gaming",
        "game-bar",
        "Game Bar",
        "Xbox Game Bar shortcuts and controller button.",
        "ms-settings:gaming-gamebar"
    ),
    page!(
        "Gaming",
        "captures",
        "Captures",
        "Where and how clips and screenshots are saved.",
        "ms-settings:gaming-gamedvr"
    ),
    page!("Gaming", "game-mode", "Game Mode", "Prioritize games while you play.", "ms-settings:gaming-gamemode"),
    page!(
        "Accessibility",
        "text-size",
        "Text size",
        "Make text bigger everywhere.",
        "ms-settings:easeofaccess-display"
    ),
    page!(
        "Accessibility",
        "visual-effects",
        "Visual effects",
        "Scrollbars, transparency and animations.",
        "ms-settings:easeofaccess-visualeffects"
    ),
    page!(
        "Accessibility",
        "pointer",
        "Mouse pointer and touch",
        "Pointer size, color and touch indicator.",
        "ms-settings:easeofaccess-mousepointer"
    ),
    page!(
        "Accessibility",
        "text-cursor",
        "Text cursor",
        "Cursor indicator and thickness.",
        "ms-settings:easeofaccess-cursor"
    ),
    page!(
        "Accessibility",
        "magnifier",
        "Magnifier",
        "Zoom in on parts of the screen.",
        "ms-settings:easeofaccess-magnifier"
    ),
    page!(
        "Accessibility",
        "color-filters",
        "Color filters",
        "Color-blind friendly filters.",
        "ms-settings:easeofaccess-colorfilter"
    ),
    page!(
        "Accessibility",
        "contrast-themes",
        "Contrast themes",
        "High contrast colors.",
        "ms-settings:easeofaccess-highcontrast"
    ),
    page!("Accessibility", "narrator", "Narrator", "Screen reader.", "ms-settings:easeofaccess-narrator"),
    page!(
        "Accessibility",
        "audio-a11y",
        "Audio",
        "Mono audio and flashing for alerts.",
        "ms-settings:easeofaccess-audio"
    ),
    page!(
        "Accessibility",
        "captions",
        "Captions",
        "Live captions and caption style.",
        "ms-settings:easeofaccess-closedcaptioning"
    ),
    page!(
        "Accessibility",
        "keyboard-a11y",
        "Keyboard",
        "Sticky, Filter and Toggle keys, on-screen keyboard.",
        "ms-settings:easeofaccess-keyboard"
    ),
    page!(
        "Accessibility",
        "eye-control",
        "Eye control",
        "Control the PC with your eyes.",
        "ms-settings:easeofaccess-eyecontrol"
    ),
    page!(
        "Privacy & security",
        "privacy",
        "Privacy & security",
        "App permissions, Windows Security and firewall.",
        "ms-settings:privacy"
    ),
    page!(
        "Privacy & security",
        "windows-security",
        "Windows Security",
        "Virus protection, firewall and device security.",
        "ms-settings:windowsdefender"
    ),
    page!(
        "Privacy & security",
        "find-my-device",
        "Find my device",
        "Locate this PC if it's lost.",
        "ms-settings:findmydevice"
    ),
    page!(
        "Privacy & security",
        "device-encryption",
        "Device encryption",
        "Encrypt this PC's drive.",
        "ms-settings:deviceencryption"
    ),
    page!(
        "Privacy & security",
        "diagnostics",
        "Diagnostics & feedback",
        "Diagnostic data and feedback frequency.",
        "ms-settings:privacy-feedback"
    ),
    page!(
        "Privacy & security",
        "activity-history",
        "Activity history",
        "What Windows remembers about your activity.",
        "ms-settings:privacy-activityhistory"
    ),
    page!(
        "Privacy & security",
        "search-permissions",
        "Search permissions",
        "SafeSearch, cloud search and history.",
        "ms-settings:search-permissions"
    ),
    page!(
        "Privacy & security",
        "searching-windows",
        "Searching Windows",
        "Indexing locations and excluded folders.",
        "ms-settings:cortana-windowssearch"
    ),
    page!(
        "Windows Update",
        "windows-update",
        "Windows Update",
        "Updates, update history and restarts.",
        "ms-settings:windowsupdate"
    ),
    page!(
        "Windows Update",
        "update-history",
        "Update history",
        "Installed updates and uninstalling them.",
        "ms-settings:windowsupdate-history"
    ),
    page!(
        "Windows Update",
        "update-options",
        "Advanced update options",
        "Active hours, optional updates and delivery.",
        "ms-settings:windowsupdate-options"
    ),
    page!("Windows Update", "insider", "Windows Insider Program", "Preview builds.", "ms-settings:windowsinsider"),
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
        assert!(CATALOG.len() > 100, "every Settings page is one click away");
        assert!(find("rm -rf").is_err());
    }
}
