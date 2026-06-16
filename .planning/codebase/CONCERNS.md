# Codebase Concerns

**Analysis Date:** 2026-06-16

## Tech Debt

### libmpv Integration is a Stub

- **Issue:** `mpv::core::startup_probe()` is a no-op that only checks an environment variable. The actual libmpv runtime (`mpv_create`, `mpv_initialize`, `mpv_render_context_create`) is not integrated. The player currently uses an HTML5 `<video>` element for playback, not libmpv.
- **Files:**
  - `/Users/roland/Desktop/appsRoland/vPlayer/src-tauri/src/mpv/core.rs` (28 lines, all stub)
  - `/Users/roland/Desktop/appsRoland/vPlayer/src-tauri/src/mpv/renderer.rs` (57 lines, simulates texture failures for debug only)
  - `/Users/roland/Desktop/appsRoland/vPlayer/src-tauri/src/mpv/event.rs` (1 line, empty)
  - `/Users/roland/Desktop/appsRoland/vPlayer/src-tauri/src/mpv/options.rs` (1 line, empty)
- **Impact:** The player cannot handle formats that HTML5 video does not support (e.g., MKV with advanced codecs, HEVC, AV1 in some containers). The entire "modern desktop video player" value proposition depends on this integration.
- **Fix approach:** Implement `mpv_create()` → `mpv_set_option_string()` → `mpv_initialize()` in `core.rs`. Wire `mpv_render_context_create()` with OpenGL in `renderer.rs`. Replace the HTML5 `<video>` element in `PlayerView.vue` with a custom GL surface rendered by libmpv.

### OpenGL Render Pipeline is Empty Scaffolding

- **Issue:** `render/gl_context.rs`, `render/frame.rs`, `render/texture.rs` are all single-line comment stubs. No actual OpenGL context management, frame upload, or texture handling exists.
- **Files:**
  - `/Users/roland/Desktop/appsRoland/vPlayer/src-tauri/src/render/gl_context.rs`
  - `/Users/roland/Desktop/appsRoland/vPlayer/src-tauri/src/render/frame.rs`
  - `/Users/roland/Desktop/appsRoland/vPlayer/src-tauri/src/render/texture.rs`
- **Impact:** Cannot render video frames from libmpv. The render module is completely non-functional.
- **Fix approach:** Implement GL context creation via `raw-window-handle` + `gl` crate. Implement `mpv_render_context_render()` integration. Frame upload logic depends on libmpv's `MPV_RENDER_PARAM_OPENGL_FBO` API.

### Plugin System: Dynamic Loading Not Implemented

- **Issue:** The plugin loader (`loader.rs`) only validates `manifest.json`. It does not actually load `.dylib`/`.so`/`.dll` files via `libloading`. The `libloading` crate is a dependency but unused for dynamic loading.
- **Files:**
  - `/Users/roland/Desktop/appsRoland/vPlayer/src-tauri/src/plugin/loader.rs` (lines 31-34: "Phase 2+: find and load the dynamic library")
  - `/Users/roland/Desktop/appsRoland/vPlayer/src-tauri/src/plugin/mod.rs` (lines 307-344: external plugins are acknowledged but not loaded)
- **Impact:** No third-party plugin ecosystem. All "plugins" are hardcoded builtins.
- **Fix approach:** Implement `libloading::Library::new()` in `loader.rs`. Create a C FFI bridge for the `Plugin` trait. Handle symbol resolution for `plugin_init()` exports.

### Plugin Permissions are Declarative Only

- **Issue:** `sandbox.rs` checks if a permission is in the manifest's declared list. There is no runtime enforcement — a plugin can still call `std::fs::write()` even without `FileWrite` permission.
- **Files:** `/Users/roland/Desktop/appsRoland/vPlayer/src-tauri/src/plugin/sandbox.rs`
- **Impact:** The permission system is security theater. A malicious plugin has full access to the filesystem and network.
- **Fix approach:** Implement syscall interception or run plugins in a separate WASM sandbox / subprocess. At minimum, use `seccomp` (Linux) or `seatbelt` (macOS) for OS-level sandboxing.

### Backend State is a Mock

- **Issue:** `ipc/commands.rs` functions like `play_file`, `pause`, `seek`, `set_volume` only mutate an in-memory `Mutex<PlayerState>` and emit events. They do not actually control any media playback engine. The real playback control happens entirely in the frontend's HTML5 `<video>` element.
- **Files:** `/Users/roland/Desktop/appsRoland/vPlayer/src-tauri/src/ipc/commands.rs` (lines 53-233)
- **Impact:** Backend and frontend playback state can diverge. The backend "state" is a shadow copy of the frontend's actual state. Commands like `seek` emit events but do not actually seek the video.
- **Fix approach:** Once libmpv is integrated, wire all commands to actual `mpv_command_string()` / `mpv_set_property()` calls. Remove the mock state mutations.

### Screenshot Uses ffmpeg Instead of Frame Capture

- **Issue:** `capture_screenshot` spawns `ffmpeg` as a subprocess to extract a frame. This is slow, requires ffmpeg to be installed, and does not capture the actual rendered frame (e.g., with subtitles, overlays, or equalizer effects applied).
- **Files:** `/Users/roland/Desktop/appsRoland/vPlayer/src-tauri/src/ipc/commands.rs` (lines 426-523)
- **Impact:** Screenshot feature is fragile (depends on external ffmpeg), slow (subprocess overhead), and inaccurate (does not reflect actual rendered output).
- **Fix approach:** Once libmpv is integrated, use `mpv_command_async()` with `screenshot-to-file`. Or capture from the OpenGL framebuffer directly.

### Media Info and Chapter Extraction Depend on External ffprobe

- **Issue:** `get_media_info` and `list_chapters` spawn `ffprobe` subprocesses. The application assumes `ffprobe` is installed and on PATH. No validation or bundled dependency.
- **Files:**
  - `/Users/roland/Desktop/appsRoland/vPlayer/src-tauri/src/ipc/mediainfo.rs` (lines 84-93)
  - `/Users/roland/Desktop/appsRoland/vPlayer/src-tauri/src/ipc/chapter.rs` (lines 39-47)
- **Impact:** Features silently fail if ffmpeg/ffprobe is not installed. No user-friendly error message about missing dependencies.
- **Fix approach:** Bundle ffmpeg/ffprobe with the application (Tauri sidecar). Or use libmpv's property API (`mpv_get_property_string` for `track-list`, `chapter-list`) once integrated.

### Hardcoded File Filter Extensions

- **Issue:** Video file picker only allows `mp4`, `mkv`, `mov`, `avi`, `webm`. Many other formats are excluded (e.g., `.m4v`, `.flv`, `.wmv`, `.ts`, `.m2ts`, `.ogv`).
- **Files:** `/Users/roland/Desktop/appsRoland/vPlayer/src-tauri/src/ipc/commands.rs` (line 119)
- **Impact:** Users cannot open valid video files with less common extensions.
- **Fix approach:** Expand the filter list or remove extension filtering entirely and validate by attempting to load the file.

### Hardcoded Chinese Text in UI

- **Issue:** Multiple UI components contain hardcoded Chinese strings mixed with English. This makes internationalization difficult and creates an inconsistent user experience.
- **Files:**
  - `/Users/roland/Desktop/appsRoland/vPlayer/frontend/src/components/BookmarkView.vue` (lines 31, 53, 93, 96, 97, 215, 309, 321, 322)
  - `/Users/roland/Desktop/appsRoland/vPlayer/frontend/src/components/SubtitleSearch.vue` (lines 8, 15, 23, 40, 69, 82)
  - `/Users/roland/Desktop/appsRoland/vPlayer/frontend/src/components/ChapterView.vue` (lines 3, 11, 59)
  - `/Users/roland/Desktop/appsRoland/vPlayer/frontend/src/components/MediaInfoView.vue` (lines 4, 10, 14, 22, 29)
- **Impact:** Inconsistent UX. Non-Chinese users see mixed-language UI.
- **Fix approach:** Extract all UI strings to a locale file. Use Vue i18n or a simple translation map.

### Frontend Playback State Compensated by requestAnimationFrame

- **Issue:** `App.vue` uses a `requestAnimationFrame` loop (`startUiCompensation`) to interpolate playback position between `player:progress` events. This is a workaround because the backend does not drive real playback.
- **Files:** `/Users/roland/Desktop/appsRoland/vPlayer/frontend/src/App.vue` (lines 395-419)
- **Impact:** Position drift between frontend interpolation and actual video time. Unnecessary complexity.
- **Fix approach:** Remove the compensation loop once libmpv drives actual playback and emits accurate progress events.

### Settings Persistence Has a TODO

- **Issue:** `window_size` is always persisted as `null` with a TODO comment: "TODO: read from Tauri window API".
- **Files:** `/Users/roland/Desktop/appsRoland/vPlayer/frontend/src/App.vue` (line 278)
- **Impact:** Window size is never restored across sessions.
- **Fix approach:** Use `getCurrentWindow().innerSize()` before saving settings. Apply on startup via `appWindow.setSize()`.

### Equalizer Creates a New AudioContext on Every Popup Open

- **Issue:** `useAudioEqualizer.ts` uses module-level singletons (`let context`, `let source`) but `connect()` creates a new `AudioContext` every time if `_isConnected` is false. If the popup is closed and reopened, the old context may be garbage collected but the video element's audio graph is disrupted.
- **Files:** `/Users/roland/Desktop/appsRoland/vPlayer/frontend/src/composables/useAudioEqualizer.ts` (lines 55-97)
- **Impact:** Audio may cut out or glitch when the equalizer popup is toggled. The `disconnect()` function is never called in the component lifecycle.
- **Fix approach:** Call `disconnect()` in `onUnmounted` of `EqualizerView.vue`. Ensure the audio graph is only created once per video session, not per popup open.

### Control Bar Fullscreen Duplicates App.vue Logic

- **Issue:** `ControlBar.vue` has its own `toggleFullscreen` implementation that is nearly identical to `App.vue`'s `toggleFullscreen`. Both try Tauri API first, then fallback to DOM fullscreen API.
- **Files:**
  - `/Users/roland/Desktop/appsRoland/vPlayer/frontend/src/components/ControlBar.vue` (lines 166-194)
  - `/Users/roland/Desktop/appsRoland/vPlayer/frontend/src/App.vue` (lines 341-369)
- **Impact:** Code duplication. Changes to fullscreen behavior must be made in two places.
- **Fix approach:** Extract fullscreen logic into a shared composable (e.g., `useFullscreen.ts`).

## Known Bugs

### Plugin Popup Error State is Static

- **Symptoms:** Plugin popup shows "Failed to load plugin data" if `getPluginDetail` fails, but the error message is hardcoded and not actionable.
- **Files:** `/Users/roland/Desktop/appsRoland/vPlayer/frontend/src/components/PluginPopup.vue` (line 152)
- **Trigger:** Network error or plugin not found in registry.
- **Workaround:** Close and reopen the popup.

### Playlist Remove Logic Has Edge Cases

- **Symptoms:** Removing the currently playing item when it is the last item in the playlist may leave `current_index` pointing to an out-of-bounds index.
- **Files:** `/Users/roland/Desktop/appsRoland/vPlayer/frontend/src/App.vue` (lines 640-664)
- **Trigger:** Remove the last item in a playlist of length 1 while it is playing.
- **Workaround:** None known. The code at line 651 checks `index >= playlistState.value.items.length` after splice, but the condition `index === playlistState.value.current_index` on line 645 may not handle the empty playlist case correctly because `current_index` is set to `null` on line 648 but then the `else if` on line 651 may still execute if the outer `if` block does not cover all cases.

### Mute Toggle Does Not Restore Previous Volume

- **Symptoms:** Clicking the mute button sets volume to 0. Clicking again sets volume to the current `props.volume` (which is 0 after muting), so it stays muted.
- **Files:** `/Users/roland/Desktop/appsRoland/vPlayer/frontend/src/components/ControlBar.vue` (lines 147-150)
- **Trigger:** Click mute button twice.
- **Workaround:** Use the volume slider to manually restore volume.

### Subtitle Track Toggle Logic is Fragile

- **Symptoms:** Toggling a subtitle track off then on may not restore it correctly because `setTextTrackEnabled` in `PlayerView.vue` clones the `<track>` element when enabling, which may reset the `src` attribute or cause the track to reload.
- **Files:** `/Users/roland/Desktop/appsRoland/vPlayer/frontend/src/components/PlayerView.vue` (lines 146-170)
- **Trigger:** Toggle a subtitle track off, then toggle it back on.
- **Workaround:** Reload the external subtitle file.

## Security Considerations

### CSP is Null

- **Risk:** Content Security Policy is disabled (`"csp": null` in `tauri.conf.json`). The frontend can load arbitrary scripts, styles, and connect to any origin.
- **Files:** `/Users/roland/Desktop/appsRoland/vPlayer/src-tauri/tauri.conf.json` (line 26)
- **Current mitigation:** None.
- **Recommendations:** Set a strict CSP. Example: `"default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' asset: https:; connect-src 'self'"`.

### Asset Protocol Scope is Overly Broad

- **Risk:** The asset protocol allows reading any file under `$HOME`, `$DESKTOP`, `$DOCUMENT`, `$DOWNLOAD`, `$VIDEO`. A malicious frontend script could exfiltrate arbitrary user files.
- **Files:** `/Users/roland/Desktop/appsRoland/vPlayer/src-tauri/tauri.conf.json` (lines 28-36)
- **Current mitigation:** Tauri's asset protocol is read-only, but the scope is still very broad.
- **Recommendations:** Restrict to only the video files the user has explicitly opened. Use a narrower scope like the specific video file path and its parent directory.

### Debug Error Buttons in Production UI

- **Risk:** `TopHud.vue` exposes "Err" and "Fatal" debug buttons that emit debug error events. These are visible in all builds, not just debug builds.
- **Files:** `/Users/roland/Desktop/appsRoland/vPlayer/frontend/src/components/TopHud.vue` (lines 33-34)
- **Current mitigation:** The buttons only work in Tauri runtime, but they are still visible and clickable.
- **Recommendations:** Wrap debug UI elements in `#[cfg(debug_assertions)]` or a build-time flag.

### No Input Sanitization on Bookmark Names

- **Risk:** Bookmark names are user-provided strings stored in JSON and rendered in the UI. No XSS sanitization is applied before rendering in `BookmarkView.vue`.
- **Files:** `/Users/roland/Desktop/appsRoland/vPlayer/frontend/src/components/BookmarkView.vue` (line 112: `{{ bm.name }}`)
- **Current mitigation:** Vue's template interpolation auto-escapes HTML, so basic XSS is prevented. However, the exported markdown in `handleExport` does not sanitize, and the bookmark name is used in `aria-label` attributes without escaping.
- **Recommendations:** Sanitize bookmark names before storage. Escape special characters in exported markdown.

## Performance Bottlenecks

### requestAnimationFrame Compensation Loop

- **Problem:** A continuous `requestAnimationFrame` loop runs while playing to interpolate playback position.
- **Files:** `/Users/roland/Desktop/appsRoland/vPlayer/frontend/src/App.vue` (lines 395-419)
- **Cause:** Backend does not drive actual playback; frontend compensates for lack of real progress events.
- **Improvement path:** Remove the loop once libmpv integration provides accurate `player:progress` events driven by actual playback position.

### Plugin Bus Uses Vec Linear Search

- **Problem:** `PluginBus` stores plugins in a `Vec` and uses `O(n)` linear search for every operation (`get_mut`, `find`, `emit`).
- **Files:** `/Users/roland/Desktop/appsRoland/vPlayer/src-tauri/src/plugin/bus.rs` (lines 13, 37-39, 62-64)
- **Cause:** Simple data structure chosen for prototype. With 7 builtins this is negligible, but with many external plugins it will matter.
- **Improvement path:** Replace `Vec<PluginInstance>` with `HashMap<String, PluginInstance>` for O(1) lookups.

### Plugin Registry Also Uses Vec Linear Search

- **Problem:** Same O(n) issue in `PluginRegistry`.
- **Files:** `/Users/roland/Desktop/appsRoland/vPlayer/src-tauri/src/plugin/registry.rs` (lines 28, 32-63)
- **Improvement path:** Replace with `HashMap<String, PluginInfo>`.

### Bookmark JSON File Grows Unbounded

- **Problem:** All bookmarks for all videos are stored in a single `bookmarks.json` file. There is no cleanup or archiving.
- **Files:** `/Users/roland/Desktop/appsRoland/vPlayer/src-tauri/src/ipc/bookmark.rs` (lines 19-23)
- **Cause:** Simple flat-file storage. No pagination or per-video files.
- **Improvement path:** Store bookmarks per video (e.g., `<video_hash>.bookmarks.json`) or use a lightweight embedded database like SQLite.

### Settings JSON Loaded on Every Persist Call

- **Problem:** `persistSettings()` in `App.vue` serializes the entire settings object and invokes `savePlayerSettings` on every volume change, speed change, and playlist mutation.
- **Files:** `/Users/roland/Desktop/appsRoland/vPlayer/frontend/src/App.vue` (lines 272-289)
- **Cause:** No debouncing or dirty-checking.
- **Improvement path:** Debounce settings persistence by 500ms. Only persist if settings have actually changed.

## Fragile Areas

### Frontend-Backend State Synchronization

- **Files:** `/Users/roland/Desktop/appsRoland/vPlayer/frontend/src/App.vue` (entire file, especially lines 810-907)
- **Why fragile:** The frontend maintains its own `playerState` ref and applies partial updates from Tauri events. If an event is missed or arrives out of order, the frontend state diverges from the backend. There is no full-state reconciliation on reconnect.
- **Safe modification:** Always emit full state snapshots, not partial updates. Add a periodic heartbeat that syncs full state.
- **Test coverage:** No automated tests for event ordering or missed events.

### Plugin Event Emitting with Mutex Lock

- **Files:** Multiple IPC command files acquire `app_state.plugin_bus.lock()` inside the command handler.
- **Why fragile:** If a plugin's `on_event` handler panics or blocks, the mutex remains poisoned. Subsequent commands will fail with "plugin bus lock poisoned".
- **Safe modification:** Use `std::sync::RwLock` instead of `Mutex` if read-heavy. Or spawn plugin events on a separate thread with a channel.
- **Test coverage:** Only basic unit tests in `sandbox.rs` and `loader.rs`. No integration tests for the bus.

### Audio Equalizer Graph Lifecycle

- **Files:** `/Users/roland/Desktop/appsRoland/vPlayer/frontend/src/composables/useAudioEqualizer.ts`
- **Why fragile:** The Web Audio API graph is created once and never torn down. If the video element is removed and a new one created (e.g., on source change), the `MediaElementAudioSourceNode` becomes invalid but the code does not detect this.
- **Safe modification:** Add a `watch` on the video element and re-create the graph when the element changes. Or use a single persistent audio element that only changes `src`.
- **Test coverage:** No tests for audio graph behavior.

### Tauri Event Listener Cleanup

- **Files:** `/Users/roland/Desktop/appsRoland/vPlayer/frontend/src/App.vue` (lines 808-922)
- **Why fragile:** `unlistenFns` is a module-level array. If `onMounted` runs multiple times (e.g., during HMR in development), listeners accumulate and old ones are never cleaned up.
- **Safe modification:** Use a `Set` or clear the array before pushing new listeners. Or use Vue's `onScopeDispose` pattern.
- **Test coverage:** No tests for listener lifecycle.

## Scaling Limits

### Bookmark Storage: Single JSON File

- **Current capacity:** Unbounded file growth. All bookmarks for all videos in one file.
- **Limit:** File I/O slows as the file grows. JSON parsing becomes expensive.
- **Scaling path:** Migrate to per-video bookmark files or SQLite.

### Plugin System: All Plugins in Memory

- **Current capacity:** All plugins (builtin + external) are loaded into memory at startup and stay there.
- **Limit:** Memory usage grows with number of plugins. No lazy loading.
- **Scaling path:** Implement lazy loading — only initialize a plugin when it receives its first event.

### Playlist: In-Memory Vec of Strings

- **Current capacity:** Unbounded `Vec<String>` in `AppState`.
- **Limit:** Large playlists (thousands of items) consume significant memory. No pagination or virtual scrolling in the UI.
- **Scaling path:** Add pagination to the playlist panel. Persist large playlists to disk and load on demand.

## Dependencies at Risk

### libmpv (Not Yet a Dependency)

- **Risk:** The project claims to be "built with Tauri and libmpv" but libmpv is not in `Cargo.toml`. When it is added, it will be a complex native dependency that requires:
  - System libmpv installation (or static linking)
  - OpenGL/GLES context setup
  - Platform-specific render loop integration
- **Impact:** This is the single biggest integration risk. libmpv's C API is powerful but complex. Getting OpenGL interop right across macOS/Windows/Linux is non-trivial.
- **Migration plan:** Start with `libmpv-sys` crate. Test on all target platforms early. Consider `mpv-rs` wrapper crate if it covers the needed APIs.

### rfd (File Dialog)

- **Risk:** `rfd` is a cross-platform file dialog crate. It is reliable but adds native dialog dependencies per platform.
- **Impact:** Low risk. Well-maintained crate.

### Vue 3 + Vite

- **Risk:** Standard frontend stack. No unusual risks.

## Missing Critical Features

### No Test Suite

- **Problem:** There are no frontend tests (no Vitest, no Jest, no Playwright). The only tests are a few Rust unit tests in `loader.rs` and `sandbox.rs`.
- **Blocks:** Cannot safely refactor. No regression protection.
- **Files:** Entire `/Users/roland/Desktop/appsRoland/vPlayer/frontend/src/` directory.

### No CI/CD Pipeline

- **Problem:** No GitHub Actions, no automated builds, no automated testing.
- **Blocks:** Cannot verify that the app builds on all platforms. Cannot run tests on PR.

### No Error Telemetry

- **Problem:** Errors are logged to `console.debug` or `eprintln` but never sent anywhere. Users must manually copy error info.
- **Blocks:** Cannot diagnose issues in production.

### No Update Mechanism

- **Problem:** No auto-updater configured. Tauri has built-in updater support but it is not enabled.
- **Blocks:** Users must manually download new versions.

### No TypeScript Strict Mode

- **Problem:** `tsconfig.json` not checked for strictness. Many `any` types used implicitly (e.g., `(el as any).audioTracks`, `(t as any).id`).
- **Blocks:** Type safety guarantees are weak. Refactoring is risky.

## Test Coverage Gaps

### Frontend: Zero Tests

- **What's not tested:** All Vue components, all composables, all API calls, all event handling.
- **Files:** All `.vue` and `.ts` files in `/Users/roland/Desktop/appsRoland/vPlayer/frontend/src/`
- **Risk:** Any refactoring of component logic, event handling, or state management could break functionality with no warning.
- **Priority:** High

### Backend: Minimal Tests

- **What's not tested:**
  - IPC commands (`commands.rs`, `subtitle.rs`, `mediainfo.rs`, `bookmark.rs`, `chapter.rs`, `config.rs`)
  - Plugin bus event dispatching
  - Plugin initialization and lifecycle
  - State management (`AppState`)
  - Error handling (`fallback.rs`)
  - Render pipeline (all stubs)
- **Files:** All `.rs` files except `loader.rs` and `sandbox.rs`
- **Risk:** Backend logic changes (e.g., adding a new command) have no automated verification.
- **Priority:** High

### Integration Tests: None

- **What's not tested:** Frontend-to-backend communication via Tauri IPC. Event emission and listening. File I/O operations.
- **Risk:** The contract between frontend and backend can drift without detection.
- **Priority:** High

---

*Concerns audit: 2026-06-16*
