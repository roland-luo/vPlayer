# External Integrations

**Analysis Date:** 2026-06-16

## APIs & External Services

**Media Analysis:**
- `ffprobe` (system binary) — Video metadata extraction (container, duration, codecs, resolution, FPS, audio channels)
  - Used by: `src-tauri/src/ipc/mediainfo.rs`
  - Command: `ffprobe -v quiet -print_format json -show_format -show_streams <path>`
- `ffmpeg` (system binary) — Screenshot frame extraction
  - Used by: `src-tauri/src/ipc/commands.rs` (capture_screenshot command)
  - Command: `ffmpeg -ss <position> -i <path> -vframes 1 -q:v 2 -y <output>`

**File System:**
- Tauri Asset Protocol — Serves local files to frontend via `convertFileSrc`
  - Scope: home, desktop, documents, downloads, video directories
  - Used by: `frontend/src/components/PlayerView.vue` for video src

**Native Dialogs:**
- `rfd` (Rust crate) — Native file picker dialogs
  - Video picker filter: mp4, mkv, mov, avi, webm
  - Subtitle picker filter: vtt, srt, ass, ssa
  - Save dialog for screenshots (PNG)

## Data Storage

**Databases:**
- None — All persistence is file-based JSON

**File Storage:**
- Local filesystem — Application data directory (Tauri `app_data_dir`)
  - Settings: `<app_data>/settings.json` (`src-tauri/src/config.rs`)
  - Bookmarks: `<app_data>/bookmarks.json` (`src-tauri/src/ipc/bookmark.rs`)
  - Plugins: `<app_config>/plugins/` (`src-tauri/src/plugin/mod.rs`)
  - Plugin logs: `<app_config>/plugin-logs/` (`src-tauri/src/plugin/mod.rs`)
  - Diagnostic reports: `<app_log>/fatal-report-<timestamp>.txt` (`src-tauri/src/ipc/commands.rs`)

**Caching:**
- None detected

## Authentication & Identity

**Auth Provider:**
- Not applicable — Desktop application, no user accounts or authentication

## Monitoring & Observability

**Error Tracking:**
- Custom fatal error overlay — Frontend displays startup/render errors with diagnostic actions
- Plugin error tracking — Error counts and last error messages per plugin
- Console debug logging — `console.debug()` for non-critical failures in frontend
- stderr logging in Rust backend — `eprintln!` for backend events

**Logs:**
- App log directory: Tauri `app_log_dir()`
- Log directory can be opened via native file manager (`open_log_directory` command)
- Fatal diagnostic reports saved to log directory with timestamps

## CI/CD & Deployment

**Hosting:**
- Desktop application distributed via Tauri bundler
- No web hosting or server deployment

**CI Pipeline:**
- None detected — 35 PR verification scripts (`scripts/verify-pr*.sh`) for local manual testing

## Environment Configuration

**Required env vars:**
- None for normal operation

**Debug env vars:**
- `VPLAYER_FORCE_STARTUP_FATAL=1` — Triggers simulated startup fatal error
- `VPLAYER_FORCE_RENDER_FATAL=1` — Triggers simulated render failure

**Secrets location:**
- Not applicable — No external API keys or secrets required

## Webhooks & Callbacks

**Incoming:**
- None — Desktop application, no web server

**Outgoing:**
- None — No external web service callbacks

## Tauri IPC Event System

**Frontend-to-Backend (invoke commands):**
- 30+ Tauri commands defined in `src-tauri/src/main.rs`
- Key commands: `play_file`, `pause`, `resume`, `seek`, `set_volume`, `pick_and_play_file`, `search_subtitles`, `download_subtitle`, `get_media_info`, `list_bookmarks`, `add_bookmark`, `capture_screenshot`, `toggle_plugin`, etc.

**Backend-to-Frontend (events):**
- `player:state_change` — Playback state updates
- `player:progress` — Position/duration updates
- `video:error` — Video element errors
- `app:fatal_error` — Startup fatal errors
- `plugin:state_changed` — Plugin enable/disable changes
- `plugin:error` — Plugin runtime errors
- `plugin:installed` — New plugin installation

## Plugin System Integration

**Builtin Plugins (7):**
- `screenshot` — Frame capture tracking
- `subtitle-download` — Subtitle search/download tracking
- `media-info` — Media info request tracking
- `playback-speed` — Speed change tracking
- `bookmark` — Bookmark CRUD tracking
- `chapter` — Chapter list tracking
- `equalizer` — Preset change tracking

**External Plugin Loading:**
- Phase 1: Manifest validation only (`manifest.json` parsing)
- Phase 2+: Dynamic library loading via `libloading` (`.dylib`, `.so`, `.dll`)
- Plugin directory: `<app_config>/plugins/<name>/`

---

*Integration audit: 2026-06-16*
