use crate::plugin::{
    Plugin, PluginContext, PluginError, PluginEvent, PluginInstance, PluginManifest, PluginState,
    UiButton, UiDescriptor, UiPopup,
};

/// Built-in chapter navigation plugin.
///
/// Reads chapter markers from the video file via ffprobe and
/// presents them in the plugin popup for quick navigation.
pub struct ChapterPlugin;

impl ChapterPlugin {
    fn new() -> Self {
        Self
    }
}

impl Plugin for ChapterPlugin {
    fn name(&self) -> &str {
        "chapter"
    }

    fn on_load(&mut self, _ctx: &PluginContext) -> Result<(), PluginError> {
        eprintln!("[chapter] plugin loaded");
        Ok(())
    }

    fn on_unload(&mut self) -> Result<(), PluginError> {
        eprintln!("[chapter] plugin unloaded");
        Ok(())
    }

    fn on_event(&mut self, event: &PluginEvent) -> Result<(), PluginError> {
        match event.name() {
            "chapter:list" => {
                eprintln!("[chapter] list requested (tracked)");
                Ok(())
            }
            other => {
                eprintln!("[chapter] ignoring unknown event: {other}");
                Ok(())
            }
        }
    }
}

pub fn create() -> PluginInstance {
    let manifest = PluginManifest {
        name: "chapter".to_string(),
        version: "1.0.0".to_string(),
        description: "Navigate video chapters".to_string(),
        author: "vPlayer Team".to_string(),
        permissions: vec![],
        ui: Some(UiDescriptor {
            button: Some(UiButton {
                label: "Chapters".to_string(),
                icon: "chapter".to_string(),
                position: "control_bar".to_string(),
            }),
            popup: Some(UiPopup {
                width: 420,
                height: 360,
            }),
        }),
        api_version: 1,
    };

    let mut instance = PluginInstance::new(manifest, Box::new(ChapterPlugin::new()));
    instance.state = PluginState::Enabled;
    instance
}
