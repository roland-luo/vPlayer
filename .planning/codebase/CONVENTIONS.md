# Coding Conventions

**Analysis Date:** 2026-06-16

## Languages & Platforms

**Frontend:** TypeScript + Vue 3 (Composition API with `<script setup lang="ts">`)

**Backend:** Rust (Tauri 2.x application)

## Naming Patterns

### TypeScript / Vue

**Files:**
- Vue components: PascalCase with `.vue` suffix — `PlayerView.vue`, `ControlBar.vue`, `BookmarkView.vue`
- Composables: camelCase prefixed with `use` — `useHotkeys.ts`, `useAudioEqualizer.ts`
- API modules: camelCase — `player.ts`
- Style sheets: `style.css` (global), scoped `<style>` blocks in Vue SFCs

**Functions:**
- camelCase for all functions — `handlePlay()`, `togglePlay()`, `fetchPlugins()`
- Async functions use `async/await`, not raw promises — `async function handleOpenFile()`
- Event handlers prefixed with `handle` — `handleSeek()`, `handleVolumeChange()`
- Composable functions prefixed with `use` — `useHotkey()`, `useAudioEqualizer()`

**Variables:**
- camelCase — `playerState`, `currentMediaPath`, `isTauriRuntime`
- Refs in Vue use same name as the value — `const playerState = ref<PlayerState>(...)`
- Boolean flags prefixed with `is`/`has`/`show` — `isPlaying`, `hasVideo`, `showPluginPanel`
- Constants for magic numbers: `const AUTO_HIDE_MS = 2500;`, `const SEEK_OFFSET = 3;`

**Types/Interfaces:**
- PascalCase — `PlayerState`, `PlaylistState`, `AppFatalError`, `BookmarkEntry`
- Exported from `api/player.ts` for shared frontend/backend contracts
- Vue component prop types defined inline or in the same file

### Rust

**Files:**
- snake_case module files — `commands.rs`, `state.rs`, `bookmark.rs`
- Module entry points: `mod.rs`

**Types:**
- PascalCase structs and enums — `PlayerState`, `PluginError`, `StartupFatalState`
- Error enums use descriptive variants — `PluginError::LoadFailed(String)`

**Functions:**
- snake_case — `startup_probe()`, `load_settings()`, `emit_state_and_progress()`
- Tauri commands: `pub async fn` with snake_case names — `pub async fn play_file()`

**Variables:**
- snake_case — `app_state`, `startup_fatal`, `plugin_bus`

## Code Style

**Formatting:**
- No explicit formatter configured (no Prettier, no rustfmt config)
- Default `cargo fmt` behavior expected for Rust
- Vue/TypeScript formatting relies on editor defaults

**TypeScript Strictness:**
- Config: `/Users/roland/Desktop/appsRoland/vPlayer/frontend/tsconfig.json`
- `strict: true` — all strict checks enabled
- `noUnusedLocals: true` — unused locals are errors
- `noUnusedParameters: true` — unused parameters are errors
- `noFallthroughCasesInSwitch: true` — no switch fallthrough
- `noEmit: true` — type-check only, Vite handles compilation
- `allowImportingTsExtensions: true` — `.ts` imports allowed
- `moduleResolution: "bundler"` — modern resolution strategy

**Rust Strictness:**
- Standard `cargo check` / `cargo clippy` workflow
- `#[allow(dead_code)]` used sparingly in `events.rs` for unused event constants
- `#[cfg_attr(not(debug_assertions), windows_subsystem = "windows")]` for Windows release builds

## Import Organization

**TypeScript/Vue order:**
1. Vue core imports (`vue`, `@tauri-apps/api/*`)
2. Component imports (relative `./components/...`)
3. Composable imports (relative `./composables/...`)
4. API imports (relative `./api/...`)
5. Type imports grouped with `type` keyword

Example from `App.vue`:
```typescript
import { computed, nextTick, onMounted, onUnmounted, ref, type ComponentPublicInstance } from "vue";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import TopHud from "./components/TopHud.vue";
import { useHotkey } from "./composables/useHotkeys";
import { type PlayerState, type PlayerSettings } from "./api/player";
```

**Rust order:**
1. Standard library (`std::`)
2. External crates (`tauri::`, `serde::`)
3. Internal modules (`crate::`)

## Error Handling

**TypeScript/Vue:**
- `console.debug("[context] message", error)` for non-fatal errors — never `console.error` in production paths
- Try/catch with specific context tags: `[open-file]`, `[playlist]`, `[screenshot]`
- User-facing errors shown via toast overlays, not console
- Fatal errors use dedicated overlay UI with diagnostic reporting

**Rust:**
- `Result<T, String>` pattern for Tauri commands — human-readable error strings returned to frontend
- `map_err` with descriptive prefixes: `"state lock poisoned: {e}"`
- `eprintln!` for backend logging to stderr
- `panic::catch_unwind` around plugin event handlers for isolation
- Startup errors use dedicated `StartupError` struct with stage/code/message/suggestion fields

## Logging

**Framework:** `console.debug` (frontend), `eprintln!` (backend)

**Patterns:**
- Prefix all log messages with bracketed context: `[bookmark]`, `[config]`, `[plugin::init]`
- Use `console.debug` not `console.log` — debug level only
- Backend uses `eprintln!` to stderr, not structured logging (scaffold in `utils/logging.rs`)

## Component Patterns (Vue)

**SFC Structure:**
```vue
<template>
  <!-- template content -->
</template>

<script setup lang="ts">
  // imports
  // types/interfaces
  // props/emit definitions
  // reactive state
  // computed
  // functions
  // lifecycle hooks
  // watches
</script>

<style scoped>
  /* component styles */
</style>
```

**Props/Emits:**
- Use `defineProps<{}>()` and `defineEmits<{}>()` with explicit type annotations
- Props are read-only; state changes emitted to parent
- Parent component (`App.vue`) is the single source of truth for all shared state

**Scoped Styles:**
- All component styles use `<style scoped>`
- Global styles only in `style.css` (CSS custom properties, animations, scrollbar)
- CSS custom properties for design tokens: `--bg-base`, `--accent-cyan`, `--font-display`

## State Management

**Pattern:** Centralized state in `App.vue` with props-down/events-up
- No Pinia, no Vuex, no global store
- `App.vue` holds all reactive state and passes down via props
- Child components emit events back to `App.vue` for mutations
- Composables for cross-cutting concerns (hotkeys, audio equalizer)

## Rust Module Patterns

**Layered architecture:**
```
src-tauri/src/
  main.rs          # Entry point, Tauri builder setup
  config.rs        # Settings persistence (JSON on disk)
  error/           # Error handling (fallback, user_notify)
  ipc/             # Tauri commands, events, shared state
  mpv/             # Video player core (scaffold for libmpv)
  plugin/          # Plugin system (trait, bus, registry, builtins)
  render/          # OpenGL rendering (scaffold)
  utils/           # Helpers (paths, logging scaffold)
```

**State sharing:** `AppState` struct with `Mutex<T>` fields managed by Tauri:
```rust
pub struct AppState {
    pub player: Mutex<PlayerState>,
    pub playlist: Mutex<PlaylistState>,
    pub startup_fatal: Mutex<Option<StartupFatalState>>,
    pub plugin_bus: Mutex<PluginBus>,
    pub plugin_registry: Mutex<PluginRegistry>,
    pub settings: Mutex<PlayerSettings>,
}
```

## Documentation Practices

**Code comments:**
- Minimal inline comments — code is expected to be self-explanatory
- Scaffold modules have `// Week 1 scaffold: description` header
- Complex logic gets brief explanation (e.g., `AbortError` handling in `PlayerView.vue`)
- No JSDoc/TSDoc on functions

**Project documentation:**
- `DESIGN.md` — mandatory read before any UI changes (design system, colors, typography, motion)
- `PLUGIN-SYSTEM.md` — plugin architecture specification
- `README.md` — project overview
- `TODOS.md` — task tracking
- `AGENTS.md` / `CLAUDE.md` — AI agent instructions (see AI Rules section)

## AI / Claude-Specific Rules

**From `AGENTS.md` and `CLAUDE.md`:**

1. **Design System Compliance:** Always read `DESIGN.md` before any visual or UI decision. All font choices, colors, spacing, and aesthetic direction are defined there. Do not deviate without explicit user approval. In QA mode, flag any code that doesn't match `DESIGN.md`.

2. **Skill Routing:** When a user request matches an available skill, ALWAYS invoke it using the Skill tool as the FIRST action. Do NOT answer directly. Key routing:
   - Product ideas/brainstorming → `office-hours`
   - Bugs/errors/500s → `investigate`
   - Ship/deploy/push/PR → `ship`
   - QA/test/find bugs → `qa`
   - Code review/diff check → `review`
   - Update docs after shipping → `document-release`
   - Weekly retro → `retro`
   - Design system/brand → `design-consultation`
   - Visual audit/polish → `design-review`
   - Architecture review → `plan-eng-review`
   - Save progress/checkpoint → `checkpoint`
   - Code quality/health check → `health`

3. **RTK (Token Optimization):** Always prefix commands with `rtk`. Even in `&&` chains. Example: `rtk git add . && rtk git commit -m "msg" && rtk git push`

## Commit Message Patterns

**Format:** `type: description` (lowercase type, imperative mood)

**Types observed:**
- `feat:` — new features
- `fix:` — bug fixes
- `chore:` — tooling, config, docs
- `feat:` with Chinese descriptions also used (e.g., `feat: 持久化存储`)

**Examples from git log:**
- `feat: redesign player controls and add playback shortcuts`
- `fix: record bookmark timestamp from current playback`
- `chore: add project workflow docs and local tooling config`
- `feat: polish notes workflow with hotkeys and auto-pause toggle`

## Runtime Detection

**Tauri runtime guard:** All Tauri API calls are guarded by runtime detection:
```typescript
const isTauriRuntime =
  typeof window !== "undefined" &&
  ("__TAURI_INTERNALS__" in window || "__TAURI__" in window);
```
This allows the frontend to run in a browser for development without Tauri backend.

---

*Convention analysis: 2026-06-16*
