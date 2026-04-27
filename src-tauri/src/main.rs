// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Manager;

mod error;
mod ipc;
mod mpv;
mod plugin;
mod render;
mod utils;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(ipc::state::AppState::default())
        .invoke_handler(tauri::generate_handler![
            ipc::commands::play_file,
            ipc::commands::pause,
            ipc::commands::resume,
            ipc::commands::seek,
            ipc::commands::set_volume,
            ipc::commands::get_player_state,
            ipc::commands::get_playlist_state,
            ipc::commands::get_startup_fatal_error,
            ipc::commands::retry_startup_probe,
            ipc::commands::pick_and_play_file,
            ipc::commands::emit_debug_video_error,
            ipc::commands::emit_debug_fatal_error,
            ipc::commands::open_log_directory,
            ipc::commands::get_log_directory,
            ipc::commands::save_fatal_diagnostic_report,
            ipc::commands::playlist_next,
            ipc::commands::playlist_prev,
            ipc::commands::list_plugins,
            ipc::commands::toggle_plugin,
            ipc::commands::get_plugin_detail,
            ipc::commands::capture_screenshot,
            ipc::subtitle::search_subtitles,
            ipc::subtitle::download_subtitle,
            ipc::mediainfo::get_media_info,
            ipc::bookmark::list_bookmarks,
            ipc::bookmark::add_bookmark,
            ipc::bookmark::delete_bookmark,
            ipc::chapter::list_chapters
        ])
        .setup(|app| {
            if let Err(startup_error) = mpv::core::startup_probe() {
                let app_state = app.state::<ipc::state::AppState>();
                if let Err(report_error) =
                    error::fallback::handle_startup_error(app.handle(), &app_state, &startup_error)
                {
                    eprintln!("failed to report startup fatal error: {report_error}");
                }
            }

            // Initialize the plugin system.
            {
                let app_state = app.state::<ipc::state::AppState>();
                let mut bus = app_state.plugin_bus.lock().unwrap();
                let mut registry = app_state.plugin_registry.lock().unwrap();
                plugin::init(app.handle(), &mut bus, &mut registry);
            }

            #[cfg(debug_assertions)]
            {
                let window = app.get_webview_window("main").unwrap();
                window.open_devtools();
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
