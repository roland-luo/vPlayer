# Testing Patterns

**Analysis Date:** 2026-06-16

## Test Framework

**Runner:** Not configured. No test framework is installed or configured in the project.

**Frontend:** No Vitest, Jest, or other test runner in `package.json` devDependencies.

**Backend:** No test dependencies in `Cargo.toml`. No `#[cfg(test)]` modules in any Rust source file.

**Assertion Library:** Not applicable.

**Run Commands:**
```bash
# No test commands defined in package.json scripts
# No cargo test workflow established
```

## Test File Organization

**Location:** Not applicable — no test files exist.

**Naming:** Not applicable.

**Structure:** The project has zero test files across both frontend (`frontend/src/`) and backend (`src-tauri/src/`).

## Coverage

**Requirements:** None enforced. No coverage tooling configured.

**View Coverage:** Not applicable.

## Test Types

**Unit Tests:** Not present.

**Integration Tests:** Not present.

**E2E Tests:** Not present. No Playwright, Cypress, or Tauri-specific E2E test setup.

## Manual QA Approach

The project relies entirely on manual testing during development. Evidence:

1. **Debug commands built into the UI:**
   - `emitDebugVideoError()` — Tauri command at `/Users/roland/Desktop/appsRoland/vPlayer/src-tauri/src/ipc/commands.rs:373`
   - `emitDebugFatalError()` — Tauri command at `/Users/roland/Desktop/appsRoland/vPlayer/src-tauri/src/ipc/commands.rs:383`
   - Debug buttons visible in `TopHud.vue`: "Err" and "Fatal" buttons

2. **Runtime environment detection:**
   - Frontend detects Tauri runtime via `window.__TAURI_INTERNALS__` / `window.__TAURI__`
   - Allows frontend to run in browser without backend for UI testing
   - Non-Tauri paths return mock data or no-op

3. **DevTools:**
   - `window.open_devtools()` called automatically in debug builds (`main.rs:92`)
   - Vite dev server on port 5175 for hot-reload frontend development

4. **Console-based debugging:**
   - `console.debug()` used throughout frontend for error tracing
   - `eprintln!` used in backend for stderr logging

## Known Test Gaps

### Critical Untested Areas

**1. Tauri IPC Commands (Backend)**
- All 30+ Tauri commands in `ipc/commands.rs` are untested
- No test coverage for state locking, error handling, or event emission
- File: `/Users/roland/Desktop/appsRoland/vPlayer/src-tauri/src/ipc/commands.rs` (551 lines)

**2. Plugin System**
- Plugin bus event dispatch with `catch_unwind` isolation
- Plugin registry state management
- Builtin plugin lifecycle (load/unload/event)
- Files: `/Users/roland/Desktop/appsRoland/vPlayer/src-tauri/src/plugin/bus.rs`, `/Users/roland/Desktop/appsRoland/vPlayer/src-tauri/src/plugin/registry.rs`, `/Users/roland/Desktop/appsRoland/vPlayer/src-tauri/src/plugin/mod.rs`

**3. Bookmark Persistence**
- JSON read/write for bookmarks
- CRUD operations (list, add, delete)
- File: `/Users/roland/Desktop/appsRoland/vPlayer/src-tauri/src/ipc/bookmark.rs` (166 lines)

**4. Settings Persistence**
- Settings load/save with fallback to defaults
- JSON serialization/deserialization error handling
- File: `/Users/roland/Desktop/appsRoland/vPlayer/src-tauri/src/config.rs` (117 lines)

**5. Screenshot Capture**
- ffmpeg integration for frame extraction
- File dialog integration
- File: `/Users/roland/Desktop/appsRoland/vPlayer/src-tauri/src/ipc/commands.rs` (capture_screenshot function)

**6. Frontend Vue Components**
- No component unit tests for any Vue SFC
- Complex components like `App.vue` (1183 lines), `BookmarkView.vue` (823 lines), `ControlBar.vue` (418 lines)
- Hotkey composable (`useHotkeys.ts`) untested
- Audio equalizer composable (`useAudioEqualizer.ts`) untested

**7. Video Playback Logic**
- `PlayerView.vue` — text track management, audio track switching, subtitle loading
- Playback state synchronization between HTML5 video and Tauri backend
- UI compensation timer using `requestAnimationFrame`

**8. Playlist Management**
- Playlist navigation (next/prev)
- Item removal with index adjustment logic
- State persistence across sessions

### Risk Assessment

| Area | Risk Level | Why |
|------|-----------|-----|
| State locking in Tauri commands | High | Mutex poisoning could crash the app; no tests verify graceful degradation |
| Plugin `catch_unwind` isolation | High | Plugin panics could corrupt state; isolation logic untested |
| Bookmark JSON persistence | Medium | Data corruption could lose user bookmarks; fallback to defaults untested |
| Screenshot ffmpeg integration | Medium | External dependency (ffmpeg) not available in all environments |
| Frontend playback sync | Medium | RAF-based UI compensation could drift; no automated verification |
| Settings serialization | Low | JSON format is simple; defaults fallback is straightforward |

## Recommended Testing Strategy

**Short-term (immediate value):**

1. **Add Rust unit tests** for core logic:
   - `PluginRegistry` (register, set_enabled, record_error)
   - `PluginBus` (emit with success/failure/panic scenarios)
   - `config.rs` (load_settings with missing/corrupt files)
   - `bookmark.rs` (CRUD operations with temp files)

2. **Add frontend component tests** with Vitest + Vue Test Utils:
   - `useHotkeys` composable (keyboard event handling)
   - `PlayerView` (prop changes, event emission)
   - `BookmarkView` (add/delete/seek interactions)

**Medium-term:**

3. **Add Tauri integration tests** using `tauri::test` mocking:
   - IPC command invocation with mocked state
   - Event emission verification

4. **Add E2E tests** with Playwright or Tauri's built-in test runner:
   - File picker flow
   - Playback controls
   - Bookmark workflow end-to-end

**Tooling needed:**
- Frontend: `vitest`, `@vue/test-utils`, `jsdom`
- Backend: `cargo test` (native, no extra deps needed)
- E2E: `@playwright/test` or `tauri-driver`

## CI / Checks

**No CI configured.** No GitHub Actions, no pre-commit hooks (only `.sample` files in `.git/hooks/`).

**Build verification:**
```bash
# Frontend build (includes type check)
cd frontend && pnpm build    # runs vue-tsc && vite build

# Backend build
cd src-tauri && cargo build
```

**Type checking:**
```bash
cd frontend && npx vue-tsc --noEmit
```

**Linting:** Not configured. No ESLint, no Biome, no Clippy config.

---

*Testing analysis: 2026-06-16*
