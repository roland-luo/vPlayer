# Architecture

**Analysis Date:** 2026-06-16

## Pattern Overview

**Overall:** Tauri v2 desktop application with a Vue 3 frontend and Rust backend. The architecture follows a hybrid web-native pattern where the UI layer is a webview (Vue 3 + Vite) and the core player logic, file I/O, and plugin system run in a Rust backend process.

**Key Characteristics:**
- Frontend is a single-page Vue 3 application with no router (all UI is component-based overlays and panels)
- Backend uses Tauri v2's IPC (invoke commands + event emitters) for frontend-backend communication
- State is centralized in a Rust-managed `AppState` struct with `std::sync::Mutex` locks
- Plugin system supports both builtin (compiled-in) and external (dynamic library, Phase 2) plugins
- Video rendering currently uses HTML5 `<video>` element via Tauri asset protocol; libmpv integration is scaffolded but not yet wired
- No database; all persistence is JSON file-based (settings, bookmarks)

## Layers

**Frontend (Vue 3 SPA):**
- Purpose: UI rendering, user interaction, media controls, plugin popups, playlist management
- Location: `/Users/roland/Desktop/appsRoland/vPlayer/frontend/src/`
- Contains: Vue SFC components, composables, API client, global CSS
- Depends on: Tauri API (`@tauri-apps/api`), browser APIs (`HTMLVideoElement`, `AudioContext`, `Clipboard`)
- Used by: End user via Tauri webview window

**Backend (Rust / Tauri):**
- Purpose: File I/O, state management, plugin lifecycle, IPC command handling, settings persistence
- Location: `/Users/roland/Desktop/appsRoland/vPlayer/src-tauri/src/`
- Contains: IPC commands, state management, plugin system, error handling, config/settings
- Depends on: Tauri v2, `serde`, `rfd` (file dialogs), `libloading` (future plugin loading)
- Used by: Frontend via Tauri invoke commands and event listeners

**Plugin System:**
- Purpose: Extensible feature modules (bookmarks, chapters, subtitles, equalizer, media info, speed control, screenshots)
- Location: `/Users/roland/Desktop/appsRoland/vPlayer/src-tauri/src/plugin/`
- Contains: Plugin trait definitions, builtin implementations, registry, bus, loader, sandbox
- Depends on: Backend state, Tauri event system
- Used by: Backend commands and frontend plugin UI panels

**Render Layer (Scaffolded):**
- Purpose: Future OpenGL/libmpv video rendering
- Location: `/Users/roland/Desktop/appsRoland/vPlayer/src-tauri/src/render/`
- Contains: `gl_context.rs`, `texture.rs`, `frame.rs` (all scaffolds)
- Depends on: Not yet integrated
- Used by: Planned for Phase 2+ libmpv video output

## Data Flow

**Media Playback Flow:**

1. User clicks "Open File" in `TopHud` or presses play in `PlayerView`
2. `App.vue` calls `pickAndPlayFile()` in `/Users/roland/Desktop/appsRoland/vPlayer/frontend/src/api/player.ts`
3. Frontend invokes `pick_and_play_file` Tauri command (`/Users/roland/Desktop/appsRoland/vPlayer/src-tauri/src/ipc/commands.rs`)
4. Backend opens `rfd::FileDialog`, adds path to playlist, updates `AppState.player` mutex
5. Backend emits `player:state_change` and `player:progress` events via `/Users/roland/Desktop/appsRoland/vPlayer/src-tauri/src/ipc/events.rs`
6. Frontend `App.vue` listens for these events and updates reactive `playerState` ref
7. `PlayerView.vue` receives `sourcePath` prop, converts to asset URL via `convertFileSrc()`, sets `<video src>`
8. `PlayerView.vue` emits `progress`, `loaded-metadata`, `ended` events back to `App.vue`
9. `App.vue` applies UI compensation (requestAnimationFrame-based position interpolation) when playing

**Settings Persistence Flow:**

1. `App.vue` calls `loadPlayerSettings()` on mount via `api/player.ts`
2. Backend `ipc/config.rs` reads `settings.json` from app data directory
3. Settings applied to `AppState` mutexes (volume, playlist, playback speed)
4. On changes (volume, speed, playlist), `App.vue` calls `savePlayerSettings()`
5. Backend serializes and writes `settings.json` via `config.rs`

**Plugin Event Flow:**

1. Backend command (e.g., `capture_screenshot`) acquires `plugin_bus` mutex
2. Creates `PluginEvent` and calls `bus.emit()` in `/Users/roland/Desktop/appsRoland/vPlayer/src-tauri/src/plugin/bus.rs`
3. `PluginBus` iterates enabled plugins, wraps each `on_event` in `catch_unwind`
4. Failed plugins are marked `Crashed` and errors are reported to frontend via `plugin:error` event
5. Frontend `App.vue` listens for `plugin:state_changed`, `plugin:error`, `plugin:installed` events

## Key Abstractions

**AppState (Centralized Mutable State):**
- Purpose: Single source of truth for all runtime state
- Location: `/Users/roland/Desktop/appsRoland/vPlayer/src-tauri/src/ipc/state.rs`
- Pattern: `std::sync::Mutex` fields inside a Tauri-managed state struct
- Fields: `player` (PlayerState), `playlist` (PlaylistState), `startup_fatal` (Option<StartupFatalState>), `plugin_bus` (PluginBus), `plugin_registry` (PluginRegistry), `settings` (PlayerSettings)

**Plugin Trait (Extensibility Interface):**
- Purpose: Contract for all plugin implementations
- Location: `/Users/roland/Desktop/appsRoland/vPlayer/src-tauri/src/plugin/mod.rs`
- Pattern: Rust trait with `Send + Sync` bounds
- Methods: `name()`, `on_load(ctx)`, `on_unload()`, `on_event(event)`

**IPC API Client (Frontend-to-Backend Bridge):**
- Purpose: Type-safe wrapper over Tauri invoke commands
- Location: `/Users/roland/Desktop/appsRoland/vPlayer/frontend/src/api/player.ts`
- Pattern: Async functions wrapping `invoke()` with typed return values
- Exports: `playFile`, `pause`, `resume`, `seek`, `setVolume`, `listBookmarks`, `searchSubtitles`, etc.

**Vue Composables (Reusable Frontend Logic):**
- Purpose: Encapsulate cross-cutting frontend concerns
- Location: `/Users/roland/Desktop/appsRoland/vPlayer/frontend/src/composables/`
- Files: `useHotkeys.ts` (keyboard shortcut management), `useAudioEqualizer.ts` (Web Audio API equalizer)
- Pattern: Vue 3 Composition API functions with lifecycle hooks

## Entry Points

**Frontend Entry Point:**
- Location: `/Users/roland/Desktop/appsRoland/vPlayer/frontend/src/main.ts`
- Triggers: Tauri webview loads `index.html` which executes this script
- Responsibilities: Creates Vue app, mounts `App.vue`, applies global styles

**Backend Entry Point:**
- Location: `/Users/roland/Desktop/appsRoland/vPlayer/src-tauri/src/main.rs`
- Triggers: Tauri runtime starts the application process
- Responsibilities: Builds Tauri app, registers all IPC commands, sets up `AppState`, runs startup probe, initializes plugin system, opens devtools in debug mode

**Build Entry Points:**
- Frontend build: `/Users/roland/Desktop/appsRoland/vPlayer/frontend/vite.config.ts` (Vite + Vue plugin, port 5175)
- Backend build: `/Users/roland/Desktop/appsRoland/vPlayer/src-tauri/Cargo.toml` (Tauri v2, Rust edition 2021)
- Tauri config: `/Users/roland/Desktop/appsRoland/vPlayer/src-tauri/tauri.conf.json` (bundles frontend dist, dev URL, window config, asset protocol scope)

## Error Handling

**Strategy:** Layered error propagation with frontend-friendly payloads

**Patterns:**
- Backend commands return `Result<T, String>` — errors are stringified and sent to frontend as promise rejections
- Fatal startup errors are stored in `AppState.startup_fatal` and emitted as `app:fatal_error` event
- Video errors are emitted as `video:error` events with `{ code, message }` payloads
- Plugin panics are caught via `catch_unwind` and converted to `plugin:error` events
- Debug error injection is supported via env vars (`VPLAYER_FORCE_STARTUP_FATAL`, `VPLAYER_FORCE_RENDER_FATAL`)

## Cross-Cutting Concerns

**Logging:** Console-based (`eprintln!`, `console.debug`) with structured prefixes like `[bookmark]`, `[plugin]`, `[config]`. No structured logging framework is integrated yet (`/Users/roland/Desktop/appsRoland/vPlayer/src-tauri/src/utils/logging.rs` is a scaffold).

**Validation:** Minimal. Volume is clamped to 0-100 on both frontend and backend. Seek position is clamped to duration. No input validation library is used.

**Authentication:** Not applicable — this is a local desktop application with no user accounts or remote auth.

**Security:** Tauri v2 CSP is set to `null` (disabled). Asset protocol is enabled with scope restricted to home directory subfolders (`$HOME`, `$DESKTOP`, `$DOCUMENT`, `$DOWNLOAD`, `$VIDEO`).

---

*Architecture analysis: 2026-06-16*
