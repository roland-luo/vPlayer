use std::collections::HashSet;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State};
use urlencoding::encode;

use crate::ipc::bookmark::BookmarkEntry;
use crate::ipc::chapter::ChapterEntry;
use crate::ipc::commands::capture_frame_to_path;
use crate::ipc::state::AppState;

#[derive(Debug, Clone, Deserialize)]
pub struct ExportNotesRequest {
    pub video_path: String,
    pub note_ids: Vec<String>,
    pub output_dir: String,
    pub filename: String,
    pub include_screenshots: bool,
    pub group_by_chapters: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExportNotesResult {
    pub markdown_path: String,
    pub assets_dir: String,
    pub note_count: usize,
    pub screenshot_count: usize,
}

#[tauri::command]
pub async fn export_notes_to_markdown(
    app: AppHandle,
    app_state: State<'_, AppState>,
    request: ExportNotesRequest,
) -> Result<ExportNotesResult, String> {
    validate_filename(&request.filename)?;

    let video_path = resolve_video_path(&request.video_path, &app_state)?;
    let video_name = Path::new(&video_path)
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "notes".to_string());

    let output_dir = PathBuf::from(&request.output_dir);
    std::fs::create_dir_all(&output_dir)
        .map_err(|e| format!("EXPORT_PERMISSION_DENIED: create output dir failed: {e}"))?;

    let markdown_path = output_dir.join(&request.filename);
    let assets_dir = output_dir.join("assets");
    std::fs::create_dir_all(&assets_dir)
        .map_err(|e| format!("EXPORT_PERMISSION_DENIED: create assets dir failed: {e}"))?;

    let bookmarks = load_selected_bookmarks(&app, &video_path, &request.note_ids)?;
    if bookmarks.is_empty() {
        return Err("EXPORT_NO_NOTES_SELECTED: no notes selected".to_string());
    }

    let chapters = if request.group_by_chapters {
        load_chapters(&video_path)?
    } else {
        vec![]
    };

    let mut screenshot_count = 0;
    let markdown = render_markdown(
        &video_name,
        &video_path,
        &bookmarks,
        &chapters,
        request.include_screenshots,
        &assets_dir,
        &mut screenshot_count,
    )?;

    std::fs::write(&markdown_path, markdown)
        .map_err(|e| format!("EXPORT_WRITE_FAILED: {e}"))?;

    Ok(ExportNotesResult {
        markdown_path: markdown_path.to_string_lossy().to_string(),
        assets_dir: assets_dir.to_string_lossy().to_string(),
        note_count: bookmarks.len(),
        screenshot_count,
    })
}

fn validate_filename(filename: &str) -> Result<(), String> {
    if filename.is_empty() {
        return Err("EXPORT_INVALID_FILENAME: filename is empty".to_string());
    }
    if filename.contains('/') || filename.contains('\\') {
        return Err("EXPORT_INVALID_FILENAME: filename cannot contain path separators".to_string());
    }
    if !filename.ends_with(".md") {
        return Err("EXPORT_INVALID_FILENAME: filename must end with .md".to_string());
    }
    Ok(())
}

fn resolve_video_path(request_path: &str, app_state: &AppState) -> Result<String, String> {
    if !request_path.is_empty() {
        let path = PathBuf::from(request_path);
        if path.exists() {
            return Ok(request_path.to_string());
        }
        return Err("EXPORT_FILE_NOT_FOUND: video file does not exist".to_string());
    }

    let playlist = app_state.playlist.lock().map_err(|e| format!("{e}"))?;
    let idx = playlist
        .current_index
        .ok_or_else(|| "EXPORT_FILE_NOT_FOUND: no video is playing".to_string())?;
    playlist
        .items
        .get(idx)
        .cloned()
        .ok_or_else(|| "EXPORT_FILE_NOT_FOUND: invalid playlist index".to_string())
}

fn load_selected_bookmarks(
    app: &AppHandle,
    video_path: &str,
    note_ids: &[String],
) -> Result<Vec<BookmarkEntry>, String> {
    let id_set: HashSet<&str> = note_ids.iter().map(|s| s.as_str()).collect();
    let path = crate::ipc::bookmark::bookmarks_path(app)?;
    let all = crate::ipc::bookmark::load_bookmarks(&path);
    let filtered: Vec<BookmarkEntry> = all
        .into_iter()
        .filter(|b| b.video == video_path && id_set.contains(b.id.as_str()))
        .collect();
    Ok(filtered)
}

fn load_chapters(video_path: &str) -> Result<Vec<ChapterEntry>, String> {
    // We intentionally do not call the async list_chapters command here.
    // Instead, synchronously run ffprobe to avoid async command recursion.
    let output = std::process::Command::new("ffprobe")
        .arg("-v")
        .arg("quiet")
        .arg("-print_format")
        .arg("json")
        .arg("-show_chapters")
        .arg(video_path)
        .output()
        .map_err(|e| format!("ffprobe failed: {e}"))?;

    if !output.status.success() {
        return Err(format!(
            "EXPORT_CHAPTERS_FAILED: ffprobe exited with status {:?}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap_or_default();
    let chapters = json["chapters"].as_array().cloned().unwrap_or_default();

    Ok(chapters
        .iter()
        .filter_map(|ch| {
            let id = ch["id"].as_u64().unwrap_or(0) as u32;
            let start: f64 = ch["start_time"].as_str().unwrap_or("0").parse().unwrap_or(0.0);
            let end: f64 = ch["end_time"].as_str().unwrap_or("0").parse().unwrap_or(0.0);
            let title = ch["metadata"]["title"]
                .as_str()
                .map(|s| s.to_string())
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| format!("Chapter {}", id + 1));
            Some(ChapterEntry { id, title, start, end })
        })
        .collect())
}

fn render_markdown(
    video_name: &str,
    video_path: &str,
    bookmarks: &[BookmarkEntry],
    chapters: &[ChapterEntry],
    include_screenshots: bool,
    assets_dir: &Path,
    screenshot_count: &mut usize,
) -> Result<String, String> {
    let mut lines: Vec<String> = vec![
        format!("# {} 笔记", video_name),
        "".to_string(),
        format!("> 源视频：{}", video_name),
        format!("> 导出时间：{}", current_date_string()),
        "".to_string(),
        "---".to_string(),
        "".to_string(),
    ];

    let sorted_bookmarks = {
        let mut bms = bookmarks.to_vec();
        bms.sort_by(|a, b| a.position.partial_cmp(&b.position).unwrap());
        bms
    };

    if chapters.is_empty() {
        for bm in &sorted_bookmarks {
            render_note(&mut lines, video_path, bm, include_screenshots, assets_dir, screenshot_count);
        }
    } else {
        let mut remaining: Vec<&BookmarkEntry> = sorted_bookmarks.iter().collect();

        for (idx, chapter) in chapters.iter().enumerate() {
            lines.push(format!("## {:02}. {}", idx + 1, chapter.title));
            lines.push("".to_string());

            let (in_chapter, rest): (Vec<_>, Vec<_>) = remaining
                .into_iter()
                .partition(|bm| bm.position >= chapter.start && bm.position < chapter.end);

            for bm in in_chapter {
                render_note(&mut lines, video_path, bm, include_screenshots, assets_dir, screenshot_count);
            }

            remaining = rest;
        }

        if !remaining.is_empty() {
            lines.push("## 未分类".to_string());
            lines.push("".to_string());
            for bm in remaining {
                render_note(&mut lines, video_path, bm, include_screenshots, assets_dir, screenshot_count);
            }
        }
    }

    Ok(lines.join("\n"))
}

fn render_note(
    lines: &mut Vec<String>,
    video_path: &str,
    bm: &BookmarkEntry,
    include_screenshots: bool,
    assets_dir: &Path,
    screenshot_count: &mut usize,
) {
    let time_str = format_seconds(bm.position);
    let t_seconds = bm.position.max(0.0) as u64;
    let link = format!("vplayer://open?file={}&t={}", encode(video_path), t_seconds);

    lines.push(format!("### [{}]({}) {}", time_str, link, bm.name));
    lines.push("".to_string());

    if include_screenshots {
        let image_name = format!("{}.png", time_str.replace(':', "_"));
        let image_path = assets_dir.join(&image_name);
        match capture_frame_to_path(video_path, bm.position, &image_path) {
            Ok(()) => {
                *screenshot_count += 1;
                lines.push(format!("![screenshot](assets/{})", image_name));
            }
            Err(e) => {
                eprintln!("[tutorial-export] screenshot failed at {}: {}", time_str, e);
                lines.push(format!("> 截图失败：{}", e));
            }
        }
    }
}

pub fn format_seconds(secs: f64) -> String {
    let h = (secs / 3600.0) as u32;
    let m = ((secs % 3600.0) / 60.0) as u32;
    let s = (secs % 60.0) as u32;
    format!("{:02}:{:02}:{:02}", h, m, s)
}

fn current_date_string() -> String {
    chrono::Local::now().format("%Y-%m-%d").to_string()
}
