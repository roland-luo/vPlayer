use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State};

use crate::ipc::state::AppState;
use crate::plugin::PluginEvent;

/// A single subtitle search result returned to the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubtitleSearchResult {
    pub name: String,
    pub language: String,
    pub format: String,
    /// "local" or "online"
    pub source: String,
    /// Absolute file path (local) or download URL (online).
    pub path: String,
}

/// Simple language detection based on common subtitle filename tags.
fn detect_language(filename: &str) -> String {
    let lower = filename.to_lowercase();
    if lower.ends_with(".en") || lower.ends_with(".eng") {
        return "English".into();
    }
    if lower.ends_with(".zh")
        || lower.ends_with(".chs")
        || lower.ends_with(".cht")
        || lower.ends_with(".chi")
    {
        return "中文".into();
    }
    if lower.ends_with(".ja") || lower.ends_with(".jpn") {
        return "日本語".into();
    }
    if lower.ends_with(".ko") || lower.ends_with(".kor") {
        return "한국어".into();
    }
    if lower.ends_with(".fr") || lower.ends_with(".fra") {
        return "Français".into();
    }
    if lower.ends_with(".de") || lower.ends_with(".deu") {
        return "Deutsch".into();
    }
    if lower.ends_with(".es") || lower.ends_with(".spa") {
        return "Español".into();
    }
    if lower.ends_with(".ru") || lower.ends_with(".rus") {
        return "Русский".into();
    }
    if lower.ends_with(".pt") || lower.ends_with(".por") {
        return "Português".into();
    }
    if lower.ends_with(".it") || lower.ends_with(".ita") {
        return "Italiano".into();
    }
    if lower.ends_with(".ar") || lower.ends_with(".ara") {
        return "العربية".into();
    }
    "Unknown".into()
}

/// Search for subtitle files in the same directory as the video.
fn search_local_subtitles(video_dir: &Path, video_stem: &str) -> Vec<SubtitleSearchResult> {
    let mut results = Vec::new();
    let extensions = ["srt", "ass", "ssa", "vtt", "sub"];
    let search_term = video_stem.to_lowercase();

    let entries = match std::fs::read_dir(video_dir) {
        Ok(e) => e,
        Err(_) => return results,
    };

    for entry in entries.flatten() {
        let entry_path = entry.path();
        if !entry_path.is_file() {
            continue;
        }

        let ext = match entry_path.extension().and_then(|e| e.to_str()) {
            Some(e) => e.to_lowercase(),
            None => continue,
        };
        if !extensions.contains(&ext.as_str()) {
            continue;
        }

        let file_stem = match entry_path.file_stem().and_then(|s| s.to_str()) {
            Some(s) => s,
            None => continue,
        };

        let stem_lower = file_stem.to_lowercase();

        // Match: same name, or name with language tag (e.g. video.en.srt)
        let matches = stem_lower == search_term
            || stem_lower.starts_with(&format!("{}.", search_term))
            || stem_lower.starts_with(&format!("{} ", search_term))
            || stem_lower.starts_with(&format!("{}-", search_term));

        if !matches {
            continue;
        }

        let language = detect_language(file_stem);
        let name = entry_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        results.push(SubtitleSearchResult {
            name,
            language,
            format: ext.to_uppercase(),
            source: "local".into(),
            path: entry_path.to_string_lossy().to_string(),
        });
    }

    results
}

/// Tauri command: search for subtitles for the currently playing video.
///
/// If `query` is provided, it overrides the auto-derived search term.
#[tauri::command]
pub async fn search_subtitles(
    query: Option<String>,
    app: AppHandle,
    app_state: State<'_, AppState>,
) -> Result<Vec<SubtitleSearchResult>, String> {
    let (video_path, video_dir, video_stem) = {
        let playlist = app_state.playlist.lock().map_err(|e| format!("{e}"))?;
        let current_index = playlist
            .current_index
            .ok_or_else(|| "no video is playing".to_string())?;
        let current_path = playlist
            .items
            .get(current_index)
            .ok_or_else(|| "invalid playlist index".to_string())?;

        let path = Path::new(current_path);
        let dir = path
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."));
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();

        (current_path.clone(), dir, stem)
    };

    let search_term = query.as_deref().unwrap_or(&video_stem);

    let mut results = search_local_subtitles(&video_dir, &video_stem);

    // Emit event to subtitle plugin for tracking.
    {
        let mut bus = app_state.plugin_bus.lock().map_err(|e| format!("{e}"))?;
        let event = PluginEvent::with_payload(
            "subtitle:search",
            serde_json::json!({
                "video": video_path,
                "query": search_term,
                "results_count": results.len(),
            }),
        );
        let _report = bus.emit(&event);
    }

    eprintln!(
        "[subtitle] searched for '{search_term}': {} local result(s)",
        results.len()
    );
    Ok(results)
}

/// Tauri command: download (copy) a local subtitle file to the video directory.
#[tauri::command]
pub async fn download_subtitle(
    source_path: String,
    app: AppHandle,
    app_state: State<'_, AppState>,
) -> Result<String, String> {
    let video_path = {
        let playlist = app_state.playlist.lock().map_err(|e| format!("{e}"))?;
        let current_index = playlist
            .current_index
            .ok_or_else(|| "no video is playing".to_string())?;
        playlist
            .items
            .get(current_index)
            .ok_or_else(|| "invalid playlist index".to_string())?
            .clone()
    };

    let video = Path::new(&video_path);
    let video_dir = video.parent().unwrap_or(Path::new("."));
    let video_stem = video
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("subtitle");

    let source = Path::new(&source_path);
    let ext = source
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("srt");
    let dest_name = format!("{}.{}", video_stem, ext);
    let dest_path = video_dir.join(&dest_name);

    std::fs::copy(&source_path, &dest_path)
        .map_err(|e| format!("failed to copy subtitle: {e}"))?;

    // Emit event to plugin for tracking.
    {
        let mut bus = app_state.plugin_bus.lock().map_err(|e| format!("{e}"))?;
        let event = PluginEvent::with_payload(
            "subtitle:download",
            serde_json::json!({
                "source": source_path,
                "destination": dest_path.to_string_lossy(),
            }),
        );
        let _report = bus.emit(&event);
    }

    eprintln!(
        "[subtitle] saved: {} -> {}",
        source_path,
        dest_path.display()
    );
    Ok(dest_path.to_string_lossy().to_string())
}
