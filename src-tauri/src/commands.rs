//! Everything the UI can ask for. Each command validates its input and calls
//! into `uds-core`; the UI never passes raw registry paths or command lines.

use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::Serialize;
use tauri::{AppHandle, Manager, State, WebviewWindow};
use uds_core::alarms::{Alarm, Countdown, Tone};
use uds_core::apps::{self, InstalledApp};
use uds_core::bluetooth::{self, BluetoothState};
use uds_core::browsers::{self, Browser};
use uds_core::config::Preferences;
use uds_core::notes::NoteMeta;
use uds_core::shortcuts::{self, ShortcutInfo};
use uds_core::startup::{self, StartupItem};
use uds_core::system::{self, AudioState, DisplayInfo, RecycleBinSummary};
use uds_core::temp::{self, CleanResult, TempSummary};
use uds_core::tweaks::{self, TweakState};
use uds_core::window_layout::{Mode, Placement};
use uds_core::{Error, Result};

use crate::state::AppState;
use crate::window::{self, WindowState};

type Cmd<'a> = State<'a, AppState>;

/// Temp files newer than this are left alone; they usually belong to an
/// installer or app that is running right now.
const TEMP_MIN_AGE: Duration = Duration::from_secs(24 * 60 * 60);

pub fn handler() -> impl Fn(tauri::ipc::Invoke) -> bool + Send + Sync + 'static {
    tauri::generate_handler![
        app_info,
        get_preferences,
        set_preferences,
        get_launch_at_startup,
        set_launch_at_startup,
        overview,
        refresh_desktop,
        temp_summary,
        clean_temp,
        empty_recycle_bin,
        list_startup,
        set_startup_enabled,
        add_startup,
        remove_startup,
        audio_state,
        set_master_volume,
        set_master_mute,
        set_app_volume,
        set_app_mute,
        list_displays,
        list_tweaks,
        set_tweak,
        bluetooth_state,
        control_bluetooth,
        list_browsers,
        launch_browser,
        list_notes,
        read_note,
        save_note,
        create_note,
        rename_note,
        delete_note,
        pin_note,
        open_notes_folder,
        list_apps,
        launch_app,
        open_app_folder,
        close_app,
        list_shortcuts,
        open_shortcut,
        list_alarms,
        save_alarm,
        delete_alarm,
        test_tone,
        start_countdown,
        cancel_countdown,
        countdown_state,
        set_window_mode,
        window_placement,
        start_dragging,
        hide_to_tray,
        quit_app,
    ]
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    version: String,
    data_dir: String,
    platform: &'static str,
}

#[tauri::command]
async fn app_info(app: AppHandle) -> Result<AppInfo> {
    Ok(AppInfo {
        version: app.package_info().version.to_string(),
        data_dir: system::app_data_dir().display().to_string(),
        platform: std::env::consts::OS,
    })
}

#[tauri::command]
async fn get_preferences(state: Cmd<'_>) -> Result<Preferences> {
    Ok(state.preferences())
}

#[tauri::command]
async fn set_preferences(app: AppHandle, state: Cmd<'_>, preferences: Preferences) -> Result<Preferences> {
    let mut prefs = preferences;
    prefs.night_light_strength = prefs.night_light_strength.min(100);
    prefs.world_clocks.truncate(12);

    let previous = state.preferences();
    if prefs.always_on_top != previous.always_on_top {
        if let Some(window) = app.get_webview_window("main") {
            window.set_always_on_top(prefs.always_on_top).map_err(|e| Error::msg(e.to_string()))?;
        }
    }
    let night_changed = prefs.night_light_enabled != previous.night_light_enabled
        || (prefs.night_light_enabled && prefs.night_light_strength != previous.night_light_strength);
    if night_changed {
        let strength = if prefs.night_light_enabled { prefs.night_light_strength } else { 0 };
        if let Err(e) = system::apply_night_light(strength) {
            prefs.night_light_enabled = false;
            let _ = system::apply_night_light(0);
            let mut config = state.config();
            config.preferences = prefs;
            state.save(&config)?;
            return Err(e);
        }
    }

    let mut config = state.config();
    config.preferences = prefs.clone();
    state.save(&config)?;
    Ok(prefs)
}

#[tauri::command]
async fn get_launch_at_startup(state: Cmd<'_>) -> Result<bool> {
    Ok(startup::self_launch_enabled(state.reg.as_ref()))
}

#[tauri::command]
async fn set_launch_at_startup(state: Cmd<'_>, enabled: bool) -> Result<bool> {
    let exe = std::env::current_exe()?;
    startup::set_self_launch(state.reg.as_ref(), &exe, enabled)?;
    Ok(startup::self_launch_enabled(state.reg.as_ref()))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Overview {
    startup_total: usize,
    startup_enabled: usize,
    bluetooth_devices: usize,
    temp: TempSummary,
    recycle_bin: RecycleBinSummary,
}

#[tauri::command]
async fn overview(state: Cmd<'_>) -> Result<Overview> {
    let items = startup::list(state.reg.as_ref(), system::startup_folder().as_deref());
    Ok(Overview {
        startup_total: items.len(),
        startup_enabled: items.iter().filter(|i| i.enabled).count(),
        bluetooth_devices: bluetooth::paired_devices(state.reg.as_ref()).len(),
        temp: temp::summarize(&system::temp_dir()),
        recycle_bin: system::recycle_bin(),
    })
}

#[tauri::command]
async fn refresh_desktop() -> Result<()> {
    system::refresh_desktop()
}

#[tauri::command]
async fn temp_summary() -> Result<TempSummary> {
    Ok(temp::summarize(&system::temp_dir()))
}

#[tauri::command]
async fn clean_temp() -> Result<CleanResult> {
    Ok(temp::clean(&system::temp_dir(), TEMP_MIN_AGE))
}

#[tauri::command]
async fn empty_recycle_bin() -> Result<RecycleBinSummary> {
    if system::recycle_bin().items > 0 {
        system::empty_recycle_bin()?;
    }
    Ok(system::recycle_bin())
}

#[tauri::command]
async fn list_startup(state: Cmd<'_>) -> Result<Vec<StartupItem>> {
    Ok(startup::list(state.reg.as_ref(), system::startup_folder().as_deref()))
}

#[tauri::command]
async fn set_startup_enabled(state: Cmd<'_>, id: String, enabled: bool) -> Result<Vec<StartupItem>> {
    startup::set_enabled(state.reg.as_ref(), &id, enabled)?;
    list_startup(state).await
}

#[tauri::command]
async fn add_startup(state: Cmd<'_>, name: String, command: String) -> Result<Vec<StartupItem>> {
    startup::add(state.reg.as_ref(), &name, &command)?;
    list_startup(state).await
}

#[tauri::command]
async fn remove_startup(state: Cmd<'_>, id: String) -> Result<Vec<StartupItem>> {
    startup::remove(state.reg.as_ref(), &id, system::startup_folder().as_deref())?;
    list_startup(state).await
}

#[tauri::command]
async fn audio_state() -> Result<AudioState> {
    system::audio_state()
}

fn level(value: f32) -> Result<f32> {
    if value.is_finite() { Ok(value.clamp(0.0, 1.0)) } else { Err(Error::Invalid("volume".into())) }
}

#[tauri::command]
async fn set_master_volume(level_value: f32) -> Result<AudioState> {
    system::set_master_audio(Some(level(level_value)?), None)?;
    system::audio_state()
}

#[tauri::command]
async fn set_master_mute(muted: bool) -> Result<AudioState> {
    system::set_master_audio(None, Some(muted))?;
    system::audio_state()
}

#[tauri::command]
async fn set_app_volume(pid: u32, level_value: f32) -> Result<AudioState> {
    system::set_session_audio(pid, Some(level(level_value)?), None)?;
    system::audio_state()
}

#[tauri::command]
async fn set_app_mute(pid: u32, muted: bool) -> Result<AudioState> {
    system::set_session_audio(pid, None, Some(muted))?;
    system::audio_state()
}

#[tauri::command]
async fn list_displays() -> Result<Vec<DisplayInfo>> {
    system::displays()
}

#[tauri::command]
async fn list_tweaks(state: Cmd<'_>) -> Result<Vec<TweakState>> {
    Ok(tweaks::list(state.reg.as_ref()))
}

#[tauri::command]
async fn set_tweak(state: Cmd<'_>, id: String, enabled: bool) -> Result<Vec<TweakState>> {
    let refresh = tweaks::set(state.reg.as_ref(), &id, enabled)?;
    system::notify_changed(refresh);
    Ok(tweaks::list(state.reg.as_ref()))
}

#[tauri::command]
async fn bluetooth_state(state: Cmd<'_>) -> Result<BluetoothState> {
    Ok(BluetoothState { service: system::bluetooth_service(), devices: bluetooth::paired_devices(state.reg.as_ref()) })
}

#[tauri::command]
async fn control_bluetooth(start: bool) -> Result<()> {
    system::control_bluetooth_service(start)
}

#[tauri::command]
async fn list_browsers(state: Cmd<'_>) -> Result<Vec<Browser>> {
    Ok(browsers::list(state.reg.as_ref()))
}

#[tauri::command]
async fn launch_browser(state: Cmd<'_>, id: String, private: bool) -> Result<()> {
    let browser = browsers::list(state.reg.as_ref())
        .into_iter()
        .find(|b| b.id == id)
        .ok_or_else(|| Error::NotFound(format!("browser {id}")))?;
    let args: Vec<&str> = match (private, browser.kind.private_flag()) {
        (true, Some(flag)) => vec![flag],
        (true, None) => return Err(Error::msg(format!("{} has no private window option.", browser.name))),
        (false, _) => vec![],
    };
    system::launch(Path::new(&browser.exe), &args)
}

#[tauri::command]
async fn list_notes(state: Cmd<'_>) -> Result<Vec<NoteMeta>> {
    state.notes.list()
}

#[tauri::command]
async fn read_note(state: Cmd<'_>, title: String) -> Result<String> {
    state.notes.read(&title)
}

#[tauri::command]
async fn save_note(state: Cmd<'_>, title: String, content: String) -> Result<()> {
    if content.len() > 5 * 1024 * 1024 {
        return Err(Error::Invalid("Notes can be at most 5 MB.".into()));
    }
    state.notes.save(&title, &content)
}

#[tauri::command]
async fn create_note(state: Cmd<'_>, title: String) -> Result<String> {
    state.notes.create(&title)
}

#[tauri::command]
async fn rename_note(state: Cmd<'_>, title: String, new_title: String) -> Result<String> {
    state.notes.rename(&title, &new_title)
}

#[tauri::command]
async fn delete_note(state: Cmd<'_>, title: String) -> Result<()> {
    state.notes.delete(&title)
}

#[tauri::command]
async fn pin_note(state: Cmd<'_>, title: String, pinned: bool) -> Result<()> {
    state.notes.set_pinned(&title, pinned)
}

#[tauri::command]
async fn open_notes_folder(state: Cmd<'_>) -> Result<()> {
    system::open_folder(state.notes.dir())
}

fn find_app(state: &AppState, id: &str) -> Result<InstalledApp> {
    apps::list(state.reg.as_ref()).into_iter().find(|a| a.id == id).ok_or_else(|| Error::NotFound(format!("app {id}")))
}

#[tauri::command]
async fn list_apps(state: Cmd<'_>) -> Result<Vec<InstalledApp>> {
    Ok(apps::list(state.reg.as_ref()))
}

#[tauri::command]
async fn launch_app(state: Cmd<'_>, id: String) -> Result<()> {
    let app = find_app(&state, &id)?;
    match &app.exe {
        Some(exe) => system::launch(Path::new(exe), &[]),
        None => Err(Error::msg(format!("{} doesn't say which program to start. Try Open folder.", app.name))),
    }
}

#[tauri::command]
async fn open_app_folder(state: Cmd<'_>, id: String) -> Result<()> {
    let app = find_app(&state, &id)?;
    let folder = app
        .install_location
        .map(PathBuf::from)
        .or_else(|| app.exe.as_deref().and_then(|e| Path::new(e).parent().map(Path::to_path_buf)))
        .ok_or_else(|| Error::msg(format!("{} doesn't list an install folder.", app.name)))?;
    system::open_folder(&folder)
}

#[tauri::command]
async fn close_app(state: Cmd<'_>, id: String) -> Result<usize> {
    let app = find_app(&state, &id)?;
    system::close_app(&app)
}

#[tauri::command]
async fn list_shortcuts() -> Result<Vec<ShortcutInfo>> {
    Ok(shortcuts::list())
}

#[tauri::command]
async fn open_shortcut(id: String) -> Result<()> {
    system::open_shortcut(shortcuts::find(&id)?)
}

#[tauri::command]
async fn list_alarms(state: Cmd<'_>) -> Result<Vec<Alarm>> {
    Ok(state.config().alarms.clone())
}

#[tauri::command]
async fn save_alarm(state: Cmd<'_>, alarm: Alarm) -> Result<Vec<Alarm>> {
    alarm.validate()?;
    let mut config = state.config();
    if let Some(existing) = config.alarms.iter_mut().find(|a| a.id == alarm.id) {
        *existing = alarm;
    } else if config.alarms.len() >= 50 {
        return Err(Error::Invalid("You can keep up to 50 alarms.".into()));
    } else {
        config.alarms.push(alarm);
    }
    config.alarms.sort_by_key(|a| (a.hour, a.minute));
    state.save(&config)?;
    Ok(config.alarms.clone())
}

#[tauri::command]
async fn delete_alarm(state: Cmd<'_>, id: String) -> Result<Vec<Alarm>> {
    let mut config = state.config();
    config.alarms.retain(|a| a.id != id);
    state.save(&config)?;
    Ok(config.alarms.clone())
}

#[tauri::command]
async fn test_tone(tone: Tone) -> Result<()> {
    system::beep(tone);
    Ok(())
}

#[tauri::command]
async fn start_countdown(state: Cmd<'_>, seconds: u64, label: String) -> Result<Countdown> {
    if seconds == 0 || seconds > 24 * 60 * 60 {
        return Err(Error::Invalid("Timer must be between 1 second and 24 hours.".into()));
    }
    let now = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0);
    let countdown = Countdown { label: label.chars().take(60).collect(), ends_at_ms: now + seconds * 1000 };
    state.clock.lock().unwrap_or_else(|p| p.into_inner()).countdown = Some(countdown.clone());
    Ok(countdown)
}

#[tauri::command]
async fn cancel_countdown(state: Cmd<'_>) -> Result<()> {
    state.clock.lock().unwrap_or_else(|p| p.into_inner()).countdown = None;
    Ok(())
}

#[tauri::command]
async fn countdown_state(state: Cmd<'_>) -> Result<Option<Countdown>> {
    Ok(state.clock.lock().unwrap_or_else(|p| p.into_inner()).countdown.clone())
}

// ---- Floating window ----

fn tauri_err(error: tauri::Error) -> Error {
    Error::msg(error.to_string())
}

#[tauri::command]
async fn set_window_mode(window: WebviewWindow, state: State<'_, WindowState>, mode: Mode) -> Result<Placement> {
    window::apply(&window, &state, mode).map_err(tauri_err)
}

#[tauri::command]
async fn window_placement(window: WebviewWindow, state: State<'_, WindowState>) -> Result<Placement> {
    window::placement(&window, &state).map_err(tauri_err)
}

/// Lets the user move the rod by pressing anywhere on it and dragging.
#[tauri::command]
fn start_dragging(window: WebviewWindow) -> Result<()> {
    window.start_dragging().map_err(tauri_err)
}

#[tauri::command]
async fn hide_to_tray(window: WebviewWindow) -> Result<()> {
    window.hide().map_err(tauri_err)
}

#[tauri::command]
async fn quit_app(app: AppHandle) -> Result<()> {
    app.exit(0);
    Ok(())
}
