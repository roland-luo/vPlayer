use crate::plugin::{
    Permission, Plugin, PluginContext, PluginError, PluginEvent, PluginInstance, PluginManifest,
    PluginState, UiButton, UiDescriptor, UiPopup,
};

/// Built-in playback speed plugin.
///
/// Provides a UI to control the video playback speed (0.5x ~ 2.0x).
/// Speed is applied directly to the HTML5 video element on the frontend.
pub struct PlaybackSpeedPlugin;

impl PlaybackSpeedPlugin {
    fn new() -> Self {
        Self
    }
}

impl Plugin for PlaybackSpeedPlugin {
    fn name(&self) -> &str {
        "playback-speed"
    }

    fn on_load(&mut self, _ctx: &PluginContext) -> Result<(), PluginError> {
        eprintln!("[playback-speed] plugin loaded");
        Ok(())
    }

    fn on_unload(&mut self) -> Result<(), PluginError> {
        eprintln!("[playback-speed] plugin unloaded");
        Ok(())
    }

    fn on_event(&mut self, event: &PluginEvent) -> Result<(), PluginError> {
        match event.name() {
            "speed:set" => {
                eprintln!("[playback-speed] speed change tracked");
                Ok(())
            }
            other => {
                eprintln!("[playback-speed] ignoring unknown event: {other}");
                Ok(())
            }
        }
    }
}

/// Creates the playback speed plugin instance.
pub fn create() -> PluginInstance {
    let manifest = PluginManifest {
        name: "playback-speed".to_string(),
        version: "1.0.0".to_string(),
        description: "Adjust video playback speed from 0.5x to 2.0x".to_string(),
        author: "vPlayer Team".to_string(),
        permissions: vec![],
        ui: Some(UiDescriptor {
            button: Some(UiButton {
                label: "Speed".to_string(),
                icon: "speed".to_string(),
                position: "control_bar".to_string(),
            }),
            popup: Some(UiPopup {
                width: 360,
                height: 280,
            }),
        }),
        api_version: 1,
    };

    let mut instance = PluginInstance::new(manifest, Box::new(PlaybackSpeedPlugin::new()));
    instance.state = PluginState::Enabled;
    instance
}
