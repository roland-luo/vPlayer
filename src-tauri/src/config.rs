use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

use crate::utils::paths;

/// Player settings persisted to disk.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerSettings {
    /// Volume 0~100.
    #[serde(default = "default_volume")]
    pub volume: f64,
    /// Playback speed multiplier.
    #[serde(default = "default_playback_speed")]
    pub playback_speed: f64,
    /// Window size at last close.
    #[serde(default)]
    pub window_size: Option<WindowSize>,
    /// Last playlist items.
    #[serde(default)]
    pub last_playlist: Vec<String>,
    /// Last selected playlist index.
    #[serde(default)]
    pub last_playlist_index: Option<usize>,
    /// Last playback position in seconds.
    #[serde(default)]
    pub last_position: f64,
    /// Preferred subtitle language (e.g. "zh", "en").
    #[serde(default)]
    pub preferred_subtitle_lang: Option<String>,
    /// Preferred audio language (e.g. "zh", "en").
    #[serde(default)]
    pub preferred_audio_lang: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowSize {
    pub width: u32,
    pub height: u32,
}

fn default_volume() -> f64 {
    100.0
}

fn default_playback_speed() -> f64 {
    1.0
}

impl Default for PlayerSettings {
    fn default() -> Self {
        Self {
            volume: default_volume(),
            playback_speed: default_playback_speed(),
            window_size: None,
            last_playlist: Vec::new(),
            last_playlist_index: None,
            last_position: 0.0,
            preferred_subtitle_lang: None,
            preferred_audio_lang: None,
        }
    }
}

fn settings_path(app: &AppHandle) -> Result<PathBuf, String> {
    let data_dir = paths::app_data_dir(app)?;
    Ok(data_dir.join("settings.json"))
}

/// Load settings from disk, returning defaults if the file does not exist or is corrupt.
pub fn load_settings(app: &AppHandle) -> PlayerSettings {
    let path = match settings_path(app) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("[config] failed to resolve settings path: {e}");
            return PlayerSettings::default();
        }
    };

    let bytes = match std::fs::read(&path) {
        Ok(b) => b,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            eprintln!("[config] no settings file yet, using defaults");
            return PlayerSettings::default();
        }
        Err(e) => {
            eprintln!("[config] failed to read settings file: {e}");
            return PlayerSettings::default();
        }
    };

    match serde_json::from_slice::<PlayerSettings>(&bytes) {
        Ok(s) => {
            eprintln!("[config] loaded settings from {}", path.display());
            s
        }
        Err(e) => {
            eprintln!("[config] settings file corrupt, using defaults: {e}");
            PlayerSettings::default()
        }
    }
}

/// Save settings to disk.
pub fn save_settings(app: &AppHandle, settings: &PlayerSettings) -> Result<(), String> {
    let path = settings_path(app)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("create settings dir failed: {e}"))?;
    }
    let json = serde_json::to_string_pretty(settings)
        .map_err(|e| format!("serialize settings failed: {e}"))?;
    std::fs::write(&path, json).map_err(|e| format!("write settings file failed: {e}"))?;
    eprintln!("[config] saved settings to {}", path.display());
    Ok(())
}
