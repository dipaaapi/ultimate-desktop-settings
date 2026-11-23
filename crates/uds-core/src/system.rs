//! Operating-system calls behind one platform-neutral API. On Windows these
//! go to [`crate::win`]; elsewhere they return [`Error::Unsupported`] so the
//! app still builds and its UI can be previewed.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde::Serialize;

use crate::alarms::Tone;
#[cfg(not(windows))]
use crate::apps::InstalledApp;
#[cfg(windows)]
use crate::apps::{self, InstalledApp};
use crate::bluetooth::ServiceState;
use crate::registry::Registry;
use crate::shortcuts::Shortcut;
use crate::startup::{ScheduledStartup, StartupFolders};
use crate::tweaks::Refresh;
#[allow(unused_imports)]
use crate::{APP_DIR_NAME, Error, Result};

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioDevice {
    pub id: String,
    pub name: String,
    pub is_default: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioSession {
    pub pid: u32,
    pub name: String,
    /// 0.0 - 1.0
    pub volume: f32,
    pub muted: bool,
    pub is_system: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioState {
    /// 0.0 - 1.0
    pub master: f32,
    pub muted: bool,
    pub devices: Vec<AudioDevice>,
    pub sessions: Vec<AudioSession>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DisplayInfo {
    pub name: String,
    pub device: String,
    pub width: u32,
    pub height: u32,
    pub refresh_hz: u32,
    pub scale_percent: u32,
    pub primary: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecycleBinSummary {
    pub items: u64,
    pub bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PowerPlan {
    pub id: String,
    pub name: String,
    pub active: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatteryStatus {
    pub has_battery: bool,
    pub percent: Option<u8>,
    pub plugged_in: bool,
    pub charging: bool,
    pub saver_on: bool,
}

/// The registry this platform uses: the real one on Windows, an empty
/// in-memory one elsewhere.
pub fn registry() -> Arc<dyn Registry> {
    #[cfg(windows)]
    {
        Arc::new(crate::win::registry::WindowsRegistry)
    }
    #[cfg(not(windows))]
    {
        Arc::new(crate::registry::MemoryRegistry::new())
    }
}

/// `%APPDATA%\UltimateWindowsDesktopSettings` (or the platform equivalent).
pub fn app_data_dir() -> PathBuf {
    let base = std::env::var_os("APPDATA")
        .or_else(|| std::env::var_os("XDG_CONFIG_HOME"))
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))
        .unwrap_or_else(std::env::temp_dir);
    base.join(APP_DIR_NAME)
}

/// The current user's Startup folder.
pub fn startup_folder() -> Option<PathBuf> {
    std::env::var_os("APPDATA").map(|a| PathBuf::from(a).join(r"Microsoft\Windows\Start Menu\Programs\Startup"))
}

/// This user's and the all-users Startup folders.
pub fn startup_folders() -> StartupFolders {
    StartupFolders {
        user: startup_folder(),
        common: std::env::var_os("ProgramData")
            .map(|p| PathBuf::from(p).join(r"Microsoft\Windows\Start Menu\Programs\StartUp")),
    }
}

/// Where v1.x kept notes (it was hardcoded to the developer's project folder).
pub fn legacy_notes_dir() -> Option<PathBuf> {
    std::env::var_os("USERPROFILE")
        .map(|home| PathBuf::from(home).join(r".gemini\antigravity-ide\scratch\ultimate_desktop_settings\notes"))
}

pub fn temp_dir() -> PathBuf {
    std::env::temp_dir()
}

macro_rules! windows_only {
    ($win:expr, $other:expr) => {{
        #[cfg(windows)]
        {
            $win
        }
        #[cfg(not(windows))]
        {
            $other
        }
    }};
}

pub fn audio_state() -> Result<AudioState> {
    windows_only!(crate::win::audio::state(), Err(Error::Unsupported))
}

pub fn set_master_audio(level: Option<f32>, mute: Option<bool>) -> Result<()> {
    windows_only!(crate::win::audio::set_master(level, mute), {
        let _ = (level, mute);
        Err(Error::Unsupported)
    })
}

pub fn set_session_audio(pid: u32, level: Option<f32>, mute: Option<bool>) -> Result<()> {
    windows_only!(crate::win::audio::set_session(pid, level, mute), {
        let _ = (pid, level, mute);
        Err(Error::Unsupported)
    })
}

pub fn displays() -> Result<Vec<DisplayInfo>> {
    windows_only!(Ok(crate::win::display::list()), Err(Error::Unsupported))
}

pub fn apply_night_light(strength: u8) -> Result<()> {
    windows_only!(crate::win::display::apply_night_light(strength), {
        let _ = strength;
        Err(Error::Unsupported)
    })
}

pub fn bluetooth_service() -> ServiceState {
    windows_only!(crate::win::service::state("bthserv"), ServiceState::Unknown)
}

pub fn control_bluetooth_service(start: bool) -> Result<()> {
    windows_only!(crate::win::service::control_elevated("bthserv", start), {
        let _ = start;
        Err(Error::Unsupported)
    })
}

pub fn open_shortcut(shortcut: &Shortcut) -> Result<()> {
    windows_only!(crate::win::shell::open_target(shortcut.target), {
        let _ = shortcut;
        Err(Error::Unsupported)
    })
}

pub fn open_folder(path: &Path) -> Result<()> {
    windows_only!(crate::win::shell::open_folder(path), {
        let _ = path;
        Err(Error::Unsupported)
    })
}

pub fn launch(exe: &Path, args: &[&str]) -> Result<()> {
    windows_only!(crate::win::shell::launch(exe, args), {
        let _ = (exe, args);
        Err(Error::Unsupported)
    })
}

pub fn refresh_desktop() -> Result<()> {
    windows_only!(
        {
            crate::win::shell::refresh_desktop();
            Ok(())
        },
        Err(Error::Unsupported)
    )
}

pub fn notify_changed(refresh: Refresh) {
    windows_only!(crate::win::shell::notify_changed(refresh), {
        let _ = refresh;
    })
}

pub fn recycle_bin() -> RecycleBinSummary {
    windows_only!(
        {
            let (items, bytes) = crate::win::shell::recycle_bin();
            RecycleBinSummary { items, bytes }
        },
        RecycleBinSummary::default()
    )
}

pub fn empty_recycle_bin() -> Result<()> {
    windows_only!(crate::win::shell::empty_recycle_bin(), Err(Error::Unsupported))
}

pub fn beep(tone: Tone) {
    windows_only!(crate::win::shell::beep(tone), {
        let _ = tone;
    })
}

/// Asks the windows of an installed app's running processes to close.
/// Returns how many windows were asked to close.
pub fn close_app(app: &InstalledApp) -> Result<usize> {
    windows_only!(
        {
            let pids: Vec<u32> = crate::win::process::running()
                .into_iter()
                .filter(|(_, path)| apps::process_belongs_to(path, app))
                .map(|(pid, _)| pid)
                .filter(|pid| *pid != std::process::id())
                .collect();
            if pids.is_empty() {
                return Err(Error::msg(format!("{} isn't running.", app.name)));
            }
            Ok(crate::win::process::close_windows(&pids))
        },
        {
            let _ = app;
            Err(Error::Unsupported)
        }
    )
}

/// Scheduled tasks that start a program at sign-in or boot.
pub fn scheduled_startup_tasks() -> Vec<ScheduledStartup> {
    windows_only!(crate::win::tasks::startup_tasks(), Vec::new())
}

pub fn set_task_enabled(path: &str, enabled: bool) -> Result<()> {
    windows_only!(crate::win::tasks::set_enabled(path, enabled), {
        let _ = (path, enabled);
        Err(Error::Unsupported)
    })
}

/// Runs this app again as admin (through the UAC prompt) with `args` and
/// waits for it to finish.
pub fn run_self_elevated(args: &str) -> Result<()> {
    windows_only!(crate::win::shell::run_self_elevated(args), {
        let _ = args;
        Err(Error::Unsupported)
    })
}

/// Reads a setting from [`crate::live::CATALOG`]. `None` when Windows can't say.
pub fn live_get(id: &str) -> Option<i32> {
    windows_only!(
        match id {
            "taskbar-autohide" => Some(i32::from(crate::win::shell::taskbar_autohide())),
            _ => crate::win::params::get(id).or_else(|| crate::win::power::get(id)),
        },
        {
            let _ = id;
            None
        }
    )
}

/// Changes a setting from [`crate::live::CATALOG`]; the value is already validated.
pub fn live_set(reg: &dyn Registry, id: &str, value: i32) -> Result<()> {
    windows_only!(
        match id {
            "taskbar-autohide" => {
                crate::win::shell::set_taskbar_autohide(value != 0);
                Ok(())
            }
            _ if crate::live::find(id).is_some_and(|d| d.page == crate::tweaks::Page::Power) => {
                crate::win::power::set(id, value)
            }
            _ => crate::win::params::set_value(reg, id, value),
        },
        {
            let _ = (reg, id, value);
            Err(Error::Unsupported)
        }
    )
}

pub fn power_plans() -> Result<Vec<PowerPlan>> {
    windows_only!(Ok(crate::win::power::plans()), Err(Error::Unsupported))
}

pub fn set_power_plan(id: &str) -> Result<()> {
    windows_only!(crate::win::power::set_plan(id), {
        let _ = id;
        Err(Error::Unsupported)
    })
}

pub fn battery() -> BatteryStatus {
    windows_only!(crate::win::power::battery(), BatteryStatus::default())
}

pub fn wallpaper() -> Option<String> {
    windows_only!(crate::win::shell::wallpaper(), None)
}

pub fn set_wallpaper(path: &Path) -> Result<()> {
    windows_only!(crate::win::shell::set_wallpaper(path), {
        let _ = path;
        Err(Error::Unsupported)
    })
}

pub fn restart_explorer() -> Result<()> {
    windows_only!(crate::win::shell::restart_explorer(), Err(Error::Unsupported))
}

/// Turns a startup entry on or off, or removes it. Scheduled tasks go to the
/// Task Scheduler; everything else to [`crate::startup`]. Shared by the app
/// and its elevated helper.
pub fn apply_startup_change(reg: &dyn Registry, id: &str, change: crate::startup::Change) -> Result<()> {
    use crate::startup::{self, Change, StartupSource};
    let folders = startup_folders();
    let (source, name) = startup::parse_id(id)?;
    match (source, change) {
        (_, Change::Remove) => startup::remove(reg, &folders, id),
        (StartupSource::ScheduledTask, _) => set_task_enabled(name, change == Change::Enable),
        _ => startup::set_enabled(reg, &folders, id, change == Change::Enable),
    }
}
