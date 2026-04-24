use crate::plugin::{
    Permission, Plugin, PluginContext, PluginError, PluginEvent, PluginInstance, PluginManifest,
    PluginState, UiButton, UiDescriptor,
};

/// Built-in subtitle download plugin.
///
/// Searches local files and downloads subtitles for the current video.
pub struct SubtitleDownloadPlugin {
    searches: u32,
    downloads: u32,
}

impl SubtitleDownloadPlugin {
    fn new() -> Self {
        Self {
            searches: 0,
            downloads: 0,
        }
    }
}

impl Plugin for SubtitleDownloadPlugin {
    fn name(&self) -> &str {
        "subtitle-download"
    }

    fn on_load(&mut self, _ctx: &PluginContext) -> Result<(), PluginError> {
        eprintln!("[subtitle-download] plugin loaded");
        Ok(())
    }

    fn on_unload(&mut self) -> Result<(), PluginError> {
        eprintln!(
            "[subtitle-download] plugin unloaded ({} searches, {} downloads)",
            self.searches, self.downloads
        );
        Ok(())
    }

    fn on_event(&mut self, event: &PluginEvent) -> Result<(), PluginError> {
        match event.name() {
            "subtitle:search" => {
                self.searches += 1;
                eprintln!("[subtitle-download] search #{} (tracked)", self.searches);
                Ok(())
            }
            "subtitle:download" => {
                self.downloads += 1;
                eprintln!(
                    "[subtitle-download] download #{} (tracked)",
                    self.downloads
                );
                Ok(())
            }
            other => {
                eprintln!("[subtitle-download] ignoring unknown event: {other}");
                Ok(())
            }
        }
    }
}

/// Creates the subtitle download plugin instance.
pub fn create() -> PluginInstance {
    let manifest = PluginManifest {
        name: "subtitle-download".to_string(),
        version: "1.0.0".to_string(),
        description: "Search and download subtitles automatically".to_string(),
        author: "vPlayer Team".to_string(),
        permissions: vec![Permission::Network, Permission::FileWrite],
        ui: Some(UiDescriptor {
            button: Some(UiButton {
                label: "Subtitles".to_string(),
                icon: "subtitles".to_string(),
                position: "control_bar".to_string(),
            }),
            popup: Some(crate::plugin::UiPopup {
                width: 500,
                height: 400,
            }),
        }),
        api_version: 1,
    };

    let mut instance = PluginInstance::new(manifest, Box::new(SubtitleDownloadPlugin::new()));
    instance.state = PluginState::Enabled;
    instance
}
