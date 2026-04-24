use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    pub name: String,
    pub version: String,
    pub enabled: bool,
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

    pub fn list(&self) -> Vec<PluginInfo> {
        self.plugins.clone()
    }
}
