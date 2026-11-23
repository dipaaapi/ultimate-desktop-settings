// Hides the console window in release builds on Windows.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod scheduler;
mod state;
mod tray;
mod window;

use tauri::{Manager, WindowEvent};

use state::AppState;

/// Marks a run of this program as the short-lived admin helper that makes
/// one startup change after the UAC prompt, then exits.
pub const ELEVATED_FLAG: &str = "--uds-elevated";

/// `--uds-elevated startup <enable|disable|remove> <hex id>`. Returns the exit code.
fn run_elevated(args: &[String]) -> i32 {
    use uds_core::startup::{Change, decode_arg};
    let [kind, change, id] = args else { return 2 };
    let (Some(change), Some(id)) = (Change::from_arg(change), decode_arg(id)) else { return 2 };
    if kind != "startup" {
        return 2;
    }
    let reg = uds_core::system::registry();
    match uds_core::system::apply_startup_change(reg.as_ref(), &id, change) {
        Ok(()) => 0,
        Err(_) => 1,
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).map(String::as_str) == Some(ELEVATED_FLAG) {
        // Runs before Tauri starts, so no window, tray or single-instance check.
        std::process::exit(run_elevated(&args[2..]));
    }
    let start_minimized = std::env::args().any(|a| a == "--minimized");

    tauri::Builder::default()
        // A second launch (e.g. from the Start menu) shows the running window.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| tray::show_main(app)))
        .plugin(tauri_plugin_dialog::init())
        .setup(move |app| {
            let state = AppState::load();
            state.apply_startup_preferences(app.handle());
            let ui_scale = state.preferences().ui_scale;
            app.manage(state);
            app.manage(window::WindowState::new(ui_scale));
            // Fit the rod to this screen, at the user's app size, before it first shows.
            if let Some(main) = app.get_webview_window("main") {
                let _ = main.set_zoom(ui_scale);
                let _ = window::apply(&main, &app.state::<window::WindowState>(), uds_core::window_layout::Mode::Rod);
            }
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
