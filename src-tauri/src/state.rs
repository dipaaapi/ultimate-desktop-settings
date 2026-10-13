use std::sync::{Arc, Mutex, MutexGuard};

use tauri::{AppHandle, Manager};
use uds_core::alarms::Clock;
use uds_core::config::{AppConfig, ConfigStore, Preferences};
use uds_core::notes::NotesStore;
use uds_core::registry::Registry;
use uds_core::{startup, system};

pub struct AppState {
    pub reg: Arc<dyn Registry>,
    pub notes: NotesStore,
    pub clock: Mutex<Clock>,
    config: Mutex<AppConfig>,
    store: ConfigStore,
}

impl AppState {
    pub fn load() -> Self {
        let dir = system::app_data_dir();
        let store = ConfigStore::new(&dir);
        let mut config = store.load();
        let reg = system::registry();
        let notes = NotesStore::open(dir.join("notes")).unwrap_or_else(|_| {
            // Fall back to a temp folder so the app still opens; Notes will say where it saves.
            NotesStore::open(std::env::temp_dir().join(uds_core::APP_DIR_NAME).join("notes")).expect("notes folder")
        });

        if !config.legacy_imported {
            if let Some(legacy) = system::legacy_notes_dir() {
                let _ = notes.import_from(&legacy);
            }
            startup::migrate_legacy(reg.as_ref());
            config.legacy_imported = true;
            let _ = store.save(&config);
        }

        Self { reg, notes, clock: Mutex::new(Clock::default()), config: Mutex::new(config), store }
    }

    pub fn config(&self) -> MutexGuard<'_, AppConfig> {
        self.config.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub fn preferences(&self) -> Preferences {
        self.config().preferences.clone()
    }

    pub fn save(&self, config: &AppConfig) -> uds_core::Result<()> {
        self.store.save(config)
    }

    pub fn apply_startup_preferences(&self, app: &AppHandle) {
        let prefs = self.preferences();
        if let Some(window) = app.get_webview_window("main") {
            let _ = window.set_always_on_top(prefs.always_on_top);
        }
        if prefs.night_light_enabled {
            let _ = system::apply_night_light(prefs.night_light_strength);
        }
    }
}
