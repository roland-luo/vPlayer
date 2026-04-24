use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    pub name: String,
    pub version: String,
    pub enabled: bool,
    #[serde(default)]
    pub error_count: u32,
    #[serde(default)]
    pub last_error: Option<String>,
    /// Label for a UI button in the control bar (None = no button).
    #[serde(default)]
    pub ui_button_label: Option<String>,
    /// Icon name for the UI button (optional).
    #[serde(default)]
    pub ui_button_icon: Option<String>,
    /// Popup width from manifest (None = default 400).
    #[serde(default)]
    pub ui_popup_width: Option<u32>,
    /// Popup height from manifest (None = default 300).
    #[serde(default)]
    pub ui_popup_height: Option<u32>,
}

#[derive(Debug, Default)]
pub struct PluginRegistry {
    plugins: Vec<PluginInfo>,
}

impl PluginRegistry {
    pub fn register(&mut self, plugin: PluginInfo) {
        if let Some(existing) = self.plugins.iter_mut().find(|it| it.name == plugin.name) {
            *existing = plugin;
            return;
        }
        self.plugins.push(plugin);
    }

    pub fn unregister(&mut self, name: &str) {
        self.plugins.retain(|p| p.name != name);
    }

    pub fn set_enabled(&mut self, name: &str, enabled: bool) {
        if let Some(plugin) = self.plugins.iter_mut().find(|it| it.name == name) {
            plugin.enabled = enabled;
        }
    }

    pub fn record_error(&mut self, name: &str, error: &str) {
        if let Some(plugin) = self.plugins.iter_mut().find(|it| it.name == name) {
            plugin.error_count = plugin.error_count.saturating_add(1);
            plugin.last_error = Some(error.to_string());
        }
    }

    pub fn get(&self, name: &str) -> Option<&PluginInfo> {
        self.plugins.iter().find(|it| it.name == name)
    }

    pub fn list(&self) -> Vec<PluginInfo> {
        self.plugins.clone()
    }
}
