use std::sync::Mutex;

use serde::{Deserialize, Serialize};

use crate::config::PlayerSettings;
use crate::plugin::{bus::PluginBus, registry::PluginRegistry};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerState {
    pub state: String,
    pub position: f64,
    pub duration: f64,
    /// Volume range is fixed to 0~100 in Week 1.
    pub volume: f64,
}

impl Default for PlayerState {
    fn default() -> Self {
        Self {
            state: "idle".to_string(),
            position: 0.0,
            duration: 0.0,
            volume: 100.0,
        }
    }
}

pub struct AppState {
    pub player: Mutex<PlayerState>,
    pub playlist: Mutex<PlaylistState>,
    pub startup_fatal: Mutex<Option<StartupFatalState>>,
    pub plugin_bus: Mutex<PluginBus>,
    pub plugin_registry: Mutex<PluginRegistry>,
    pub settings: Mutex<PlayerSettings>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlaylistState {
    pub items: Vec<String>,
    pub current_index: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartupFatalState {
    pub stage: String,
    pub code: String,
    pub message: String,
    pub suggestion: String,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            player: Mutex::new(PlayerState::default()),
            playlist: Mutex::new(PlaylistState::default()),
            startup_fatal: Mutex::new(None),
            plugin_bus: Mutex::new(PluginBus::default()),
            plugin_registry: Mutex::new(PluginRegistry::default()),
            settings: Mutex::new(PlayerSettings::default()),
        }
    }
}
