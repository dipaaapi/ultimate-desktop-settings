// Hides the console window in release builds on Windows.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod scheduler;
mod state;
mod tray;

use tauri::{Manager, WindowEvent};

use state::AppState;

fn main() {
    let start_minimized = std::env::args().any(|a| a == "--minimized");

    tauri::Builder::default()
        // A second launch (e.g. from the Start menu) shows the running window.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| tray::show_main(app)))
        .plugin(tauri_plugin_dialog::init())
        .setup(move |app| {
            let state = AppState::load();
            state.apply_startup_preferences(app.handle());
            app.manage(state);
            tray::create(app.handle())?;
            scheduler::start(app.handle().clone());
            if !start_minimized {
                tray::show_main(app.handle());
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                let state = window.state::<AppState>();
                if state.preferences().close_to_tray {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .invoke_handler(commands::handler())
        .build(tauri::generate_context!())
        .expect("failed to start Ultimate Windows Desktop Settings")
        .run(|app, event| {
            if let tauri::RunEvent::Exit = event {
                // Never leave the screen tinted after the app quits.
                if app.state::<AppState>().preferences().night_light_enabled {
                    let _ = uds_core::system::apply_night_light(0);
                }
            }
        });
}
