use tauri::{AppHandle, State};

use crate::config::{load_settings, save_settings, PlayerSettings};
use crate::ipc::state::AppState;

#[tauri::command]
pub async fn load_player_settings(app: AppHandle) -> Result<PlayerSettings, String> {
    Ok(load_settings(&app))
}

#[tauri::command]
pub async fn save_player_settings(
    app: AppHandle,
    settings: PlayerSettings,
) -> Result<(), String> {
    save_settings(&app, &settings)
}

#[tauri::command]
pub async fn get_player_settings(app_state: State<'_, AppState>) -> Result<PlayerSettings, String> {
    let snapshot = app_state
        .settings
        .lock()
        .map_err(|e| format!("settings lock poisoned: {e}"))?
        .clone();
    Ok(snapshot)
}

#[tauri::command]
pub async fn update_player_settings(
    app: AppHandle,
    app_state: State<'_, AppState>,
    settings: PlayerSettings,
) -> Result<(), String> {
    {
        let mut stored = app_state
            .settings
            .lock()
            .map_err(|e| format!("settings lock poisoned: {e}"))?;
        *stored = settings.clone();
    }
    save_settings(&app, &settings)?;
    Ok(())
}
