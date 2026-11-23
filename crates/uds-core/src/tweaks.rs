//! Windows desktop settings that are plain per-user registry values: theme,
//! taskbar, Start, File Explorer, multitasking, notifications, privacy and
//! more. Every value lives under `HKEY_CURRENT_USER`, so no admin rights are
//! needed.
//!
//! Only the values in [`CATALOG`] can be changed, so the UI (and any future
//! assistant) can never write an arbitrary registry path.

use serde::Serialize;

use crate::registry::{Hive, RegValue, Registry};
use crate::{Error, Result};

const PERSONALIZE: &str = r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize";
const DWM: &str = r"Software\Microsoft\Windows\DWM";
const DESKTOP: &str = r"Control Panel\Desktop";
const EXPLORER: &str = r"Software\Microsoft\Windows\CurrentVersion\Explorer";
const ADVANCED: &str = r"Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced";
const CABINET: &str = r"Software\Microsoft\Windows\CurrentVersion\Explorer\CabinetState";
const HIDE_ICONS: &str = r"Software\Microsoft\Windows\CurrentVersion\Explorer\HideDesktopIcons\NewStartPanel";
const DEV_SETTINGS: &str = r"Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced\TaskbarDeveloperSettings";
const SEARCH: &str = r"Software\Microsoft\Windows\CurrentVersion\Search";
const SEARCH_SETTINGS: &str = r"Software\Microsoft\Windows\CurrentVersion\SearchSettings";
const CDM: &str = r"Software\Microsoft\Windows\CurrentVersion\ContentDeliveryManager";
const ENGAGEMENT: &str = r"Software\Microsoft\Windows\CurrentVersion\UserProfileEngagement";
const PUSH: &str = r"Software\Microsoft\Windows\CurrentVersion\PushNotifications";
const NOTIFICATIONS: &str = r"Software\Microsoft\Windows\CurrentVersion\Notifications\Settings";
const ADVERTISING: &str = r"Software\Microsoft\Windows\CurrentVersion\AdvertisingInfo";
const PRIVACY: &str = r"Software\Microsoft\Windows\CurrentVersion\Privacy";
const SPEECH: &str = r"Software\Microsoft\Speech_OneCore\Settings\OnlineSpeechPrivacy";
const LANGUAGE_PROFILE: &str = r"Control Panel\International\User Profile";
const INTERNATIONAL: &str = r"Control Panel\International";
const CLIPBOARD: &str = r"Software\Microsoft\Clipboard";
const GAME_BAR: &str = r"Software\Microsoft\GameBar";
const GAME_DVR: &str = r"Software\Microsoft\Windows\CurrentVersion\GameDVR";
const STORAGE_SENSE: &str = r"Software\Microsoft\Windows\CurrentVersion\StorageSense\Parameters\StoragePolicy";
#[cfg(test)]
const CONSENT: &str = r"Software\Microsoft\Windows\CurrentVersion\CapabilityAccessManager\ConsentStore";
/// The Windows 11 context menu's COM class. An empty override brings back the full classic menu.
const CLASSIC_MENU: &str = r"Software\Classes\CLSID\{86ca1aa0-34aa-4e8b-a509-50c905bae2a2}\InprocServer32";

/// What Windows needs to hear about after a value changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Refresh {
    /// Nothing; Windows reads the value when it next needs it.
    None,
    /// Broadcast `ImmersiveColorSet` so apps repaint in the new theme.
    Theme,
    /// Tell Explorer windows to re-read their view settings.
    Explorer,
    /// Tell the taskbar to re-read its settings.
    Taskbar,
    /// Broadcast `intl` so clocks and dates use the new format.
    International,
    /// Apply the wallpaper again so its new fit shows.
    Wallpaper,
    /// Explorer only reads this when it starts; the UI offers a restart.
    ExplorerRestart,
}

/// A raw registry value one option stands for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Raw {
    Dword(u32),
    Str(&'static str),
    /// The key exists with an empty default value.
    KeyPresent,
    /// The key does not exist.
    KeyAbsent,
}

#[derive(Debug, Clone, Copy)]
pub struct Opt {
    pub label: &'static str,
    pub raw: Raw,
    /// Other values in the same key written along with this option.
    pub also: &'static [(&'static str, Raw)],
}

/// Which board a setting shows on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Page {
    Look,
    Taskbar,
    Explorer,
    System,
    Privacy,
    Input,
    Power,
}

#[derive(Debug, Clone, Copy)]
pub struct TweakDef {
    pub id: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    pub page: Page,
    pub group: &'static str,
    pub path: &'static str,
    pub value: &'static str,
    /// For toggles: `[off, on]`.
    pub options: &'static [Opt],
    /// Option Windows uses when the value doesn't exist yet.
    pub default: usize,
    pub toggle: bool,
    pub refresh: Refresh,
}

const fn o(label: &'static str, raw: Raw) -> Opt {
    Opt { label, raw, also: &[] }
}

macro_rules! toggle {
    ($id:literal, $page:ident, $group:literal, $title:literal, $desc:literal, $path:expr, $value:literal,
     on: $on:expr, off: $off:expr, default_on: $def:literal, $refresh:ident) => {
        TweakDef {
            id: $id,
            title: $title,
            description: $desc,
            page: Page::$page,
            group: $group,
            path: $path,
            value: $value,
            options: &[o("Off", $off), o("On", $on)],
            default: if $def { 1 } else { 0 },
            toggle: true,
            refresh: Refresh::$refresh,
        }
    };
}

/// A 1 = on, 0 = off DWORD toggle.
macro_rules! flag {
    ($id:literal, $page:ident, $group:literal, $title:literal, $desc:literal, $path:expr, $value:literal,
     default_on: $def:literal, $refresh:ident) => {
        toggle!($id, $page, $group, $title, $desc, $path, $value, on: D(1), off: D(0), default_on: $def, $refresh)
    };
}

macro_rules! choice {
    ($id:literal, $page:ident, $group:literal, $title:literal, $desc:literal, $path:expr, $value:literal,
     [$($opt:expr),+ $(,)?], default: $def:literal, $refresh:ident) => {
        TweakDef {
            id: $id,
            title: $title,
            description: $desc,
            page: Page::$page,
            group: $group,
            path: $path,
            value: $value,
            options: &[$($opt),+],
            default: $def,
            toggle: false,
            refresh: Refresh::$refresh,
        }
    };
}

/// "Let apps access ..." in Settings > Privacy & security.
macro_rules! permission {
    ($id:literal, $cap:literal, $title:literal, $desc:literal) => {
        toggle!($id, Privacy, "App permissions", $title, $desc, concat!(
            r"Software\Microsoft\Windows\CurrentVersion\CapabilityAccessManager\ConsentStore\", $cap),
            "Value", on: S("Allow"), off: S("Deny"), default_on: true, None)
    };
}

use Raw::{Dword as D, Str as S};

const CDM_SETTINGS_SUGGESTIONS: &[(&str, Raw)] =
    &[("SubscribedContent-353694Enabled", D(0)), ("SubscribedContent-353696Enabled", D(0))];
const CDM_SETTINGS_SUGGESTIONS_ON: &[(&str, Raw)] =
    &[("SubscribedContent-353694Enabled", D(1)), ("SubscribedContent-353696Enabled", D(1))];

pub const CATALOG: &[TweakDef] = &[
    // ---- Look: theme, colors, desktop ----
    toggle!("apps-dark", Look, "Theme", "Dark mode for apps", "Apps that follow the Windows theme use dark colors.",
        PERSONALIZE, "AppsUseLightTheme", on: D(0), off: D(1), default_on: false, Theme),
    toggle!("system-dark", Look, "Theme", "Dark mode for Windows", "Taskbar, Start menu and notifications use dark colors.",
        PERSONALIZE, "SystemUsesLightTheme", on: D(0), off: D(1), default_on: false, Theme),
    flag!("transparency", Look, "Theme", "Transparency effects", "Windows and surfaces appear slightly see-through.",
        PERSONALIZE, "EnableTransparency", default_on: true, Theme),
    flag!("accent-start", Look, "Theme", "Accent color on Start and taskbar", "Tints Start, the taskbar and the action center with your accent color (dark mode only).",
        PERSONALIZE, "ColorPrevalence", default_on: false, Theme),
    flag!("accent-titlebars", Look, "Theme", "Accent color on title bars", "Window title bars and borders use your accent color.",
        DWM, "ColorPrevalence", default_on: false, Theme),
    choice!("wallpaper-fit", Look, "Desktop", "Wallpaper fit", "How the desktop picture fills your screen.",
        DESKTOP, "WallpaperStyle",
        [o("Fill", S("10")), o("Fit", S("6")), o("Stretch", S("2")), o("Center", S("0")), o("Span", S("22"))],
        default: 0, Wallpaper),
    toggle!("desktop-icons", Look, "Desktop", "Show desktop icons", "Hides or shows every icon on the desktop.",
        ADVANCED, "HideIcons", on: D(0), off: D(1), default_on: true, Explorer),
    toggle!("icon-this-pc", Look, "Desktop", "This PC on the desktop", "Shows the This PC icon.",
        HIDE_ICONS, "{20D04FE0-3AEA-1069-A2D8-08002B30309D}", on: D(0), off: D(1), default_on: false, Explorer),
    toggle!("icon-recycle-bin", Look, "Desktop", "Recycle Bin on the desktop", "Shows the Recycle Bin icon.",
        HIDE_ICONS, "{645FF040-5081-101B-9F08-00AA002F954E}", on: D(0), off: D(1), default_on: true, Explorer),
    toggle!("icon-user-files", Look, "Desktop", "Your user folder on the desktop", "Shows the folder with your name.",
        HIDE_ICONS, "{59031a47-3f72-44a7-89c5-5595fe6b30ee}", on: D(0), off: D(1), default_on: false, Explorer),
    toggle!("icon-network", Look, "Desktop", "Network on the desktop", "Shows the Network icon.",
        HIDE_ICONS, "{F02C1A0D-BE21-4350-88B0-7367FC96EF3C}", on: D(0), off: D(1), default_on: false, Explorer),
    toggle!("icon-control-panel", Look, "Desktop", "Control Panel on the desktop", "Shows the Control Panel icon.",
        HIDE_ICONS, "{5399E694-6CE5-4D6C-8FCE-1D8870FDCBA0}", on: D(0), off: D(1), default_on: false, Explorer),
    toggle!("scrollbars-always", Look, "Effects", "Always show scrollbars", "Scrollbars stay visible instead of hiding when idle.",
        r"Control Panel\Accessibility", "DynamicScrollbars", on: D(0), off: D(1), default_on: false, None),
    // ---- Taskbar & Start ----
    toggle!("taskbar-left", Taskbar, "Taskbar", "Align taskbar to the left", "Start and pinned apps sit on the left like Windows 10.",
        ADVANCED, "TaskbarAl", on: D(0), off: D(1), default_on: false, Taskbar),
    choice!("taskbar-search", Taskbar, "Taskbar", "Search on the taskbar", "How the search box shows on the taskbar.",
        SEARCH, "SearchboxTaskbarMode",
        [o("Hidden", D(0)), o("Icon", D(1)), o("Icon and label", D(3)), o("Search box", D(2))],
        default: 3, Taskbar),
    flag!("task-view-button", Taskbar, "Taskbar", "Task View button", "Shows the Task View button on the taskbar.",
        ADVANCED, "ShowTaskViewButton", default_on: true, Taskbar),
    flag!("copilot-button", Taskbar, "Taskbar", "Copilot button", "Shows the Copilot button on builds that have it.",
        ADVANCED, "ShowCopilotButton", default_on: false, Taskbar),
    choice!("taskbar-combine", Taskbar, "Taskbar", "Combine taskbar buttons", "Group windows of the same app into one button.",
        ADVANCED, "TaskbarGlomLevel",
        [o("Always", D(0)), o("When full", D(1)), o("Never", D(2))],
        default: 0, Taskbar),
    flag!("taskbar-badges", Taskbar, "Taskbar", "Badges on taskbar apps", "Unread counters on app icons.",
        ADVANCED, "TaskbarBadges", default_on: true, Taskbar),
    flag!("taskbar-flashing", Taskbar, "Taskbar", "Flashing taskbar apps", "Apps can flash their button to get your attention.",
        ADVANCED, "TaskbarFlashing", default_on: true, Taskbar),
    flag!("taskbar-all-displays", Taskbar, "Taskbar", "Taskbar on all displays", "Shows a taskbar on every monitor.",
        ADVANCED, "MMTaskbarEnabled", default_on: false, Taskbar),
    flag!("taskbar-show-desktop", Taskbar, "Taskbar", "Far corner shows the desktop", "Clicking the far right corner of the taskbar shows the desktop.",
        ADVANCED, "TaskbarSd", default_on: true, Taskbar),
    flag!("clock-seconds", Taskbar, "Taskbar", "Show seconds in the clock", "The taskbar clock shows seconds (Windows 11).",
        ADVANCED, "ShowSecondsInSystemClock", default_on: false, Taskbar),
    flag!("end-task", Taskbar, "Taskbar", "End task on right-click", "Right-clicking a taskbar app offers End task (Windows 11).",
        DEV_SETTINGS, "TaskbarEndTask", default_on: false, Taskbar),
    choice!("start-layout", Taskbar, "Start menu", "Start layout", "More room for pinned apps or for recommendations.",
        ADVANCED, "Start_Layout",
        [o("Default", D(0)), o("More pins", D(1)), o("More recommendations", D(2))],
        default: 0, Taskbar),
    flag!("start-most-used", Taskbar, "Start menu", "Show most used apps", "Start lists the apps you open most.",
        ADVANCED, "Start_TrackProgs", default_on: true, Taskbar),
    flag!("start-recent", Taskbar, "Start menu", "Show recent files", "Recently opened files in Start, Jump Lists and File Explorer.",
        ADVANCED, "Start_TrackDocs", default_on: true, Taskbar),
    flag!("start-recommendations", Taskbar, "Start menu", "Recommendations in Start", "Tips, app promotions and new apps in Start.",
        ADVANCED, "Start_IrisRecommendations", default_on: true, Taskbar),
    flag!("start-account", Taskbar, "Start menu", "Account notifications", "Account reminders on your picture in Start.",
        ADVANCED, "Start_AccountNotifications", default_on: true, Taskbar),
    // ---- File Explorer ----
    toggle!("file-extensions", Explorer, "View", "Show file extensions", "File Explorer shows endings like .exe, .pdf and .txt.",
        ADVANCED, "HideFileExt", on: D(0), off: D(1), default_on: false, Explorer),
    toggle!("hidden-files", Explorer, "View", "Show hidden files", "File Explorer shows files and folders marked hidden.",
        ADVANCED, "Hidden", on: D(1), off: D(2), default_on: false, Explorer),
    flag!("system-files", Explorer, "View", "Show protected system files", "Also shows Windows' own protected files. Leave off unless you need it.",
        ADVANCED, "ShowSuperHidden", default_on: false, Explorer),
    flag!("compact-view", Explorer, "View", "Compact view", "Tighter spacing between files in File Explorer.",
        ADVANCED, "UseCompactMode", default_on: false, Explorer),
    flag!("item-checkboxes", Explorer, "View", "Item check boxes", "A check box on each file for selecting several.",
        ADVANCED, "AutoCheckSelect", default_on: false, Explorer),
    flag!("full-path", Explorer, "View", "Full path in the title bar", "Shows the whole folder path at the top of the window.",
        CABINET, "FullPath", default_on: false, Explorer),
    flag!("status-bar", Explorer, "View", "Status bar", "Item counts and sizes at the bottom of the window.",
        ADVANCED, "ShowStatusBar", default_on: true, Explorer),
    toggle!("thumbnails", Explorer, "View", "Show thumbnails", "Pictures and videos show a preview instead of an icon.",
        ADVANCED, "IconsOnly", on: D(0), off: D(1), default_on: true, Explorer),
    flag!("info-tips", Explorer, "View", "Pop-up descriptions", "Hovering a file shows its details.",
        ADVANCED, "ShowInfoTip", default_on: true, Explorer),
    flag!("preview-handlers", Explorer, "View", "Previews in the preview pane", "The preview pane can show file contents.",
        ADVANCED, "ShowPreviewHandlers", default_on: true, Explorer),
    flag!("sync-notifications", Explorer, "View", "Sync provider notifications", "OneDrive and other cloud tips inside File Explorer.",
        ADVANCED, "ShowSyncProviderNotifications", default_on: true, Explorer),
    toggle!("open-this-pc", Explorer, "Navigation", "Open File Explorer to This PC", "New Explorer windows start at This PC instead of Home.",
        ADVANCED, "LaunchTo", on: D(1), off: D(2), default_on: false, Explorer),
    flag!("quick-access-recent", Explorer, "Navigation", "Recent files in Home", "Home lists the files you opened recently.",
        EXPLORER, "ShowRecent", default_on: true, Explorer),
    flag!("quick-access-frequent", Explorer, "Navigation", "Frequent folders in Home", "Home lists the folders you use most.",
        EXPLORER, "ShowFrequent", default_on: true, Explorer),
    flag!("quick-access-cloud", Explorer, "Navigation", "Files from Office.com in Home", "Home also lists your cloud documents.",
        EXPLORER, "ShowCloudFilesInQuickAccess", default_on: true, Explorer),
    flag!("nav-expand", Explorer, "Navigation", "Expand to the open folder", "The navigation pane follows the folder you are in.",
        ADVANCED, "NavPaneExpandToCurrentFolder", default_on: false, Explorer),
    flag!("nav-all-folders", Explorer, "Navigation", "Show all folders", "The navigation pane also shows Desktop, Control Panel and Recycle Bin.",
        ADVANCED, "NavPaneShowAllFolders", default_on: false, Explorer),
    flag!("separate-process", Explorer, "Navigation", "Open folders in a separate process", "A crashed folder window can't take the taskbar down with it.",
        ADVANCED, "SeparateProcess", default_on: false, ExplorerRestart),
    flag!("restore-folders", Explorer, "Navigation", "Reopen folders at sign-in", "Folder windows that were open come back when you sign in.",
        ADVANCED, "PersistBrowsers", default_on: false, None),
    toggle!("classic-context-menu", Explorer, "Navigation", "Classic right-click menu", "Shows the full Windows 10 menu instead of \"Show more options\" (Windows 11).",
        CLASSIC_MENU, "", on: Raw::KeyPresent, off: Raw::KeyAbsent, default_on: false, ExplorerRestart),
    // ---- System: multitasking, notifications, clipboard, gaming, storage, formats ----
    flag!("snap-layouts", System, "Multitasking", "Snap layouts on the maximize button", "Hovering the maximize button offers window layouts.",
        ADVANCED, "EnableSnapAssistFlyout", default_on: true, None),
    flag!("snap-assist", System, "Multitasking", "Suggest what to snap next", "After snapping a window, pick another to fill the space.",
        ADVANCED, "SnapAssist", default_on: true, None),
    flag!("snap-bar", System, "Multitasking", "Snap bar at the top", "Dragging a window to the top of the screen shows layouts.",
        ADVANCED, "EnableSnapBar", default_on: true, None),
    toggle!("title-shake", System, "Multitasking", "Shake a title bar to minimize others", "Shake a window to minimize every other window.",
        ADVANCED, "DisallowShaking", on: D(0), off: D(1), default_on: false, None),
    choice!("desktops-taskbar", System, "Multitasking", "Taskbar shows windows from", "With several virtual desktops.",
        ADVANCED, "VirtualDesktopTaskbarFilter",
        [o("This desktop", D(1)), o("All desktops", D(0))], default: 0, Taskbar),
    choice!("desktops-alttab", System, "Multitasking", "Alt+Tab shows windows from", "With several virtual desktops.",
        ADVANCED, "VirtualDesktopAltTabFilter",
        [o("This desktop", D(1)), o("All desktops", D(0))], default: 0, None),
    flag!("notifications", System, "Notifications", "Notifications", "Apps and Windows can show notifications.",
        PUSH, "ToastEnabled", default_on: true, None),
    flag!("notifications-lock", System, "Notifications", "Notifications on the lock screen", "Show notifications while the PC is locked.",
        NOTIFICATIONS, "NOC_GLOBAL_SETTING_ALLOW_TOASTS_ABOVE_LOCK", default_on: true, None),
    flag!("notifications-sound", System, "Notifications", "Notification sounds", "Notifications can play a sound.",
        NOTIFICATIONS, "NOC_GLOBAL_SETTING_ALLOW_NOTIFICATION_SOUND", default_on: true, None),
    flag!("clipboard-history", System, "Clipboard", "Clipboard history", "Win+V keeps the last things you copied.",
        CLIPBOARD, "EnableClipboardHistory", default_on: false, None),
    flag!("game-mode", System, "Gaming", "Game Mode", "Windows puts games first and pauses background work while you play.",
        GAME_BAR, "AutoGameModeEnabled", default_on: true, None),
    flag!("game-bar-controller", System, "Gaming", "Open Game Bar with a controller", "The Xbox button opens Game Bar.",
        GAME_BAR, "UseNexusForGameBarEnabled", default_on: true, None),
    flag!("game-capture", System, "Gaming", "Background game recording", "Game Bar can record clips of what just happened. Costs some performance.",
        GAME_DVR, "AppCaptureEnabled", default_on: false, None),
    flag!("storage-sense", System, "Storage", "Storage Sense", "Windows frees space automatically by clearing temp files and the Recycle Bin.",
        STORAGE_SENSE, "01", default_on: false, None),
    TweakDef {
        id: "clock-format",
        title: "Clock format",
        description: "12-hour or 24-hour time in the taskbar and apps.",
        page: Page::System,
        group: "Date & time",
        path: INTERNATIONAL,
        value: "sShortTime",
        options: &[
            Opt { label: "12-hour", raw: S("h:mm tt"), also: &[("sTimeFormat", S("h:mm:ss tt"))] },
            Opt { label: "24-hour", raw: S("HH:mm"), also: &[("sTimeFormat", S("HH:mm:ss"))] },
        ],
        default: 0,
        toggle: false,
        refresh: Refresh::International,
    },
    choice!("date-format", System, "Date & time", "Short date format", "How dates look in the taskbar and File Explorer.",
        INTERNATIONAL, "sShortDate",
        [o("12/31/2026", S("M/d/yyyy")), o("31/12/2026", S("dd/MM/yyyy")), o("2026-12-31", S("yyyy-MM-dd"))],
        default: 0, International),
    choice!("first-day", System, "Date & time", "First day of the week", "Used by calendars.",
        INTERNATIONAL, "iFirstDayOfWeek",
        [o("Monday", S("0")), o("Saturday", S("5")), o("Sunday", S("6"))],
        default: 0, International),
    // ---- Privacy & search ----
    flag!("advertising-id", Privacy, "General", "Personalized ads", "Apps can use your advertising ID.",
        ADVERTISING, "Enabled", default_on: true, None),
    flag!("tailored-experiences", Privacy, "General", "Tailored experiences", "Microsoft uses diagnostic data for tips and offers.",
        PRIVACY, "TailoredExperiencesWithDiagnosticDataEnabled", default_on: true, None),
    toggle!("language-list", Privacy, "General", "Let websites see your languages", "Sites can show local content using your language list.",
        LANGUAGE_PROFILE, "HttpAcceptLanguageOptOut", on: D(0), off: D(1), default_on: true, None),
    flag!("online-speech", Privacy, "General", "Online speech recognition", "Voice typing and dictation use Microsoft's cloud.",
        SPEECH, "HasAccepted", default_on: false, None),
    TweakDef {
        id: "settings-suggestions",
        title: "Suggested content in Settings",
        description: "Settings shows tips and app promotions.",
        page: Page::Privacy,
        group: "Tips & ads",
        path: CDM,
        value: "SubscribedContent-338393Enabled",
        options: &[
            Opt { label: "Off", raw: D(0), also: CDM_SETTINGS_SUGGESTIONS },
            Opt { label: "On", raw: D(1), also: CDM_SETTINGS_SUGGESTIONS_ON },
        ],
        default: 1,
        toggle: true,
        refresh: Refresh::None,
    },
    flag!("windows-tips", Privacy, "Tips & ads", "Tips and suggestions", "Windows shows tips while you use it.",
        CDM, "SubscribedContent-338389Enabled", default_on: true, None),
    flag!("start-suggestions", Privacy, "Tips & ads", "Suggestions in Start", "App promotions in the Start menu.",
        CDM, "SystemPaneSuggestionsEnabled", default_on: true, None),
    TweakDef {
        id: "lock-screen-tips",
        title: "Fun facts and tips on the lock screen",
        description: "Text and promotions over the lock screen picture.",
        page: Page::Privacy,
        group: "Tips & ads",
        path: CDM,
        value: "RotatingLockScreenOverlayEnabled",
        options: &[
            Opt { label: "Off", raw: D(0), also: &[("SubscribedContent-338387Enabled", D(0))] },
            Opt { label: "On", raw: D(1), also: &[("SubscribedContent-338387Enabled", D(1))] },
        ],
        default: 1,
        toggle: true,
        refresh: Refresh::None,
    },
    flag!("welcome-experience", Privacy, "Tips & ads", "Welcome experience after updates", "Shows what's new after Windows updates.",
        CDM, "SubscribedContent-310093Enabled", default_on: true, None),
    flag!("finish-setup", Privacy, "Tips & ads", "Suggest ways to finish setting up", "Full-screen setup reminders.",
        ENGAGEMENT, "ScoobeSystemSettingEnabled", default_on: true, None),
    flag!("silent-app-installs", Privacy, "Tips & ads", "Automatically install suggested apps", "Windows may install promoted apps on its own.",
        CDM, "SilentInstalledAppsEnabled", default_on: true, None),
    flag!("web-search", Privacy, "Search", "Web results in Start search", "Search in Start also searches Bing.",
        SEARCH, "BingSearchEnabled", default_on: true, None),
    flag!("search-highlights", Privacy, "Search", "Search highlights", "Daily pictures and trending topics in search.",
        SEARCH_SETTINGS, "IsDynamicSearchBoxEnabled", default_on: true, None),
    flag!("search-history", Privacy, "Search", "Search history on this device", "Search remembers what you searched for.",
        SEARCH_SETTINGS, "IsDeviceSearchHistoryEnabled", default_on: true, None),
    flag!("search-cloud-personal", Privacy, "Search", "Search your Microsoft account", "Results from OneDrive and Outlook.",
        SEARCH_SETTINGS, "IsMSACloudSearchEnabled", default_on: true, None),
    flag!("search-cloud-work", Privacy, "Search", "Search your work or school account", "Results from SharePoint and work OneDrive.",
        SEARCH_SETTINGS, "IsAADCloudSearchEnabled", default_on: true, None),
    permission!("perm-location", "location", "Location", "Apps can use your location."),
    permission!("perm-camera", "webcam", "Camera", "Apps can use your camera."),
    permission!("perm-microphone", "microphone", "Microphone", "Apps can use your microphone."),
    permission!("perm-contacts", "contacts", "Contacts", "Apps can read your contacts."),
    permission!("perm-calendar", "appointments", "Calendar", "Apps can read your calendar."),
    permission!("perm-email", "email", "Email", "Apps can read your email."),
    permission!("perm-messaging", "chat", "Messaging", "Apps can read text messages."),
    permission!("perm-call-history", "phoneCallHistory", "Call history", "Apps can read your call history."),
    permission!("perm-account-info", "userAccountInformation", "Account info", "Apps can see your name and picture."),
    permission!("perm-documents", "documentsLibrary", "Documents", "Apps can open your Documents folder."),
    permission!("perm-pictures", "picturesLibrary", "Pictures", "Apps can open your Pictures folder."),
    permission!("perm-videos", "videosLibrary", "Videos", "Apps can open your Videos folder."),
    permission!("perm-files", "broadFileSystemAccess", "File system", "Apps can open any of your files."),
    permission!("perm-radios", "radios", "Radios", "Apps can turn Bluetooth and Wi-Fi radios on and off."),
    permission!("perm-diagnostics", "appDiagnostic", "App diagnostics", "Apps can see what other apps are running."),
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TweakState {
    pub id: String,
    pub title: String,
    pub description: String,
    pub page: Page,
    pub group: String,
    /// A switch (options are `[off, on]`) or a row of choices.
    pub toggle: bool,
    pub options: Vec<String>,
    /// The option in effect, or `None` when Windows holds a value this app
    /// doesn't offer (such as a custom date format).
    pub selected: Option<usize>,
    /// For toggles: the switch position.
    pub enabled: bool,
    pub needs_restart: bool,
    /// For sliders: the allowed range and current value.
    pub range: Option<RangeSpec>,
    pub value: Option<i32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RangeSpec {
    pub min: i32,
    pub max: i32,
    pub step: i32,
    pub unit: &'static str,
}

pub fn find(id: &str) -> Result<&'static TweakDef> {
    CATALOG.iter().find(|t| t.id == id).ok_or_else(|| Error::NotFound(format!("setting {id}")))
}

fn parent(path: &str) -> &str {
    path.rsplit_once('\\').map(|(p, _)| p).unwrap_or(path)
}

fn matches(current: &Option<RegValue>, raw: Raw) -> bool {
    match (current, raw) {
        (Some(RegValue::Dword(v)), Raw::Dword(want)) => *v == want,
        (Some(value), Raw::Str(want)) => value.as_str().is_some_and(|s| s.eq_ignore_ascii_case(want)),
        _ => false,
    }
}

/// The option currently in effect.
pub fn selected(reg: &dyn Registry, def: &TweakDef) -> Option<usize> {
    if let Some(present) = def.options.iter().position(|o| o.raw == Raw::KeyPresent) {
        let exists = reg.get(Hive::CurrentUser, def.path, def.value).is_some();
        let absent = def.options.iter().position(|o| o.raw == Raw::KeyAbsent).unwrap_or(def.default);
        return Some(if exists { present } else { absent });
    }
    let current = reg.get(Hive::CurrentUser, def.path, def.value);
    if current.is_none() {
        // Missing values: Windows falls back to its default.
        return Some(def.default);
    }
    match def.options.iter().position(|o| matches(&current, o.raw)) {
        Some(i) => Some(i),
        // Unknown numbers in a toggle behave like Windows' default; unknown text is a custom value.
        None if def.toggle => Some(def.default),
        None => None,
    }
}

pub fn is_on(reg: &dyn Registry, def: &TweakDef) -> bool {
    selected(reg, def) == Some(1)
}

fn state(reg: &dyn Registry, def: &TweakDef) -> TweakState {
    let selected = selected(reg, def);
    TweakState {
        id: def.id.into(),
        title: def.title.into(),
        description: def.description.into(),
        page: def.page,
        group: def.group.into(),
        toggle: def.toggle,
        options: def.options.iter().map(|o| o.label.to_owned()).collect(),
        selected,
        enabled: def.toggle && selected == Some(1),
        needs_restart: def.refresh == Refresh::ExplorerRestart,
        range: None,
        value: None,
    }
}

pub fn list(reg: &dyn Registry) -> Vec<TweakState> {
    CATALOG.iter().map(|def| state(reg, def)).collect()
}

pub fn get(reg: &dyn Registry, id: &str) -> Result<TweakState> {
    Ok(state(reg, find(id)?))
}

fn write(reg: &dyn Registry, path: &str, name: &str, raw: Raw) -> Result<()> {
    match raw {
        Raw::Dword(v) => reg.set(Hive::CurrentUser, path, name, RegValue::Dword(v)),
        Raw::Str(s) => reg.set(Hive::CurrentUser, path, name, RegValue::String(s.to_owned())),
        Raw::KeyPresent => reg.set(Hive::CurrentUser, path, name, RegValue::String(String::new())),
        Raw::KeyAbsent => {
            // Remove the override (the CLSID key and everything under it).
            let _ = reg.delete_key(Hive::CurrentUser, parent(path));
            Ok(())
        }
    }
}

/// Picks option `index` (for toggles: 0 = off, 1 = on).
pub fn set(reg: &dyn Registry, id: &str, index: usize) -> Result<Refresh> {
    let def = find(id)?;
    let option = def.options.get(index).ok_or_else(|| Error::Invalid(format!("option {index} of {id}")))?;
    write(reg, def.path, def.value, option.raw)?;
    for (name, raw) in option.also {
        write(reg, def.path, name, *raw)?;
    }
    Ok(def.refresh)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::MemoryRegistry;

    #[test]
    fn catalog_is_well_formed() {
        let mut ids: Vec<_> = CATALOG.iter().map(|t| t.id).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), CATALOG.len(), "ids are unique");
        for def in CATALOG {
            assert!(def.default < def.options.len(), "{}", def.id);
            assert!(def.options.len() >= 2, "{}", def.id);
            if def.toggle {
                assert_eq!(def.options.len(), 2, "{}", def.id);
            }
            let raws: Vec<_> = def.options.iter().map(|o| o.raw).collect();
            for (i, a) in raws.iter().enumerate() {
                assert!(!raws[i + 1..].contains(a), "{} has duplicate values", def.id);
            }
            assert!(!def.path.starts_with('\\') && !def.path.starts_with("HK"), "{}", def.id);
        }
        assert!(CATALOG.len() >= 90, "covers the desktop settings people look for");
    }

    #[test]
    fn defaults_apply_when_missing() {
        let reg = MemoryRegistry::new();
        let states = list(&reg);
        let get = |id: &str| states.iter().find(|s| s.id == id).unwrap().clone();
        assert!(get("transparency").enabled);
        assert!(!get("file-extensions").enabled);
        assert!(get("icon-recycle-bin").enabled);
        assert_eq!(get("taskbar-search").selected, Some(3));
        assert!(get("perm-camera").enabled);
    }

    #[test]
    fn set_writes_the_documented_values() {
        let reg = MemoryRegistry::new();
        assert_eq!(set(&reg, "apps-dark", 1).unwrap(), Refresh::Theme);
        assert_eq!(reg.get_dword(Hive::CurrentUser, PERSONALIZE, "AppsUseLightTheme"), Some(0));
        set(&reg, "hidden-files", 1).unwrap();
        assert_eq!(reg.get_dword(Hive::CurrentUser, ADVANCED, "Hidden"), Some(1));
        set(&reg, "hidden-files", 0).unwrap();
        assert_eq!(reg.get_dword(Hive::CurrentUser, ADVANCED, "Hidden"), Some(2));
        assert!(!get(&reg, "hidden-files").unwrap().enabled);

        set(&reg, "taskbar-search", 1).unwrap();
        assert_eq!(reg.get_dword(Hive::CurrentUser, SEARCH, "SearchboxTaskbarMode"), Some(1));
        assert_eq!(get(&reg, "taskbar-search").unwrap().selected, Some(1));

        set(&reg, "perm-camera", 0).unwrap();
        assert_eq!(reg.get_string(Hive::CurrentUser, &format!(r"{CONSENT}\webcam"), "Value").as_deref(), Some("Deny"));
        assert!(!get(&reg, "perm-camera").unwrap().enabled);
    }

    #[test]
    fn options_write_their_companion_values() {
        let reg = MemoryRegistry::new();
        assert_eq!(set(&reg, "clock-format", 1).unwrap(), Refresh::International);
        assert_eq!(reg.get_string(Hive::CurrentUser, INTERNATIONAL, "sShortTime").as_deref(), Some("HH:mm"));
        assert_eq!(reg.get_string(Hive::CurrentUser, INTERNATIONAL, "sTimeFormat").as_deref(), Some("HH:mm:ss"));
        set(&reg, "settings-suggestions", 0).unwrap();
        assert_eq!(reg.get_dword(Hive::CurrentUser, CDM, "SubscribedContent-353696Enabled"), Some(0));
    }

    #[test]
    fn custom_text_values_are_not_mislabelled() {
        let reg = MemoryRegistry::new();
        reg.set(Hive::CurrentUser, INTERNATIONAL, "sShortDate", RegValue::String("d MMM yy".into())).unwrap();
        assert_eq!(get(&reg, "date-format").unwrap().selected, None);
        reg.set(Hive::CurrentUser, INTERNATIONAL, "sShortDate", RegValue::String("yyyy-MM-dd".into())).unwrap();
        assert_eq!(get(&reg, "date-format").unwrap().selected, Some(2));
    }

    #[test]
    fn classic_menu_creates_and_removes_its_key() {
        let reg = MemoryRegistry::new();
        assert!(!get(&reg, "classic-context-menu").unwrap().enabled);
        assert_eq!(set(&reg, "classic-context-menu", 1).unwrap(), Refresh::ExplorerRestart);
        assert_eq!(reg.get_string(Hive::CurrentUser, CLASSIC_MENU, "").as_deref(), Some(""));
        assert!(get(&reg, "classic-context-menu").unwrap().enabled);
        set(&reg, "classic-context-menu", 0).unwrap();
        assert!(reg.get(Hive::CurrentUser, CLASSIC_MENU, "").is_none());
        assert!(!get(&reg, "classic-context-menu").unwrap().enabled);
        assert!(get(&reg, "classic-context-menu").unwrap().needs_restart);
    }

    #[test]
    fn unknown_ids_and_options_are_rejected() {
        let reg = MemoryRegistry::new();
        assert!(set(&reg, "anything-else", 1).is_err());
        assert!(set(&reg, "apps-dark", 2).is_err());
    }
}
