//! App preferences and alarms, saved as JSON under `%APPDATA%`.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::Result;
use crate::alarms::Alarm;
use crate::notes::write_atomic;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ThemePreference {
    #[default]
    System,
    Light,
    Dark,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Preferences {
    pub theme: ThemePreference,
    pub always_on_top: bool,
    /// Closing the window hides it to the tray instead of quitting.
    pub close_to_tray: bool,
    pub night_light_enabled: bool,
    /// 0-100
    pub night_light_strength: u8,
    /// IANA time zone names shown in the world clock.
    pub world_clocks: Vec<String>,
    /// Size of the whole app, 1.0 = 100%. See [`clamp_ui_scale`].
    pub ui_scale: f64,
}

pub const UI_SCALE_MIN: f64 = 0.6;
pub const UI_SCALE_MAX: f64 = 1.6;

/// Keeps the app size in range and on 5% steps; bad values fall back to 100%.
pub fn clamp_ui_scale(scale: f64) -> f64 {
    if !scale.is_finite() {
        return 1.0;
    }
    ((scale.clamp(UI_SCALE_MIN, UI_SCALE_MAX) * 20.0).round()) / 20.0
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            theme: ThemePreference::System,
            always_on_top: false,
            close_to_tray: true,
            night_light_enabled: false,
            night_light_strength: 40,
            world_clocks: vec!["Europe/London".into(), "America/New_York".into(), "Asia/Tokyo".into()],
            ui_scale: 1.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AppConfig {
    pub preferences: Preferences,
    pub alarms: Vec<Alarm>,
    /// Set once v1.x data (notes, disabled startup apps) has been imported.
    pub legacy_imported: bool,
}

pub struct ConfigStore {
    path: PathBuf,
}

impl ConfigStore {
    pub fn new(dir: &Path) -> Self {
        Self { path: dir.join("settings.json") }
    }

    /// Loads settings. A damaged file is kept as `settings.json.bak` and
    /// defaults are used, so the app always starts.
    pub fn load(&self) -> AppConfig {
        let Ok(text) = std::fs::read_to_string(&self.path) else { return AppConfig::default() };
        match serde_json::from_str::<AppConfig>(&text) {
            Ok(mut config) => {
                config.preferences.night_light_strength = config.preferences.night_light_strength.min(100);
                config.preferences.ui_scale = clamp_ui_scale(config.preferences.ui_scale);
                config
            }
            Err(_) => {
                let _ = std::fs::rename(&self.path, self.path.with_extension("json.bak"));
                AppConfig::default()
            }
        }
    }

    pub fn save(&self, config: &AppConfig) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        write_atomic(&self.path, serde_json::to_string_pretty(config)?.as_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_and_defaults() {
        let dir = tempfile::tempdir().unwrap();
        let store = ConfigStore::new(dir.path());
        assert_eq!(store.load(), AppConfig::default());
        let mut config = AppConfig::default();
        config.preferences.theme = ThemePreference::Dark;
        config.preferences.always_on_top = true;
        store.save(&config).unwrap();
        assert_eq!(store.load(), config);
    }

    #[test]
    fn partial_files_fill_in_defaults() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("settings.json"),
            r#"{"preferences":{"theme":"light","nightLightStrength":250,"uiScale":9.0}}"#,
        )
        .unwrap();
        let config = ConfigStore::new(dir.path()).load();
        assert_eq!(config.preferences.theme, ThemePreference::Light);
        assert!(config.preferences.close_to_tray);
        assert_eq!(config.preferences.night_light_strength, 100);
        assert_eq!(config.preferences.ui_scale, UI_SCALE_MAX);
    }

    #[test]
    fn ui_scale_is_clamped_to_steps() {
        assert_eq!(clamp_ui_scale(0.1), UI_SCALE_MIN);
        assert_eq!(clamp_ui_scale(1.234), 1.25);
        assert_eq!(clamp_ui_scale(f64::NAN), 1.0);
        assert_eq!(Preferences::default().ui_scale, 1.0);
    }

    #[test]
    fn corrupt_file_is_backed_up() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("settings.json"), "{not json").unwrap();
        assert_eq!(ConfigStore::new(dir.path()).load(), AppConfig::default());
        assert!(dir.path().join("settings.json.bak").exists());
    }
}
