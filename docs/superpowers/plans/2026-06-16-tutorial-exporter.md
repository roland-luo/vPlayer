# Tutorial Exporter Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a Tutorial Exporter feature that converts timestamped notes into a GitHub-ready Markdown file with optional screenshots and chapter grouping.

**Architecture:** Reuse existing bookmark/chapter/screenshot data. Add a backend Tauri command `export_notes_to_markdown` that generates the Markdown and assets, plus a frontend modal for selecting notes and configuring export options.

**Tech Stack:** Tauri 2 + Rust, Vue 3 + TypeScript, ffmpeg, rfd.

---

## File Structure

| File | Action | Responsibility |
|------|--------|----------------|
| `src-tauri/src/ipc/commands.rs` | Modify | Extract reusable frame-capture function from `capture_screenshot` |
| `src-tauri/src/ipc/tutorial_export.rs` | Create | `export_notes_to_markdown` command and Markdown rendering |
| `src-tauri/src/ipc/mod.rs` | Modify | Register `tutorial_export` module |
| `src-tauri/src/main.rs` | Modify | Register `export_notes_to_markdown` in Tauri invoke handler |
| `frontend/src/api/tutorialExport.ts` | Create | Frontend IPC wrapper |
| `frontend/src/components/TutorialExporterModal.vue` | Create | Export modal UI |
| `frontend/src/components/BookmarkView.vue` | Modify | Change Export button to open modal instead of clipboard copy |
| `frontend/src/App.vue` | Modify | Manage modal visibility and pass video state |

---

## Task 1: Extract reusable frame-capture helper

**Files:**
- Modify: `src-tauri/src/ipc/commands.rs`

**Why:** The existing `capture_screenshot` command shows a Save As dialog and saves to a user-chosen path. The exporter needs to capture frames to deterministic paths without UI.

- [ ] **Step 1: Add `capture_frame_to_path` helper**

Add this function in `src-tauri/src/ipc/commands.rs`, immediately before the `capture_screenshot` command:

```rust
/// Extract a single frame from `video_path` at `position` seconds into `output_path`.
/// Returns Ok(()) on success. Uses ffmpeg.
pub fn capture_frame_to_path(
    video_path: &str,
    position: f64,
    output_path: &std::path::Path,
) -> Result<(), String> {
    let clamped_position = position.max(0.0);
    let output = std::process::Command::new("ffmpeg")
        .arg("-ss")
        .arg(format!("{:.3}", clamped_position))
        .arg("-i")
        .arg(video_path)
        .arg("-vframes")
        .arg("1")
        .arg("-q:v")
        .arg("2")
        .arg("-y")
        .arg(output_path)
        .output()
        .map_err(|e| format!("failed to execute ffmpeg: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("ffmpeg failed: {stderr}"));
    }

    Ok(())
}
```

- [ ] **Step 2: Refactor `capture_screenshot` to use the helper**

Replace the ffmpeg block inside `capture_screenshot` (around lines 457-475) with a call to the new helper:

```rust
    // Extract frame to a temp file via ffmpeg.
    let temp_dir = std::env::temp_dir();
    let temp_path = temp_dir.join(&default_name);
    capture_frame_to_path(&path, position, &temp_path)?;
```

Keep the rest of `capture_screenshot` (Save As dialog, copy to final path, plugin event) unchanged.

- [ ] **Step 3: Verify backend compiles**

Run:

```bash
cd /Users/roland/Desktop/appsRoland/vPlayer/src-tauri
rtk cargo check
```

Expected: no errors.

- [ ] **Step 4: Commit**

```bash
rtk git add src-tauri/src/ipc/commands.rs
rtk git commit -m "refactor: extract reusable frame capture helper for exporter"
```

---

## Task 2: Implement backend Markdown export

**Files:**
- Create: `src-tauri/src/ipc/tutorial_export.rs`

- [ ] **Step 1: Create the file with types and command**

```rust
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
        load_chapters(&app_state)?
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
    let path = crate::ipc::bookmark::bookmarks_path(app)?;
    let all = crate::ipc::bookmark::load_bookmarks(&path);
    let filtered: Vec<BookmarkEntry> = all
        .into_iter()
        .filter(|b| b.video == video_path && note_ids.contains(&b.id))
        .collect();
    Ok(filtered)
}

fn load_chapters(app_state: &AppState) -> Result<Vec<ChapterEntry>, String> {
    // We intentionally do not call the async list_chapters command here.
    // Instead, synchronously run ffprobe to avoid async command recursion.
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
        return Ok(vec![]);
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
                lines.push(format!("![screenshot](assets/{})\n", image_name));
            }
            Err(e) => {
                eprintln!("[tutorial-export] screenshot failed at {}: {}", time_str, e);
                lines.push(format!("> 截图失败：{}\n", e));
            }
        }
    }
}

fn format_seconds(secs: f64) -> String {
    let h = (secs / 3600.0) as u32;
    let m = ((secs % 3600.0) / 60.0) as u32;
    let s = (secs % 60.0) as u32;
    format!("{:02}:{:02}:{:02}", h, m, s)
}

fn current_date_string() -> String {
    chrono::Local::now().format("%Y-%m-%d").to_string()
}
```

- [ ] **Step 2: Add required Rust dependencies**

Add to `src-tauri/Cargo.toml` under `[dependencies]`:

```toml
urlencoding = "2"
chrono = "0.4"
tauri-plugin-dialog = "2"
```

Run:

```bash
cd /Users/roland/Desktop/appsRoland/vPlayer/src-tauri
rtk cargo add urlencoding chrono tauri-plugin-dialog
```

- [ ] **Step 3: Make `bookmarks_path` and `load_bookmarks` public**

In `src-tauri/src/ipc/bookmark.rs`, change:

```rust
pub fn bookmarks_path(app: &AppHandle) -> Result<PathBuf, String> {
```

and

```rust
pub fn load_bookmarks(path: &PathBuf) -> Vec<BookmarkEntry> {
```

(They are currently module-private.)

- [ ] **Step 4: Verify backend compiles**

Run:

```bash
cd /Users/roland/Desktop/appsRoland/vPlayer/src-tauri
rtk cargo check
```

Expected: no errors.

- [ ] **Step 5: Commit**

```bash
rtk git add src-tauri/src/ipc/tutorial_export.rs src-tauri/src/ipc/bookmark.rs src-tauri/Cargo.toml
rtk git commit -m "feat: add backend export_notes_to_markdown command"
```

---

## Task 3: Register the new command

**Files:**
- Modify: `src-tauri/src/ipc/mod.rs`
- Modify: `src-tauri/src/main.rs`

- [ ] **Step 1: Register module**

In `src-tauri/src/ipc/mod.rs`, add:

```rust
pub mod tutorial_export;
```

- [ ] **Step 2: Register handler and initialize dialog plugin**

In `src-tauri/src/main.rs`, add `ipc::tutorial_export::export_notes_to_markdown` to the `generate_handler!` macro list (e.g., after `ipc::chapter::list_chapters`).

Also initialize the dialog plugin in the builder chain (before `.manage`):

```rust
.plugin(tauri_plugin_dialog::init())
```

- [ ] **Step 3: Verify compile**

```bash
cd /Users/roland/Desktop/appsRoland/vPlayer/src-tauri
rtk cargo check
```

- [ ] **Step 4: Commit**

```bash
rtk git add src-tauri/src/ipc/mod.rs src-tauri/src/main.rs
rtk git commit -m "chore: register export_notes_to_markdown command"
```

---

## Task 4: Frontend API wrapper

**Files:**
- Create: `frontend/src/api/tutorialExport.ts`

- [ ] **Step 1: Create the API file**

```typescript
import { invoke } from "@tauri-apps/api/core";

export type ExportNotesRequest = {
  video_path: string;
  note_ids: string[];
  output_dir: string;
  filename: string;
  include_screenshots: boolean;
  group_by_chapters: boolean;
};

export type ExportNotesResult = {
  markdown_path: string;
  assets_dir: string;
  note_count: number;
  screenshot_count: number;
};

export async function exportNotesToMarkdown(
  request: ExportNotesRequest,
): Promise<ExportNotesResult> {
  return invoke("export_notes_to_markdown", { request });
}
```

- [ ] **Step 2: Commit**

```bash
rtk git add frontend/src/api/tutorialExport.ts
rtk git commit -m "feat: add frontend tutorial export API wrapper"
```

---

## Task 5: Build TutorialExporterModal component

**Files:**
- Create: `frontend/src/components/TutorialExporterModal.vue`

- [ ] **Step 1: Create the component**

```vue
<template>
  <Teleport to="body">
    <div v-if="visible" class="tutor-export-overlay" @click.self="close">
      <div class="tutor-export-card">
        <div class="tutor-export-header">
          <span class="tutor-export-title">EXPORT TUTORIAL</span>
          <button class="tutor-export-close" type="button" @click="close">×</button>
        </div>

        <div v-if="!hasVideo" class="tutor-export-empty">
          请先打开视频。
        </div>

        <div v-else class="tutor-export-body">
          <div class="tutor-export-field">
            <label for="export-filename" class="tutor-export-label">File name</label>
            <input
              id="export-filename"
              v-model="filename"
              class="tutor-export-input"
              :class="{ invalid: filenameInvalid }"
              type="text"
            />
            <span v-if="filenameError" class="tutor-export-error">{{ filenameError }}</span>
          </div>

          <div class="tutor-export-options">
            <label class="tutor-export-option">
              <input v-model="includeScreenshots" type="checkbox" />
              Include screenshots
            </label>
            <label class="tutor-export-option">
              <input v-model="groupByChapters" type="checkbox" />
              Group by chapters
            </label>
          </div>

          <div class="tutor-export-list-header">
            <span>Select notes</span>
            <button class="tutor-export-link" type="button" @click="toggleAll">
              {{ allSelected ? "Deselect all" : "Select all" }}
            </button>
          </div>

          <div class="tutor-export-list">
            <label
              v-for="bm in bookmarks"
              :key="bm.id"
              class="tutor-export-item"
            >
              <input v-model="selectedIds" type="checkbox" :value="bm.id" />
              <span class="tutor-export-item-time">{{ formatTime(bm.position) }}</span>
              <span class="tutor-export-item-content">{{ bm.name }}</span>
            </label>
          </div>

          <div class="tutor-export-actions">
            <button class="tutor-export-cancel" type="button" @click="close">Cancel</button>
            <button
              class="tutor-export-submit"
              type="button"
              :disabled="!canExport || exporting"
              @click="handleExport"
            >
              {{ exporting ? "..." : "Export .md" }}
            </button>
          </div>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, computed, watch } from "vue";
import { save } from "@tauri-apps/plugin-dialog";
import { open } from "@tauri-apps/plugin-shell";
import { dirname } from "@tauri-apps/api/path";
import type { BookmarkEntry } from "../api/player";
import { exportNotesToMarkdown } from "../api/tutorialExport";

const props = defineProps<{
  visible: boolean;
  hasVideo?: boolean;
  currentVideoPath?: string;
  currentVideoName?: string;
  bookmarks: BookmarkEntry[];
}>();

const emit = defineEmits<{
  close: [];
  success: [message: string];
  error: [message: string];
}>();

const filename = ref("");
const includeScreenshots = ref(true);
const groupByChapters = ref(true);
const selectedIds = ref<string[]>([]);
const exporting = ref(false);

const allSelected = computed(() =>
  props.bookmarks.length > 0 && selectedIds.value.length === props.bookmarks.length,
);

const canExport = computed(() => {
  if (!filename.value.trim()) return false;
  if (selectedIds.value.length === 0) return false;
  if (filenameInvalid.value) return false;
  return true;
});

const filenameInvalid = computed(() => {
  const name = filename.value;
  if (!name) return false;
  if (name.includes("/") || name.includes("\\")) return true;
  if (!name.endsWith(".md")) return true;
  return false;
});

const filenameError = computed(() => {
  const name = filename.value;
  if (!name) return "";
  if (name.includes("/") || name.includes("\\")) return "文件名不能包含路径分隔符";
  if (!name.endsWith(".md")) return "文件名需以 .md 结尾";
  return "";
});

function formatTime(seconds: number): string {
  if (!seconds || seconds < 0) return "00:00:00";
  const h = Math.floor(seconds / 3600);
  const m = Math.floor((seconds % 3600) / 60);
  const s = Math.floor(seconds % 60);
  return `${String(h).padStart(2, "0")}:${String(m).padStart(2, "0")}:${String(s).padStart(2, "0")}`;
}

function toggleAll() {
  if (allSelected.value) {
    selectedIds.value = [];
  } else {
    selectedIds.value = props.bookmarks.map((bm) => bm.id);
  }
}

function close() {
  emit("close");
}

watch(
  () => props.visible,
  (isVisible) => {
    if (isVisible) {
      const base = props.currentVideoName || "notes";
      filename.value = `${base.replace(/\.[^.]+$/, "")}-notes.md`;
      selectedIds.value = props.bookmarks.map((bm) => bm.id);
      includeScreenshots.value = true;
      groupByChapters.value = true;
    }
  },
);

async function handleExport() {
  if (!canExport.value) return;
  if (!props.currentVideoPath) {
    emit("error", "视频路径为空");
    return;
  }

  const selectedDir = await save({
    defaultPath: filename.value,
    filters: [{ name: "Markdown", extensions: ["md"] }],
  });

  if (!selectedDir) return;

  const outputDir = await dirname(selectedDir);

  exporting.value = true;
  try {
    const result = await exportNotesToMarkdown({
      video_path: props.currentVideoPath,
      note_ids: selectedIds.value,
      output_dir: outputDir,
      filename: filename.value,
      include_screenshots: includeScreenshots.value,
      group_by_chapters: groupByChapters.value,
    });

    emit(
      "success",
      `已导出 ${result.note_count} 条笔记，${result.screenshot_count} 张截图`,
    );

    // Offer to open the containing directory.
    await open(outputDir);
    close();
  } catch (e) {
    const message = e instanceof Error ? e.message : String(e);
    emit("error", `导出失败：${message}`);
  } finally {
    exporting.value = false;
  }
}
</script>

<style scoped>
.tutor-export-overlay {
  position: fixed;
  inset: 0;
  z-index: 50;
  background: rgba(6, 6, 10, 0.6);
  display: flex;
  align-items: center;
  justify-content: center;
}

.tutor-export-card {
  width: 420px;
  max-height: 520px;
  background: rgba(18, 18, 26, 0.96);
  backdrop-filter: blur(16px);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-lg);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.5);
}

.tutor-export-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 14px;
  border-bottom: 1px solid var(--border-subtle);
}

.tutor-export-title {
  font-family: var(--font-display);
  font-size: 13px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  color: var(--text-primary);
}

.tutor-export-close {
  background: none;
  border: none;
  color: var(--text-muted);
  font-size: 18px;
  cursor: pointer;
}

.tutor-export-close:hover {
  color: var(--text-primary);
}

.tutor-export-body {
  padding: 14px;
  display: flex;
  flex-direction: column;
  gap: 12px;
  overflow-y: auto;
}

.tutor-export-empty {
  padding: 24px;
  text-align: center;
  font-family: var(--font-body);
  font-size: 12px;
  color: var(--text-muted);
}

.tutor-export-field {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.tutor-export-label {
  font-family: var(--font-body);
  font-size: 11px;
  color: var(--text-muted);
}

.tutor-export-input {
  background: var(--bg-elevated);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-sm);
  padding: 6px 10px;
  color: var(--text-primary);
  font-family: var(--font-body);
  font-size: 13px;
  outline: none;
}

.tutor-export-input:focus {
  border-color: var(--accent-cyan);
  box-shadow: 0 0 8px rgba(0, 229, 255, 0.2);
}

.tutor-export-input.invalid {
  border-color: var(--accent-magenta);
}

.tutor-export-error {
  font-family: var(--font-mono);
  font-size: 10px;
  color: var(--accent-magenta);
}

.tutor-export-options {
  display: flex;
  flex-direction: column;
  gap: 6px;
  font-family: var(--font-body);
  font-size: 12px;
  color: var(--text-muted);
}

.tutor-export-option {
  display: flex;
  align-items: center;
  gap: 6px;
  cursor: pointer;
}

.tutor-export-list-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-family: var(--font-body);
  font-size: 11px;
  color: var(--text-muted);
}

.tutor-export-link {
  background: none;
  border: none;
  color: var(--accent-cyan);
  font-family: var(--font-mono);
  font-size: 10px;
  cursor: pointer;
  padding: 0;
}

.tutor-export-list {
  max-height: 160px;
  overflow-y: auto;
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  padding: 6px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.tutor-export-item {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  padding: 6px;
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: background 0.15s ease-out;
}

.tutor-export-item:hover {
  background: rgba(255, 255, 255, 0.04);
}

.tutor-export-item-time {
  font-family: var(--font-mono);
  font-size: 11px;
  font-variant-numeric: tabular-nums;
  color: var(--accent-cyan);
  flex-shrink: 0;
}

.tutor-export-item-content {
  font-family: var(--font-body);
  font-size: 12px;
  color: var(--text-primary);
  line-height: 1.4;
  word-break: break-word;
}

.tutor-export-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 4px;
}

.tutor-export-cancel {
  background: transparent;
  border: 1px solid var(--border-subtle);
  color: var(--text-muted);
  font-family: var(--font-display);
  font-size: 11px;
  letter-spacing: 0.06em;
  font-weight: 600;
  text-transform: uppercase;
  padding: 4px 12px;
  border-radius: var(--radius-sm);
  cursor: pointer;
}

.tutor-export-cancel:hover {
  color: var(--text-primary);
  border-color: rgba(255, 255, 255, 0.12);
}

.tutor-export-submit {
  background: var(--accent-cyan);
  border: none;
  color: var(--bg-base);
  font-family: var(--font-display);
  font-size: 11px;
  letter-spacing: 0.06em;
  font-weight: 600;
  text-transform: uppercase;
  padding: 4px 12px;
  border-radius: var(--radius-sm);
  cursor: pointer;
}

.tutor-export-submit:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
```

- [ ] **Step 2: Install Tauri dialog and shell plugins**

The modal uses `@tauri-apps/plugin-dialog` (save dialog) and `@tauri-apps/plugin-shell` (open directory).

Install frontend packages:

```bash
cd /Users/roland/Desktop/appsRoland/vPlayer/frontend
pnpm add @tauri-apps/plugin-dialog @tauri-apps/plugin-shell
```

Configure the Rust dialog plugin (already added in Task 2 via `tauri-plugin-dialog`).

Update `src-tauri/tauri.conf.json` to allow the dialog capability. Add a new capability file `src-tauri/capabilities/dialog.json`:

```json
{
  "$schema": "../node_modules/@tauri-apps/cli/config.schema.json",
  "identifier": "dialog",
  "windows": ["main"],
  "permissions": ["dialog:default"]
}
```

Then reference it in `tauri.conf.json`:

```json
"capabilities": ["default", "dialog"]
```

- [ ] **Step 3: Type-check the frontend**

```bash
cd /Users/roland/Desktop/appsRoland/vPlayer/frontend
rtk tsc --noEmit
```

Expected: no errors.

- [ ] **Step 4: Commit**

```bash
rtk git add frontend/src/components/TutorialExporterModal.vue frontend/package.json frontend/pnpm-lock.yaml
rtk git commit -m "feat: add TutorialExporterModal component"
```

---

## Task 6: Wire up the modal from BookmarkView and App.vue

**Files:**
- Modify: `frontend/src/components/BookmarkView.vue`
- Modify: `frontend/src/App.vue`

- [ ] **Step 1: Replace clipboard export in BookmarkView**

In `BookmarkView.vue`, update `defineEmits` to include the new event and replace the `handleExport` function (lines 306-325) with:

```typescript
const emit = defineEmits<{
  (e: "seek", position: number): void;
  (e: "pause"): void;
  (e: "resume"): void;
  (e: "open-exporter"): void;
}>();

// ... later in the file ...

function handleExport() {
  if (bookmarks.value.length === 0) return;
  emit("open-exporter");
}
```

(If your previous `defineEmits` is already above, just add `open-exporter` to the type declaration and replace the function body.)

- [ ] **Step 2: Add modal state in App.vue**

In `App.vue`, add these refs near the other UI state refs:

```typescript
const tutorialExporterVisible = ref(false);
```

Add the modal component in the template, near `PluginPopup`:

```vue
<TutorialExporterModal
  :visible="tutorialExporterVisible"
  :has-video="!!currentMediaPath"
  :current-video-path="currentMediaPath"
  :current-video-name="currentTitle"
  :bookmarks="currentBookmarks"
  @close="tutorialExporterVisible = false"
  @success="showToast"
  @error="showToast"
/>
```

Add `currentBookmarks` as a computed or ref. The simplest approach is to store the bookmark list in App.vue. However, `BookmarkView` currently owns its own list.

Better: expose the bookmark list from `BookmarkView` via `defineExpose`, and read it when the export button is clicked.

In `BookmarkView.vue`, add to `defineExpose`:

```typescript
defineExpose({
  focusInput,
  bookmarks: computed(() => bookmarks.value),
});
```

In `App.vue`, add a ref for the bookmark view:

```typescript
const bookmarkViewRef = ref<ComponentPublicInstance & {
  focusBookmarkInput?: () => void;
  bookmarks?: BookmarkEntry[];
} | null>(null);
```

Add a function to open the exporter:

```typescript
function openTutorialExporter() {
  tutorialExporterVisible.value = true;
}
```

Pass `currentBookmarks` to the modal using the exposed value:

```vue
<TutorialExporterModal
  :visible="tutorialExporterVisible"
  :has-video="!!currentMediaPath"
  :current-video-path="currentMediaPath"
  :current-video-name="currentTitle"
  :bookmarks="bookmarkViewRef?.bookmarks ?? []"
  @close="tutorialExporterVisible = false"
  @success="showToast"
  @error="showToast"
/>
```

Bind the ref to `PluginPopup` so `bookmarkViewRef` points to the `BookmarkView` instance:

```vue
<BookmarkView
  ref="bookmarkViewRef"
  ...
/>
```

Wait — `BookmarkView` is rendered inside `PluginPopup`, not directly in `App.vue`. So `App.vue` cannot easily get a ref to it.

Alternative approach: move the export trigger up. Instead of `BookmarkView` emitting `open-exporter`, have `PluginPopup` emit it.

In `PluginPopup.vue`, bind to `BookmarkView`'s export event:

```vue
<BookmarkView
  ...
  @open-exporter="emit('open-exporter')"
/>
```

In `App.vue`, listen on `PluginPopup`:

```vue
<PluginPopup
  ...
  @open-exporter="openTutorialExporter"
/>
```

And for the bookmarks list, expose it from `BookmarkView` and then from `PluginPopup`:

In `BookmarkView.vue`:

```typescript
defineExpose({
  focusInput,
  getBookmarks: () => bookmarks.value,
});
```

In `PluginPopup.vue`:

```typescript
const bookmarkViewRef = ref<InstanceType<typeof BookmarkView> | null>(null);

function getBookmarks() {
  return bookmarkViewRef.value?.getBookmarks?.() ?? [];
}

defineExpose({
  focusBookmarkInput: () => bookmarkViewRef.value?.focusInput?.(),
  getBookmarks,
});
```

In `App.vue`:

```typescript
const pluginPopupRef = ref<ComponentPublicInstance & {
  focusBookmarkInput?: () => void;
  getBookmarks?: () => BookmarkEntry[];
} | null>(null);

const currentBookmarks = computed(() => pluginPopupRef.value?.getBookmarks?.() ?? []);
```

This is a bit indirect but keeps the data flow clean.

- [ ] **Step 3: Import dependencies**

In `App.vue`, add `BookmarkEntry` import and `TutorialExporterModal` import:

```typescript
import { type BookmarkEntry } from "./api/player";
import TutorialExporterModal from "./components/TutorialExporterModal.vue";
```

In `PluginPopup.vue`, add the emit:

```typescript
const emit = defineEmits<{
  // ... existing emits
  "open-exporter": [];
}>();
```

- [ ] **Step 4: Type-check and commit**

```bash
cd /Users/roland/Desktop/appsRoland/vPlayer/frontend
rtk tsc --noEmit
```

```bash
rtk git add frontend/src/components/BookmarkView.vue frontend/src/components/PluginPopup.vue frontend/src/App.vue
rtk git commit -m "feat: wire TutorialExporterModal into app"
```

---

## Task 7: Add backend unit tests

**Files:**
- Create: `src-tauri/src/ipc/tutorial_export_test.rs`

- [ ] **Step 1: Create tests for pure functions**

Create `src-tauri/src/ipc/tutorial_export_test.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::super::tutorial_export::format_seconds;

    #[test]
    fn test_format_seconds() {
        assert_eq!(format_seconds(0.0), "00:00:00");
        assert_eq!(format_seconds(61.5), "00:01:01");
        assert_eq!(format_seconds(3661.0), "01:01:01");
    }
}
```

In `src-tauri/src/ipc/tutorial_export.rs`, make `format_seconds` public so tests can use it:

```rust
pub fn format_seconds(secs: f64) -> String {
```

Add the test module to `src-tauri/src/ipc/mod.rs`:

```rust
#[cfg(test)]
mod tutorial_export_test;
```

- [ ] **Step 2: Run tests**

```bash
cd /Users/roland/Desktop/appsRoland/vPlayer/src-tauri
rtk cargo test tutorial_export
```

Expected: tests pass.

- [ ] **Step 3: Commit**

```bash
rtk git add src-tauri/src/ipc/tutorial_export_test.rs src-tauri/src/ipc/tutorial_export.rs src-tauri/src/ipc/mod.rs
rtk git commit -m "test: add tutorial export format helper tests"
```

---

## Task 8: Manual integration verification

- [ ] **Step 1: Start the app in dev mode**

```bash
cd /Users/roland/Desktop/appsRoland/vPlayer/frontend
pnpm tauri:dev
```

- [ ] **Step 2: Open a video and add at least 3 notes**

Use the Notes panel. Include one note with a Markdown code block, e.g.:

```
```rust
let x = 1;
```
```

- [ ] **Step 3: Click EXPORT**

Verify the modal opens with the correct video name, default filename, and all notes pre-selected.

- [ ] **Step 4: Configure export**

- Keep "Include screenshots" checked
- Keep "Group by chapters" checked (if the video has chapters)
- Uncheck one note

- [ ] **Step 5: Export and inspect**

Choose a directory. Verify:

- A `.md` file is created
- An `assets/` folder is created with PNG files
- Markdown structure is correct
- Code block is preserved
- Time links are `vplayer://` format

- [ ] **Step 6: Test error paths**

- Empty filename → Export disabled
- Filename without `.md` → error shown
- Cancel dialog → modal stays open
- Output dir without write permission → error Toast

- [ ] **Step 7: Commit any fixes**

If you made changes during verification, commit them with descriptive messages.

---

## Self-Review Checklist

| Spec Section | Covered By |
|--------------|------------|
| Goal | Task 2 |
| Scope in/out | Architecture decisions in Task 2 |
| User flow | Tasks 5-6 |
| UI design | Task 5 |
| Export format | Task 2 |
| Time links | Task 2 `render_note` |
| Chapter grouping | Task 2 `render_markdown` |
| Backend command | Tasks 2-3 |
| Frontend modal | Tasks 4-6 |
| Error handling | Task 2 validation + Task 5 UI |
| Tests | Task 7 |

**Placeholder scan:** No TBD/TODO/fill-in-details found.

**Type consistency:** `ExportNotesRequest`/`ExportNotesResult` match between Rust (Task 2) and TypeScript (Task 4).

**Open issues to resolve during execution:**
1. `@tauri-apps/plugin-dialog` and `@tauri-apps/plugin-shell` must be confirmed installed; add if missing.
2. The `save` dialog returns a file path; the plan derives `output_dir` by stripping the filename. Verify on all target platforms.
3. `capture_frame_to_path` uses ffmpeg. Ensure ffmpeg is available in the test environment.

---

*Plan generated by /superpowers:writing-plans based on `docs/superpowers/specs/2026-06-16-tutorial-exporter-design.md`.*
