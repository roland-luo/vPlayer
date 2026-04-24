use std::fmt;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tauri::Manager;

pub mod builtin;
pub mod bus;
pub mod loader;
pub mod registry;
pub mod sandbox;

/// Errors that can occur during plugin lifecycle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PluginError {
    /// Plugin rejected the operation.
    Rejected(String),
    /// Plugin panicked during execution.
    Panic(String),
    /// Dynamic library could not be loaded.
    LoadFailed(String),
    /// Manifest parsing failed.
    InvalidManifest(String),
    /// Permission denied for the requested operation.
    PermissionDenied(String),
    /// The operation timed out (Phase 2+).
    Timeout(String),
    /// Other internal error.
    Internal(String),
}

impl fmt::Display for PluginError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PluginError::Rejected(msg) => write!(f, "plugin rejected: {msg}"),
            PluginError::Panic(msg) => write!(f, "plugin panicked: {msg}"),
            PluginError::LoadFailed(msg) => write!(f, "plugin load failed: {msg}"),
            PluginError::InvalidManifest(msg) => write!(f, "invalid manifest: {msg}"),
            PluginError::PermissionDenied(msg) => write!(f, "permission denied: {msg}"),
            PluginError::Timeout(msg) => write!(f, "plugin timeout: {msg}"),
            PluginError::Internal(msg) => write!(f, "plugin internal error: {msg}"),
        }
    }
}

impl std::error::Error for PluginError {}

/// Events that can be sent to plugins.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PluginEvent {
    /// A named event with optional JSON payload.
    Named {
        event: String,
        payload: serde_json::Value,
    },
    /// A simple event name with no payload.
    Simple(String),
}

impl PluginEvent {
    pub fn name(&self) -> &str {
        match self {
            PluginEvent::Named { event, .. } => event,
            PluginEvent::Simple(name) => name,
        }
    }

    pub fn named(event: &str) -> Self {
        PluginEvent::Simple(event.to_string())
    }

    pub fn with_payload(event: &str, payload: serde_json::Value) -> Self {
        PluginEvent::Named {
            event: event.to_string(),
            payload,
        }
    }
}

/// Context provided to plugins during initialization.
#[derive(Debug, Clone)]
pub struct PluginContext {
    /// Path to the plugin's data directory.
    pub data_dir: PathBuf,
    /// Path to the plugin's config file.
    pub config_path: PathBuf,
}

/// The core trait that all plugins must implement.
pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn on_load(&mut self, ctx: &PluginContext) -> Result<(), PluginError>;
    fn on_unload(&mut self) -> Result<(), PluginError>;
    fn on_event(&mut self, event: &PluginEvent) -> Result<(), PluginError>;
}

/// Plugin permissions.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Permission {
    #[serde(rename = "file_read")]
    FileRead,
    #[serde(rename = "file_write")]
    FileWrite,
    #[serde(rename = "network")]
    Network,
}

/// UI button descriptor from manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiButton {
    pub label: String,
    #[serde(default)]
    pub icon: String,
    #[serde(default = "default_button_position")]
    pub position: String,
}

fn default_button_position() -> String {
    "control_bar".to_string()
}

/// Popup configuration from manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiPopup {
    #[serde(default = "default_popup_width")]
    pub width: u32,
    #[serde(default = "default_popup_height")]
    pub height: u32,
}

fn default_popup_width() -> u32 {
    400
}
fn default_popup_height() -> u32 {
    300
}

/// UI descriptor from manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiDescriptor {
    #[serde(default)]
    pub button: Option<UiButton>,
    #[serde(default)]
    pub popup: Option<UiPopup>,
}

/// Plugin manifest parsed from manifest.json.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub permissions: Vec<Permission>,
    #[serde(default)]
    pub ui: Option<UiDescriptor>,
    pub api_version: u32,
}

/// Runtime state of a loaded plugin.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PluginState {
    Enabled,
    Disabled,
    Crashed,
}

/// A loaded plugin instance — combines the dynamic library handle with metadata.
pub struct PluginInstance {
    pub manifest: PluginManifest,
    pub state: PluginState,
    pub error_count: u32,
    pub last_error: Option<String>,
    /// The trait object — either from a builtin or from a dynamic library.
    pub plugin: Box<dyn Plugin>,
    /// Keeps the dynamic library alive if loaded from a .dylib/.so/.dll.
    #[allow(dead_code)]
    pub library: Option<libloading::Library>,
}

impl PluginInstance {
    pub fn new(manifest: PluginManifest, plugin: Box<dyn Plugin>) -> Self {
        Self {
            manifest,
            state: PluginState::Enabled,
            error_count: 0,
            last_error: None,
            plugin,
            library: None,
        }
    }

    pub fn name(&self) -> &str {
        &self.manifest.name
    }

    pub fn is_enabled(&self) -> bool {
        self.state == PluginState::Enabled
    }
}

/// Returns the path to the plugins directory.
pub fn plugins_dir(app_handle: &tauri::AppHandle) -> Result<PathBuf, PluginError> {
    let config_dir = app_handle
        .path()
        .app_config_dir()
        .map_err(|e| PluginError::Internal(format!("failed to get config dir: {e}")))?;
    Ok(config_dir.join("plugins"))
}

/// Returns the path to the plugin logs directory.
pub fn plugin_logs_dir(app_handle: &tauri::AppHandle) -> Result<PathBuf, PluginError> {
    let config_dir = app_handle
        .path()
        .app_config_dir()
        .map_err(|e| PluginError::Internal(format!("failed to get config dir: {e}")))?;
    Ok(config_dir.join("plugin-logs"))
}

/// Initialize the plugin system: load builtin plugins, scan for external plugins.
pub fn init(
    app_handle: &tauri::AppHandle,
    bus: &mut bus::PluginBus,
    registry: &mut registry::PluginRegistry,
) {
    let plugins_dir = match plugins_dir(app_handle) {
        Ok(dir) => dir,
        Err(e) => {
            eprintln!("[plugin::init] failed to resolve plugins dir: {e}");
            return;
        }
    };

    // Create directories if they don't exist.
    if let Err(e) = std::fs::create_dir_all(&plugins_dir) {
        eprintln!("[plugin::init] failed to create plugins dir: {e}");
    }
    if let Ok(logs_dir) = plugin_logs_dir(app_handle) {
        if let Err(e) = std::fs::create_dir_all(&logs_dir) {
            eprintln!("[plugin::init] failed to create plugin logs dir: {e}");
        }
    }

    // 1. Load builtin plugins.
    for mut instance in builtin::all_builtins() {
        let name = instance.name().to_string();
        let version = instance.manifest.version.clone();
        let enabled = instance.is_enabled();

        // Initialize the plugin with a context.
        let data_dir = plugins_dir.join(&name);
        let config_path = data_dir.join(format!("{name}.conf"));
        let ctx = PluginContext {
            data_dir,
            config_path,
        };

        if let Err(e) = instance.plugin.on_load(&ctx) {
            eprintln!("[plugin::init] builtin plugin '{name}' on_load failed: {e}");
            continue;
        }

        let ui_button_label = instance
            .manifest
            .ui
            .as_ref()
            .and_then(|u| u.button.as_ref())
            .map(|b| b.label.clone());
        let ui_button_icon = instance
            .manifest
            .ui
            .as_ref()
            .and_then(|u| u.button.as_ref())
            .and_then(|b| if b.icon.is_empty() { None } else { Some(b.icon.clone()) });
        let ui_popup_width = instance
            .manifest
            .ui
            .as_ref()
            .and_then(|u| u.popup.as_ref())
            .map(|p| p.width);
        let ui_popup_height = instance
            .manifest
            .ui
            .as_ref()
            .and_then(|u| u.popup.as_ref())
            .map(|p| p.height);

        bus.register_plugin(instance);
        registry.register(registry::PluginInfo {
            name: name.clone(),
            version: version.clone(),
            enabled,
            error_count: 0,
            last_error: None,
            ui_button_label,
            ui_button_icon,
            ui_popup_width,
            ui_popup_height,
        });
        eprintln!("[plugin::init] loaded builtin plugin: {name} v{version}");
    }

    // 2. Scan for external plugins from plugins directory.
    // Phase 1: validate manifest only; Phase 2: full dynamic loading.
    let entries = match std::fs::read_dir(&plugins_dir) {
        Ok(entries) => entries,
        Err(e) => {
            eprintln!("[plugin::init] failed to read plugins dir: {e}");
            return;
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let dir_name = path.file_name().unwrap_or_default().to_string_lossy().to_string();

        // Skip if a builtin with this name is already loaded.
        if bus.plugin_names().contains(&dir_name) {
            eprintln!("[plugin::init] skipping '{dir_name}': builtin with same name already loaded");
            continue;
        }

        match loader::load_plugin_from_dir(&path) {
            Ok(manifest) => {
                // Phase 2+: load the dynamic library and register the plugin instance.
                // For now, just acknowledge the manifest was found.
                eprintln!(
                    "[plugin::init] found external plugin candidate: {} v{} (dynamic loading in Phase 2)",
                    manifest.name, manifest.version
                );
            }
            Err(e) => {
                eprintln!("[plugin::init] skipped plugin dir {dir_name}: {e}");
            }
        }
    }
}

