use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State};

use crate::ipc::state::AppState;
use crate::plugin::PluginEvent;
use crate::utils;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookmarkEntry {
    pub id: String,
    pub name: String,
    pub video: String,
    pub position: f64,
    pub created_at: u64,
}

fn bookmarks_path(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = utils::paths::app_data_dir(app)?;
    std::fs::create_dir_all(&dir).map_err(|e| format!("create data dir: {e}"))?;
    Ok(dir.join("bookmarks.json"))
}

fn load_bookmarks(path: &PathBuf) -> Vec<BookmarkEntry> {
    if !path.exists() {
        return vec![];
    }
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return vec![],
    };
    serde_json::from_str(&content).unwrap_or_default()
}

fn save_bookmarks(path: &PathBuf, bookmarks: &[BookmarkEntry]) -> Result<(), String> {
    let content =
        serde_json::to_string_pretty(bookmarks).map_err(|e| format!("serialize: {e}"))?;
    std::fs::write(path, content).map_err(|e| format!("write bookmarks: {e}"))
}

fn generate_id() -> String {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("bm_{ts}")
}

/// Tauri command: list bookmarks for the currently playing video.
#[tauri::command]
pub async fn list_bookmarks(
    app: AppHandle,
    app_state: State<'_, AppState>,
) -> Result<Vec<BookmarkEntry>, String> {
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

    let path = bookmarks_path(&app)?;
    let all = load_bookmarks(&path);
    let filtered: Vec<BookmarkEntry> = all.into_iter().filter(|b| b.video == video_path).collect();

    // Emit event to plugin for tracking.
    {
        if let Ok(mut bus) = app_state.plugin_bus.lock() {
            let event = PluginEvent::with_payload(
                "bookmark:list",
                serde_json::json!({ "video": video_path, "count": filtered.len() }),
            );
            let _ = bus.emit(&event);
        }
    }

    Ok(filtered)
}

/// Tauri command: add a bookmark at the current playback position.
#[tauri::command]
pub async fn add_bookmark(
    app: AppHandle,
    app_state: State<'_, AppState>,
    name: String,
) -> Result<BookmarkEntry, String> {
    let (video_path, position) = {
        let playlist = app_state.playlist.lock().map_err(|e| format!("{e}"))?;
        let player = app_state.player.lock().map_err(|e| format!("{e}"))?;
        let idx = playlist
            .current_index
            .ok_or_else(|| "no video is playing".to_string())?;
        let path = playlist
            .items
            .get(idx)
            .cloned()
            .ok_or_else(|| "invalid playlist index".to_string())?;
        (path, player.position)
    };

    let entry = BookmarkEntry {
        id: generate_id(),
        name,
        video: video_path,
        position,
        created_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
    };

    let path = bookmarks_path(&app)?;
    let mut all = load_bookmarks(&path);
    all.push(entry.clone());
    save_bookmarks(&path, &all)?;

    // Emit event to plugin for tracking.
    {
        if let Ok(mut bus) = app_state.plugin_bus.lock() {
            let event = PluginEvent::with_payload(
                "bookmark:add",
                serde_json::json!({ "id": entry.id, "name": entry.name }),
            );
            let _ = bus.emit(&event);
        }
    }

    eprintln!(
        "[bookmark] added '{}' at {:.1}s for {}",
        entry.name, entry.position, entry.video
    );
    Ok(entry)
}

/// Tauri command: delete a bookmark by ID.
#[tauri::command]
pub async fn delete_bookmark(
    app: AppHandle,
    app_state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    let path = bookmarks_path(&app)?;
    let mut all = load_bookmarks(&path);
    all.retain(|b| b.id != id);
    save_bookmarks(&path, &all)?;

    // Emit event to plugin for tracking.
    {
        if let Ok(mut bus) = app_state.plugin_bus.lock() {
            let event =
                PluginEvent::with_payload("bookmark:remove", serde_json::json!({ "id": id }));
            let _ = bus.emit(&event);
        }
    }

    eprintln!("[bookmark] deleted id={id}");
    Ok(())
}
