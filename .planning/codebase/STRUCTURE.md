# Codebase Structure

**Analysis Date:** 2026-06-16

## Directory Layout

```
/Users/roland/Desktop/appsRoland/vPlayer/
├── frontend/                  # Vue 3 SPA (Tauri webview UI)
│   ├── src/
│   │   ├── api/
│   │   ├── components/
│   │   ├── composables/
│   │   ├── App.vue
│   │   ├── main.ts
│   │   ├── style.css
│   │   └── vite-env.d.ts
│   ├── index.html
│   ├── package.json
│   ├── vite.config.ts
│   ├── tsconfig.json
│   └── tsconfig.node.json
├── src-tauri/                 # Rust backend (Tauri v2)
│   ├── src/
│   │   ├── config.rs          # Settings persistence
│   │   ├── error/             # Error handling modules
│   │   ├── ipc/               # IPC commands, events, state
│   │   ├── main.rs            # Application entry point
│   │   ├── mpv/               # libmpv integration (scaffolded)
│   │   ├── plugin/            # Plugin system
│   │   ├── render/            # OpenGL rendering (scaffolded)
│   │   └── utils/             # Logging, path helpers
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── build.rs
│   └── capabilities/
├── docs/                      # Project documentation
├── scripts/                   # PR verification scripts
├── .planning/                 # GSD planning artifacts
├── DESIGN.md                  # Design system specification
├── CLAUDE.md                  # Project instructions
├── PLUGIN-SYSTEM.md           # Plugin system documentation
└── README.md
```

## Directory Purposes

**`frontend/src/`:**
- Purpose: All frontend source code for the Vue 3 application
- Contains: Vue SFCs, TypeScript modules, composables, global styles
- Key files: `App.vue` (root component), `main.ts` (entry point), `api/player.ts` (backend API client)

**`frontend/src/components/`:**
- Purpose: Vue single-file components
- Contains: 16 `.vue` files covering all UI surfaces (player, controls, popups, panels, plugin views)
- Key files: `App.vue`, `PlayerView.vue`, `ControlBar.vue`, `TopHud.vue`, `PluginPopup.vue`, `BookmarkView.vue`

**`frontend/src/composables/`:**
- Purpose: Reusable Vue 3 Composition API logic
- Contains: `useHotkeys.ts`, `useAudioEqualizer.ts`
- Key files: `useHotkeys.ts` (global keyboard shortcuts), `useAudioEqualizer.ts` (Web Audio API 8-band EQ)

**`frontend/src/api/`:**
- Purpose: Frontend-to-backend API bridge
- Contains: `player.ts` — all Tauri invoke wrappers and type definitions
- Key files: `player.ts` (exports 30+ async functions and TypeScript interfaces)

**`src-tauri/src/`:**
- Purpose: Rust backend source code
- Contains: IPC handlers, state management, plugin system, error handling, config
- Key files: `main.rs` (entry), `config.rs` (settings), `ipc/commands.rs` (command handlers), `ipc/state.rs` (AppState)

**`src-tauri/src/ipc/`:**
- Purpose: All Tauri IPC-related code
- Contains: Commands, events, shared state definitions
- Key files: `commands.rs` (25+ command handlers), `events.rs` (event emitters), `state.rs` (AppState), `bookmark.rs`, `subtitle.rs`, `chapter.rs`, `mediainfo.rs`, `config.rs`

**`src-tauri/src/plugin/`:**
- Purpose: Plugin system core
- Contains: Trait definitions, builtin plugins, registry, bus, loader, sandbox
- Key files: `mod.rs` (Plugin trait, PluginManifest, PluginInstance), `bus.rs` (event dispatch), `registry.rs` (PluginRegistry), `builtin/mod.rs` (builtin plugin factories)

**`src-tauri/src/plugin/builtin/`:**
- Purpose: Built-in plugin implementations
- Contains: 8 builtin plugins as Rust modules
- Key files: `bookmark.rs`, `chapter.rs`, `equalizer.rs`, `media_info.rs`, `screenshot.rs`, `speed.rs`, `subtitle_download.rs`

**`src-tauri/src/mpv/`:**
- Purpose: libmpv integration (scaffolded for future Phase 2+)
- Contains: `core.rs` (startup probe), `renderer.rs` (frame render stub), `event.rs` (event loop stub), `options.rs` (options stub)
- Status: `core.rs` has a `startup_probe()` that always succeeds unless `VPLAYER_FORCE_STARTUP_FATAL` is set

**`src-tauri/src/render/`:**
- Purpose: OpenGL rendering infrastructure (scaffolded)
- Contains: `gl_context.rs`, `texture.rs`, `frame.rs` (all empty scaffolds)
- Status: Not integrated into the application flow

**`src-tauri/src/error/`:**
- Purpose: Error handling and reporting
- Contains: `fallback.rs` (startup fatal error handling), `user_notify.rs`, `mod.rs`
- Key files: `fallback.rs` (emits `app:fatal_error` events, stores fatal state)

**`src-tauri/src/utils/`:**
- Purpose: Utility modules
- Contains: `paths.rs` (app data/log directory resolution), `logging.rs` (scaffold), `mod.rs`
- Key files: `paths.rs` (wraps Tauri path APIs)

**`docs/`:**
- Purpose: Project documentation and planning artifacts
- Contains: Architecture docs, workflow docs, milestone plans, design reviews
- Key files: `architecture.md`, `notes-view-plan.md`, `milestones.md`

**`scripts/`:**
- Purpose: PR verification shell scripts
- Contains: `verify-prN.sh` files (numbered 2 through 35)
- Pattern: Each script verifies a specific PR's changes

## Key File Locations

**Entry Points:**
- `/Users/roland/Desktop/appsRoland/vPlayer/frontend/src/main.ts` — Vue app bootstrap
- `/Users/roland/Desktop/appsRoland/vPlayer/frontend/index.html` — HTML shell (loads Google Fonts, mounts `#app`)
- `/Users/roland/Desktop/appsRoland/vPlayer/src-tauri/src/main.rs` — Rust application entry point

**Configuration:**
- `/Users/roland/Desktop/appsRoland/vPlayer/frontend/vite.config.ts` — Vite build config (port 5175, Vue plugin)
- `/Users/roland/Desktop/appsRoland/vPlayer/frontend/tsconfig.json` — TypeScript config (ES2020, strict, noUnusedLocals)
- `/Users/roland/Desktop/appsRoland/vPlayer/frontend/package.json` — Frontend dependencies (Vue 3.5, Tauri API 2.2, Vite 6)
- `/Users/roland/Desktop/appsRoland/vPlayer/src-tauri/Cargo.toml` — Rust package config (Tauri 2, serde, rfd, libloading)
- `/Users/roland/Desktop/appsRoland/vPlayer/src-tauri/tauri.conf.json` — Tauri app config (window size, asset protocol, build hooks)
- `/Users/roland/Desktop/appsRoland/vPlayer/src-tauri/capabilities/default.json` — Tauri capability definitions

**Core Logic:**
- `/Users/roland/Desktop/appsRoland/vPlayer/frontend/src/App.vue` — Root component: orchestrates all UI state, event listeners, IPC calls
- `/Users/roland/Desktop/appsRoland/vPlayer/frontend/src/api/player.ts` — All backend API calls and shared types
- `/Users/roland/Desktop/appsRoland/vPlayer/src-tauri/src/ipc/commands.rs` — All Tauri command handlers (play, pause, seek, volume, playlist, plugins, screenshot)
- `/Users/roland/Desktop/appsRoland/vPlayer/src-tauri/src/ipc/state.rs` — AppState struct with mutex-protected fields
- `/Users/roland/Desktop/appsRoland/vPlayer/src-tauri/src/config.rs` — Settings load/save logic (JSON file)

**Styling:**
- `/Users/roland/Desktop/appsRoland/vPlayer/frontend/src/style.css` — Global CSS variables, resets, animations, scrollbar styles, glass-panel utility

## Naming Conventions

**Files:**
- Vue components: PascalCase with `.vue` suffix — `PlayerView.vue`, `ControlBar.vue`, `BookmarkView.vue`
- TypeScript modules: camelCase with `.ts` suffix — `useHotkeys.ts`, `player.ts`
- Rust modules: `mod.rs` for directory modules, snake_case for file modules — `commands.rs`, `state.rs`, `bookmark.rs`
- Rust submodules within directories: directory name is the module name — `ipc/`, `plugin/`, `render/`

**Directories:**
- Frontend: plural nouns for collections — `components/`, `composables/`, `api/`
- Backend: singular nouns for domains — `ipc/`, `plugin/`, `render/`, `error/`, `utils/`
- Tauri convention: `src-tauri/` for the Rust backend

**Rust Naming:**
- Structs: PascalCase — `PlayerState`, `AppState`, `PluginManifest`
- Functions/variables: snake_case — `play_file`, `seek`, `startup_probe`
- Traits: PascalCase — `Plugin`
- Constants: SCREAMING_SNAKE_CASE — `EVT_PLAYER_STATE_CHANGE`
- Types: PascalCase with explicit type aliases — `PlayerState`, `PlaylistState`

**Vue Naming:**
- Components: PascalCase in imports, kebab-case in templates — `<player-view>` vs `import PlayerView`
- Props: camelCase in script, kebab-case in template — `:source-path` vs `sourcePath`
- Emits: camelCase event names — `@speed-change`, `@toggle-track`
- Refs: camelCase with "Ref" suffix — `playerViewRef`, `videoEl`
- Composables: camelCase with "use" prefix — `useHotkey`, `useAudioEqualizer`

## Where to Add New Code

**New Vue Component:**
- Implementation: `/Users/roland/Desktop/appsRoland/vPlayer/frontend/src/components/{ComponentName}.vue`
- Import in: `/Users/roland/Desktop/appsRoland/vPlayer/frontend/src/App.vue` (if top-level) or parent component
- If it's a plugin popup view: add conditional rendering in `PluginPopup.vue`

**New Composable:**
- Implementation: `/Users/roland/Desktop/appsRoland/vPlayer/frontend/src/composables/use{Feature}.ts`
- Import in: consuming components

**New IPC Command (Backend):**
- Handler: `/Users/roland/Desktop/appsRoland/vPlayer/src-tauri/src/ipc/commands.rs` (or domain-specific module like `ipc/bookmark.rs`)
- Register in: `/Users/roland/Desktop/appsRoland/vPlayer/src-tauri/src/main.rs` in the `generate_handler!` macro
- Frontend wrapper: `/Users/roland/Desktop/appsRoland/vPlayer/frontend/src/api/player.ts`

**New Plugin (Builtin):**
- Implementation: `/Users/roland/Desktop/appsRoland/vPlayer/src-tauri/src/plugin/builtin/{plugin_name}.rs`
- Export in: `/Users/roland/Desktop/appsRoland/vPlayer/src-tauri/src/plugin/builtin/mod.rs` (add to `all_builtins()`)
- Frontend UI (if needed): new component in `/Users/roland/Desktop/appsRoland/vPlayer/frontend/src/components/`, add to `PluginPopup.vue`

**New Settings Field:**
- Add to: `/Users/roland/Desktop/appsRoland/vPlayer/src-tauri/src/config.rs` (`PlayerSettings` struct)
- Add default in: `PlayerSettings::default()`
- Frontend type: `/Users/roland/Desktop/appsRoland/vPlayer/frontend/src/api/player.ts` (`PlayerSettings` type)

**New Backend Utility:**
- Implementation: `/Users/roland/Desktop/appsRoland/vPlayer/src-tauri/src/utils/{module}.rs`
- Export in: `/Users/roland/Desktop/appsRoland/vPlayer/src-tauri/src/utils/mod.rs`

## Special Directories

**`src-tauri/gen/schemas/`:**
- Purpose: Tauri-generated schema files for ACL, capabilities, and desktop config
- Generated: Yes (by Tauri CLI during build)
- Committed: Yes (checked into git)
- Files: `acl-manifests.json`, `capabilities.json`, `desktop-schema.json`, `macOS-schema.json`

**`src-tauri/target/`:**
- Purpose: Rust build artifacts
- Generated: Yes (by Cargo)
- Committed: No (in `.gitignore`)

**`frontend/dist/`:**
- Purpose: Vite production build output
- Generated: Yes (by `vite build`)
- Committed: No (in `.gitignore`)
- Consumed by: Tauri bundles this directory into the desktop app (`tauri.conf.json` `frontendDist`)

**`.planning/`:**
- Purpose: GSD workflow artifacts
- Generated: By GSD commands
- Committed: Yes

---

*Structure analysis: 2026-06-16*
