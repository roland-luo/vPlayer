pub mod bookmark;
pub mod media_info;
pub mod screenshot;
pub mod speed;
pub mod subtitle_download;

use super::PluginEvent;

/// Iterates all builtin plugin factories and returns their instances.
/// Each factory registers itself here for startup loading.
pub fn all_builtins() -> Vec<super::PluginInstance> {
    vec![
        screenshot::create(),
        subtitle_download::create(),
        media_info::create(),
        speed::create(),
        bookmark::create(),
    ]
}

/// Handle a builtin-specific event dispatch.
/// Builtins are always loaded, so no loading/unloading logic here.
pub fn dispatch_to_builtin(
    instance: &mut super::PluginInstance,
    event: &PluginEvent,
) -> Result<(), super::PluginError> {
    if !instance.is_enabled() {
        return Ok(());
    }
    instance.plugin.on_event(event)
}
