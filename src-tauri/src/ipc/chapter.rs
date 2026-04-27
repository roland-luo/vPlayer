use serde::{Deserialize, Serialize};
use tauri::State;

use crate::ipc::state::AppState;
use crate::plugin::PluginEvent;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChapterEntry {
    pub id: u32,
    pub title: String,
    pub start: f64,
    pub end: f64,
}

fn format_seconds(secs: f64) -> String {
    let h = (secs / 3600.0) as u32;
    let m = ((secs % 3600.0) / 60.0) as u32;
    let s = secs % 60.0;
    format!("{:02}:{:02}:{:05.2}", h, m, s)
}

/// Tauri command: extract chapter markers from the current video via ffprobe.
#[tauri::command]
pub async fn list_chapters(
    app_state: State<'_, AppState>,
) -> Result<Vec<ChapterEntry>, String> {
    let video_path = {
        let playlist = app_state.playlist.lock().map_err(|e| format!("{e}"))?;
        let idx = playlist
            .current_index
            .ok_or_else(|| "no video is playing".to_string())?;
        playlist
            .items
            .get(idx)
            .cloned()
            .ok_or_else(|| "invalid playlist index".to_string())?
    };

    let output = std::process::Command::new("ffprobe")
        .arg("-v")
        .arg("quiet")
        .arg("-print_format")
        .arg("json")
        .arg("-show_chapters")
        .arg(&video_path)
        .output()
        .map_err(|e| format!("ffprobe failed: {e}"))?;

    if !output.status.success() {
        return Err("ffprobe returned non-zero exit status".to_string());
    }

    let json: serde_json::Value = serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("parse ffprobe output: {e}"))?;

    let chapters = json["chapters"].as_array().cloned().unwrap_or_default();

    let entries: Vec<ChapterEntry> = chapters
        .iter()
        .filter_map(|ch| {
            let id = ch["id"].as_u64().unwrap_or(0) as u32;
            let start: f64 = ch["start_time"].as_str().unwrap_or("0").parse().unwrap_or(0.0);
            let end: f64 = ch["end_time"].as_str().unwrap_or("0").parse().unwrap_or(0.0);

            // Use the chapter title if available, otherwise generate one.
            let title = ch["metadata"]["title"]
                .as_str()
                .map(|s| s.to_string())
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| format!("Chapter {}", id + 1));

            Some(ChapterEntry { id, title, start, end })
        })
        .collect();

    // Emit event to plugin for tracking.
    {
        if let Ok(mut bus) = app_state.plugin_bus.lock() {
            let event = PluginEvent::with_payload(
                "chapter:list",
                serde_json::json!({
                    "video": video_path,
                    "count": entries.len()
                }),
            );
            let _ = bus.emit(&event);
        }
    }

    eprintln!(
        "[chapter] {} chapter(s) found in {}",
        entries.len(),
        video_path
    );
    Ok(entries)
}
