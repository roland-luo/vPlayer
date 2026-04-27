use crate::plugin::{
    Plugin, PluginContext, PluginError, PluginEvent, PluginInstance, PluginManifest, PluginState,
    UiButton, UiDescriptor, UiPopup,
};

/// Built-in audio equalizer plugin.
///
/// Provides an 8-band graphic equalizer using the Web Audio API.
/// All audio processing happens on the frontend — this plugin
/// exists for manifest registration and event tracking.
pub struct EqualizerPlugin;

impl EqualizerPlugin {
    fn new() -> Self {
        Self
    }
}

impl Plugin for EqualizerPlugin {
    fn name(&self) -> &str {
        "equalizer"
    }

    fn on_load(&mut self, _ctx: &PluginContext) -> Result<(), PluginError> {
        eprintln!("[equalizer] plugin loaded");
        Ok(())
    }

    fn on_unload(&mut self) -> Result<(), PluginError> {
        eprintln!("[equalizer] plugin unloaded");
        Ok(())
    }

    fn on_event(&mut self, event: &PluginEvent) -> Result<(), PluginError> {
        match event.name() {
            "equalizer:preset" => {
                eprintln!("[equalizer] preset change tracked");
                Ok(())
            }
            other => {
                eprintln!("[equalizer] ignoring unknown event: {other}");
                Ok(())
            }
        }
    }
}

pub fn create() -> PluginInstance {
    let manifest = PluginManifest {
        name: "equalizer".to_string(),
        version: "1.0.0".to_string(),
        description: "8-band audio equalizer with presets".to_string(),
        author: "vPlayer Team".to_string(),
        permissions: vec![],
        ui: Some(UiDescriptor {
            button: Some(UiButton {
                label: "EQ".to_string(),
                icon: "equalizer".to_string(),
                position: "control_bar".to_string(),
            }),
            popup: Some(UiPopup {
                width: 520,
                height: 320,
            }),
        }),
        api_version: 1,
    };

    let mut instance = PluginInstance::new(manifest, Box::new(EqualizerPlugin::new()));
    instance.state = PluginState::Enabled;
    instance
}
