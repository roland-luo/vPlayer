use crate::plugin::{
    Permission, Plugin, PluginContext, PluginError, PluginEvent, PluginInstance, PluginManifest,
    PluginState, UiButton, UiDescriptor, UiPopup,
};

/// Built-in bookmark plugin.
///
/// Lets users mark positions in videos and jump back to them.
/// Bookmarks are stored persistently in a JSON file in the app data directory.
pub struct BookmarkPlugin;

impl BookmarkPlugin {
    fn new() -> Self {
        Self
    }
}

impl Plugin for BookmarkPlugin {
    fn name(&self) -> &str {
        "bookmark"
    }

    fn on_load(&mut self, _ctx: &PluginContext) -> Result<(), PluginError> {
        eprintln!("[bookmark] plugin loaded");
        Ok(())
    }

    fn on_unload(&mut self) -> Result<(), PluginError> {
        eprintln!("[bookmark] plugin unloaded");
        Ok(())
    }

    fn on_event(&mut self, event: &PluginEvent) -> Result<(), PluginError> {
        match event.name() {
            "bookmark:list" | "bookmark:add" | "bookmark:remove" => {
                eprintln!("[bookmark] event tracked: {}", event.name());
                Ok(())
            }
            other => {
                eprintln!("[bookmark] ignoring unknown event: {other}");
                Ok(())
            }
        }
    }
}

/// Creates the bookmark plugin instance.
pub fn create() -> PluginInstance {
    let manifest = PluginManifest {
        name: "bookmark".to_string(),
        version: "1.0.0".to_string(),
        description: "Mark video positions and jump back to them".to_string(),
        author: "vPlayer Team".to_string(),
        permissions: vec![Permission::FileWrite],
        ui: Some(UiDescriptor {
            button: Some(UiButton {
                label: "Bookmarks".to_string(),
                icon: "bookmark".to_string(),
                position: "control_bar".to_string(),
            }),
            popup: Some(UiPopup {
                width: 420,
                height: 380,
            }),
        }),
        api_version: 1,
    };

    let mut instance = PluginInstance::new(manifest, Box::new(BookmarkPlugin::new()));
    instance.state = PluginState::Enabled;
    instance
}
