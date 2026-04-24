use crate::plugin::{
    Permission, Plugin, PluginContext, PluginError, PluginEvent, PluginInstance, PluginManifest,
    PluginState, UiButton, UiDescriptor, UiPopup,
};

/// Built-in media info plugin.
///
/// Displays detailed technical information about the currently playing video
/// by running ffprobe and presenting the results in the plugin popup.
pub struct MediaInfoPlugin;

impl MediaInfoPlugin {
    fn new() -> Self {
        Self
    }
}

impl Plugin for MediaInfoPlugin {
    fn name(&self) -> &str {
        "media-info"
    }

    fn on_load(&mut self, _ctx: &PluginContext) -> Result<(), PluginError> {
        eprintln!("[media-info] plugin loaded");
        Ok(())
    }

    fn on_unload(&mut self) -> Result<(), PluginError> {
        eprintln!("[media-info] plugin unloaded");
        Ok(())
    }

    fn on_event(&mut self, event: &PluginEvent) -> Result<(), PluginError> {
        match event.name() {
            "mediainfo:request" => {
                eprintln!("[media-info] info requested (tracked)");
                Ok(())
            }
            other => {
                eprintln!("[media-info] ignoring unknown event: {other}");
                Ok(())
            }
        }
    }
}

/// Creates the media info plugin instance.
pub fn create() -> PluginInstance {
    let manifest = PluginManifest {
        name: "media-info".to_string(),
        version: "1.0.0".to_string(),
        description: "Display detailed media file information".to_string(),
        author: "vPlayer Team".to_string(),
        permissions: vec![],
        ui: Some(UiDescriptor {
            button: Some(UiButton {
                label: "Info".to_string(),
                icon: "info".to_string(),
                position: "control_bar".to_string(),
            }),
            popup: Some(UiPopup {
                width: 420,
                height: 380,
            }),
        }),
        api_version: 1,
    };

    let mut instance = PluginInstance::new(manifest, Box::new(MediaInfoPlugin::new()));
    instance.state = PluginState::Enabled;
    instance
}
