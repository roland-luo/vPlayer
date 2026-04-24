use std::path::Path;

use super::{PluginError, PluginManifest};

/// Load plugin metadata from a directory containing manifest.json.
///
/// Phase 1: only parses manifest.json to validate plugin structure.
/// Dynamic library loading (via libloading) will be fully implemented in Phase 2.
///
/// Directory layout:
///   <plugin_dir>/
///     manifest.json
///     lib<name>.dylib   (macOS)
///     lib<name>.so      (Linux)
///     <name>.dll        (Windows)
pub fn load_plugin_from_dir(dir: &Path) -> Result<PluginManifest, PluginError> {
    if !dir.is_dir() {
        return Err(PluginError::Internal(format!(
            "not a directory: {}",
            dir.display()
        )));
    }

    let manifest_path = dir.join("manifest.json");
    let manifest_content = std::fs::read_to_string(&manifest_path).map_err(|e| {
        PluginError::InvalidManifest(format!("cannot read manifest.json: {e}"))
    })?;
    let manifest: PluginManifest = serde_json::from_str(&manifest_content).map_err(|e| {
        PluginError::InvalidManifest(format!("manifest parse error: {e}"))
    })?;

    // Phase 2+: find and load the dynamic library via libloading.
    // For now, just validate the manifest exists and is parseable.
    Ok(manifest)
}

/// Register a builtin plugin that doesn't go through dynamic loading.
pub fn register_builtin(
    manifest: PluginManifest,
    plugin: Box<dyn super::Plugin>,
) -> super::PluginInstance {
    super::PluginInstance::new(manifest, plugin)
}

/// Find a dynamic library file in the plugin directory.
///
/// Platform-specific naming:
/// - macOS: lib<name>.dylib
/// - Linux: lib<name>.so
/// - Windows: <name>.dll
#[allow(dead_code)]
fn find_library_file(dir: &Path, plugin_name: &str) -> Option<std::path::PathBuf> {
    let candidates: &[String] = &[
        format!("lib{plugin_name}.dylib"),
        format!("lib{plugin_name}.so"),
        format!("{plugin_name}.dll"),
    ];

    for candidate in candidates {
        let path = dir.join(candidate);
        if path.exists() {
            return Some(path);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_plugin_from_nonexistent_dir() {
        let result = load_plugin_from_dir(Path::new("/nonexistent/plugin/dir"));
        assert!(result.is_err());
    }

    #[test]
    fn test_load_plugin_from_empty_dir() {
        let dir = std::env::temp_dir().join("vplayer_plugin_test_empty");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let result = load_plugin_from_dir(&dir);
        assert!(result.is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_load_plugin_with_invalid_manifest() {
        let dir = std::env::temp_dir().join("vplayer_plugin_test_invalid");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("manifest.json"), b"not valid json").unwrap();
        let result = load_plugin_from_dir(&dir);
        assert!(matches!(result, Err(PluginError::InvalidManifest(_))));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
