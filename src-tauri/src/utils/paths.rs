use std::path::PathBuf;

use tauri::{AppHandle, Manager};

pub fn app_log_dir(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_log_dir()
        .map_err(|e| format!("resolve app log dir failed: {e}"))
}

pub fn app_data_dir(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_data_dir()
        .map_err(|e| format!("resolve app data dir failed: {e}"))
}
