//! Rings alarms and the countdown timer from a background thread, so they
//! work while the window is hidden in the tray.

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use tauri::{AppHandle, Emitter, Manager};
use uds_core::alarms::{Fired, Tone};
use uds_core::system;

use crate::state::AppState;

pub fn start(app: AppHandle) {
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(Duration::from_millis(500));
            let state = app.state::<AppState>();
            let now_ms = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0);
            let local = chrono::Local::now().naive_local();

            let fired = {
                let mut config = state.config();
                let mut clock = state.clock.lock().unwrap_or_else(|p| p.into_inner());
                let (fired, changed) = clock.tick(&mut config.alarms, local, now_ms);
                if changed {
                    let _ = state.save(&config);
                }
                fired
            };

            for event in fired {
                let tone = match &event {
                    Fired::Alarm { tone, .. } => *tone,
                    Fired::Countdown { .. } => Tone::Information,
                };
                crate::tray::show_main(&app);
                let _ = app.emit("timer-fired", &event);
                for _ in 0..3 {
                    system::beep(tone);
                    std::thread::sleep(Duration::from_millis(700));
                }
            }
        }
    });
}
