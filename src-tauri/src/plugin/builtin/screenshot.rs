use crate::plugin::{Permission, Plugin, PluginContext, PluginError, PluginEvent, PluginInstance, PluginManifest, PluginState, UiButton, UiDescriptor};

/// Built-in screenshot plugin.
///
/// Captures video frames from the frontend and saves as PNG via backend IPC.
/// The actual frame capture happens in the frontend (Canvas API on the `<video>` element),
/// then the image data is sent to the backend for file storage.
pub struct ScreenshotPlugin {
    /// Total number of successful captures in this session.
    captures: u32,
    /// Path of the most recent screenshot.
    last_path: Option<String>,
}

impl ScreenshotPlugin {
    fn new() -> Self {
        Self {
            captures: 0,
            last_path: None,
        }
    }
}

impl Plugin for ScreenshotPlugin {
    fn name(&self) -> &str {
        "screenshot"
    }

    fn on_load(&mut self, _ctx: &PluginContext) -> Result<(), PluginError> {
        eprintln!("[screenshot] plugin loaded");
        Ok(())
    }

    fn on_unload(&mut self) -> Result<(), PluginError> {
        eprintln!(
            "[screenshot] plugin unloaded ({} captures this session)",
            self.captures
        );
        Ok(())
    }

    fn on_event(&mut self, event: &PluginEvent) -> Result<(), PluginError> {
        match event.name() {
            "screenshot:captured" => {
                self.captures += 1;
                // Extract the file path from the event payload if present
                if let PluginEvent::Named { payload, .. } = event {
                    if let Some(path) = payload.get("path").and_then(|v| v.as_str()) {
                        self.last_path = Some(path.to_string());
                        eprintln!(
                            "[screenshot] capture #{} saved to: {}",
                            self.captures, path
                        );
                    }
                } else {
                    eprintln!("[screenshot] capture #{} completed", self.captures);
                }
                Ok(())
            }
            other => {
                eprintln!("[screenshot] ignoring unknown event: {other}");
                Ok(())
            }
        }
    }
}

/// Creates the screenshot plugin instance.
pub fn create() -> PluginInstance {
    let manifest = PluginManifest {
        name: "screenshot".to_string(),
        version: "1.0.0".to_string(),
        description: "Capture video frames as PNG images".to_string(),
        author: "vPlayer Team".to_string(),
        permissions: vec![Permission::FileWrite],
        ui: Some(UiDescriptor {
            button: Some(UiButton {
                label: "Screenshot".to_string(),
                icon: "camera".to_string(),
                position: "control_bar".to_string(),
            }),
            popup: None,
        }),
        api_version: 1,
    };

    let mut instance = PluginInstance::new(manifest, Box::new(ScreenshotPlugin::new()));
    instance.state = PluginState::Enabled;
    instance
}
