use tauri::{AppHandle, State};

use crate::error::fallback;
use crate::ipc::events;
use crate::ipc::state::{AppState, PlayerState, PlaylistState, StartupFatalState};
use crate::mpv::{core, renderer};
use crate::plugin::registry::PluginInfo;
use crate::utils::paths;

fn reveal_file_in_file_manager(path: &std::path::Path) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    let status = std::process::Command::new("open")
        .arg("-R")
        .arg(path)
        .status();

    #[cfg(target_os = "windows")]
    let status = std::process::Command::new("explorer")
        .arg(format!("/select,{}", path.display()))
        .status();

    #[cfg(all(unix, not(target_os = "macos")))]
    let status = std::process::Command::new("xdg-open")
        .arg(path.parent().unwrap_or(path))
        .status();

    status
        .map_err(|e| format!("reveal report file failed: {e}"))?
        .success()
        .then_some(())
        .ok_or_else(|| "reveal report file exited with non-zero status".to_string())
}

fn emit_state_and_progress(app: &AppHandle, state: &PlayerState) -> Result<(), String> {
    let state_payload = events::PlayerStateChangePayload {
        state: state.state.clone(),
        position: state.position,
        duration: state.duration,
        volume: state.volume,
    };
    events::emit_player_state_change(app, &state_payload)?;

    let progress_payload = events::PlayerProgressPayload {
        position: state.position,
        duration: state.duration,
    };
    events::emit_player_progress(app, &progress_payload)?;

    Ok(())
}

fn apply_play_file_state(path: &str, app_state: &State<'_, AppState>) -> Result<PlayerState, String> {
    println!("Playing file: {}", path);
    let mut state = app_state
        .player
        .lock()
        .map_err(|e| format!("state lock poisoned: {e}"))?;
    state.state = "playing".to_string();
    state.position = 0.0;
    state.duration = 0.0;
    Ok(state.clone())
}

fn set_current_path_in_playlist(path: &str, app_state: &State<'_, AppState>) -> Result<(), String> {
    let mut playlist = app_state
        .playlist
        .lock()
        .map_err(|e| format!("playlist lock poisoned: {e}"))?;

    if let Some(existing) = playlist.items.iter().position(|item| item == path) {
        playlist.current_index = Some(existing);
        return Ok(());
    }

    playlist.items.push(path.to_string());
    playlist.current_index = Some(playlist.items.len().saturating_sub(1));
    Ok(())
}

fn shift_playlist_cursor(
    app_state: &State<'_, AppState>,
    step: i32,
) -> Result<Option<String>, String> {
    let mut playlist = app_state
        .playlist
        .lock()
        .map_err(|e| format!("playlist lock poisoned: {e}"))?;

    if playlist.items.is_empty() {
        return Ok(None);
    }

    let current = playlist.current_index.unwrap_or(0) as i32;
    let max_index = playlist.items.len() as i32 - 1;
    let next = (current + step).clamp(0, max_index) as usize;
    playlist.current_index = Some(next);
    Ok(Some(playlist.items[next].clone()))
}

#[tauri::command]
pub async fn play_file(
    path: String,
    app: AppHandle,
    app_state: State<'_, AppState>,
) -> Result<String, String> {
    set_current_path_in_playlist(&path, &app_state)?;
    let snapshot = apply_play_file_state(&path, &app_state)?;
    emit_state_and_progress(&app, &snapshot)?;
    Ok(path)
}

#[tauri::command]
pub async fn pick_and_play_file(
    app: AppHandle,
    app_state: State<'_, AppState>,
) -> Result<Option<String>, String> {
    let selected = rfd::FileDialog::new()
        .add_filter("Video", &["mp4", "mkv", "mov", "avi", "webm"])
        .pick_file();

    let Some(path) = selected else {
        return Ok(None);
    };

    let path_string = path.to_string_lossy().to_string();
    set_current_path_in_playlist(&path_string, &app_state)?;
    let snapshot = apply_play_file_state(&path_string, &app_state)?;
    emit_state_and_progress(&app, &snapshot)?;
    Ok(Some(path_string))
}

#[tauri::command]
pub async fn playlist_next(
    app: AppHandle,
    app_state: State<'_, AppState>,
) -> Result<Option<String>, String> {
    let Some(path) = shift_playlist_cursor(&app_state, 1)? else {
        return Ok(None);
    };
    let snapshot = apply_play_file_state(&path, &app_state)?;
    emit_state_and_progress(&app, &snapshot)?;
    Ok(Some(path))
}

#[tauri::command]
pub async fn playlist_prev(
    app: AppHandle,
    app_state: State<'_, AppState>,
) -> Result<Option<String>, String> {
    let Some(path) = shift_playlist_cursor(&app_state, -1)? else {
        return Ok(None);
    };
    let snapshot = apply_play_file_state(&path, &app_state)?;
    emit_state_and_progress(&app, &snapshot)?;
    Ok(Some(path))
}

#[tauri::command]
pub async fn pause(app: AppHandle, app_state: State<'_, AppState>) -> Result<(), String> {
    println!("Pause requested");
    let mut state = app_state
        .player
        .lock()
        .map_err(|e| format!("state lock poisoned: {e}"))?;
    state.state = "paused".to_string();
    let snapshot = state.clone();
    drop(state);
    emit_state_and_progress(&app, &snapshot)?;
    Ok(())
}

#[tauri::command]
pub async fn resume(app: AppHandle, app_state: State<'_, AppState>) -> Result<(), String> {
    println!("Resume requested");
    let mut state = app_state
        .player
        .lock()
        .map_err(|e| format!("state lock poisoned: {e}"))?;
    state.state = "playing".to_string();
    let snapshot = state.clone();
    drop(state);
    emit_state_and_progress(&app, &snapshot)?;
    Ok(())
}

#[tauri::command]
pub async fn seek(
    position: f64,
    app: AppHandle,
    app_state: State<'_, AppState>,
) -> Result<(), String> {
    println!("Seek to: {}", position);
    let mut state = app_state
        .player
        .lock()
        .map_err(|e| format!("state lock poisoned: {e}"))?;
    let mut next = position.max(0.0);
    if state.duration > 0.0 {
        next = next.min(state.duration);
    }
    state.position = next;
    let snapshot = state.clone();
    drop(state);
    emit_state_and_progress(&app, &snapshot)?;
    Ok(())
}

#[tauri::command]
pub async fn set_volume(
    volume: f64,
    app: AppHandle,
    app_state: State<'_, AppState>,
) -> Result<(), String> {
    println!("Set volume: {}", volume);
    let mut state = app_state
        .player
        .lock()
        .map_err(|e| format!("state lock poisoned: {e}"))?;
    state.volume = volume.clamp(0.0, 100.0);
    let snapshot = state.clone();
    drop(state);
    emit_state_and_progress(&app, &snapshot)?;
    Ok(())
}

#[tauri::command]
pub async fn get_player_state(app_state: State<'_, AppState>) -> Result<PlayerState, String> {
    let snapshot = app_state
        .player
        .lock()
        .map_err(|e| format!("state lock poisoned: {e}"))?
        .clone();
    Ok(snapshot)
}

#[tauri::command]
pub async fn get_playlist_state(app_state: State<'_, AppState>) -> Result<PlaylistState, String> {
    let snapshot = app_state
        .playlist
        .lock()
        .map_err(|e| format!("playlist lock poisoned: {e}"))?
        .clone();
    Ok(snapshot)
}

#[tauri::command]
pub async fn list_plugins(app_state: State<'_, AppState>) -> Result<Vec<PluginInfo>, String> {
    let snapshot = app_state
        .plugin_registry
        .lock()
        .map_err(|e| format!("plugin registry lock poisoned: {e}"))?
        .list();
    Ok(snapshot)
}

#[tauri::command]
pub async fn get_startup_fatal_error(
    app_state: State<'_, AppState>,
) -> Result<Option<StartupFatalState>, String> {
    let snapshot = app_state
        .startup_fatal
        .lock()
        .map_err(|e| format!("startup fatal lock poisoned: {e}"))?
        .clone();
    Ok(snapshot)
}

#[tauri::command]
pub async fn retry_startup_probe(
    app: AppHandle,
    app_state: State<'_, AppState>,
) -> Result<Option<StartupFatalState>, String> {
    match core::startup_probe() {
        Ok(()) => {
            let mut stored = app_state
                .startup_fatal
                .lock()
                .map_err(|e| format!("startup fatal lock poisoned: {e}"))?;
            *stored = None;
            Ok(None)
        }
        Err(startup_error) => {
            let next_fatal = StartupFatalState {
                stage: startup_error.stage.clone(),
                code: startup_error.code.clone(),
                message: startup_error.message.clone(),
                suggestion: startup_error.suggestion.clone(),
            };
            fallback::handle_startup_error(&app, &app_state, &startup_error)?;
            Ok(Some(next_fatal))
        }
    }
}

#[tauri::command]
pub async fn emit_debug_video_error(app: AppHandle) -> Result<(), String> {
    if let Err(error) = renderer::simulate_texture_alloc_failure(&app) {
        // Debug injection should not reject the frontend promise.
        // The visible signal is the emitted `video:error` event.
        eprintln!("[debug-video-error] simulated render failure: {error}");
    }
    Ok(())
}

#[tauri::command]
pub async fn emit_debug_fatal_error(app: AppHandle) -> Result<(), String> {
    fallback::emit_startup_fatal_error(
        &app,
        "libmpv_init",
        "MPV_INIT_FAILED",
        "Debug fatal startup error from backend.",
        "Please check your libmpv runtime and GPU driver compatibility.",
    )
}

#[tauri::command]
pub async fn open_log_directory(app: AppHandle) -> Result<String, String> {
    let dir = paths::app_log_dir(&app)?;
    std::fs::create_dir_all(&dir).map_err(|e| format!("create log dir failed: {e}"))?;

    #[cfg(target_os = "macos")]
    let mut command = {
        let mut cmd = std::process::Command::new("open");
        cmd.arg(&dir);
        cmd
    };

    #[cfg(target_os = "windows")]
    let mut command = {
        let mut cmd = std::process::Command::new("explorer");
        cmd.arg(&dir);
        cmd
    };

    #[cfg(all(unix, not(target_os = "macos")))]
    let mut command = {
        let mut cmd = std::process::Command::new("xdg-open");
        cmd.arg(&dir);
        cmd
    };

    command
        .status()
        .map_err(|e| format!("open log dir failed: {e}"))?;

    Ok(dir.display().to_string())
}

#[tauri::command]
pub async fn get_log_directory(app: AppHandle) -> Result<String, String> {
    let dir = paths::app_log_dir(&app)?;
    Ok(dir.display().to_string())
}

#[tauri::command]
pub async fn save_fatal_diagnostic_report(
    app: AppHandle,
    report: String,
) -> Result<String, String> {
    let dir = paths::app_log_dir(&app)?;
    std::fs::create_dir_all(&dir).map_err(|e| format!("create log dir failed: {e}"))?;

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| format!("system time before epoch: {e}"))?
        .as_secs();
    let file_name = format!("fatal-report-{timestamp}.txt");
    let path = dir.join(file_name);

    std::fs::write(&path, report).map_err(|e| format!("write fatal report failed: {e}"))?;
    if let Err(e) = reveal_file_in_file_manager(&path) {
        eprintln!("{e}");
    }
    Ok(path.display().to_string())
}
