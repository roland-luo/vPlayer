# Technology Stack

**Analysis Date:** 2026-06-16

## Languages

**Primary:**
- TypeScript (ES2020) — Frontend application logic, Vue SFC components, composables
- Rust (Edition 2021, minimum version 1.70) — Backend/desktop runtime, IPC commands, plugin system
- CSS (custom properties) — Component-scoped styling with design tokens

**Secondary:**
- Shell (bash) — Tauri dev/build hooks, PR verification scripts
- HTML — Single-page app entry point (`frontend/index.html`)

## Runtime

**Environment:**
- Tauri v2 — Desktop application framework (Rust backend + Web frontend)
- Default window: 1280x720, transparent, decorated, centered, resizable

**Package Manager:**
- pnpm — Frontend dependency management (inferred from `beforeDevCommand`/`beforeBuildCommand` in `tauri.conf.json`)
- Cargo — Rust dependency management
- Lockfile: Not present in repository (no `pnpm-lock.yaml` or `Cargo.lock` committed)

## Frameworks

**Core:**
- Vue 3.5.13 (`vue`) — Frontend UI framework with Composition API
- Tauri 2 (`tauri`, `tauri-build`) — Desktop app shell, IPC bridge, native APIs

**Testing:**
- Rust built-in test framework (`cargo test`) — Unit tests in plugin loader, permission sandbox
- No frontend test framework detected

**Build/Dev:**
- Vite 6.0.0 (`vite`) — Frontend build tool and dev server (port 5175)
- `@vitejs/plugin-vue` 5.2.1 — Vue SFC compilation
- TypeScript 5.6.3 (`typescript`) — Type checking
- `vue-tsc` 2.2.0 — Vue-specific TypeScript compiler
- `@tauri-apps/cli` 2.2.0 — Tauri CLI for building and dev

## Key Dependencies

**Critical:**
- `@tauri-apps/api` 2.2.0 — Frontend Tauri API (invoke, events, window, file path conversion)
- `tauri` 2 — Backend framework with `protocol-asset` feature for local file access
- `tauri-plugin-shell` 2 — Shell command execution from backend
- `serde` + `serde_json` — Serialization for IPC, settings, bookmarks, plugin manifests
- `rfd` 0.15 — Native file dialogs (open/save)
- `libloading` 0.8 — Dynamic library loading for external plugins (Phase 2)

**Infrastructure:**
- `convertFileSrc` from `@tauri-apps/api/core` — Converts local file paths to Tauri asset URLs for `<video>` src
- `getCurrentWindow` from `@tauri-apps/api/window` — Fullscreen toggle via Tauri window API
- HTML5 `<video>` element — Core media playback (current implementation)
- Web Audio API (`AudioContext`, `BiquadFilterNode`) — 8-band equalizer on frontend

## Configuration

**Environment:**
- No `.env` files present in repository
- Debug env vars used for error simulation:
  - `VPLAYER_FORCE_STARTUP_FATAL=1` — Simulates startup fatal error (`src-tauri/src/mpv/core.rs`)
  - `VPLAYER_FORCE_RENDER_FATAL=1` — Simulates render texture allocation failure (`src-tauri/src/mpv/renderer.rs`)

**Build:**
- `frontend/vite.config.ts` — Vite config with Vue plugin, port 5175, host 127.0.0.1
- `frontend/tsconfig.json` — TypeScript: ES2020, strict mode, noUnusedLocals, noUnusedParameters
- `frontend/tsconfig.node.json` — Node-specific TS config for Vite config file
- `src-tauri/tauri.conf.json` — Tauri app config: dev URL, asset protocol scope, window settings, bundle
- `src-tauri/Cargo.toml` — Rust package manifest with Tauri dependencies

**Asset Protocol Scope:**
- `$HOME/**`, `$DESKTOP/**`, `$DOCUMENT/**`, `$DOWNLOAD/**`, `$VIDEO/**`
- Enables local video file playback via `convertFileSrc`

## Platform Requirements

**Development:**
- Rust toolchain (>= 1.70)
- Node.js + pnpm
- System dependencies: `ffmpeg` (screenshot capture), `ffprobe` (media info, chapters)

**Production:**
- Desktop targets: macOS, Windows, Linux (Tauri cross-platform)
- Bundle targets: `all` (configured in `tauri.conf.json`)
- External runtime dependencies: `ffmpeg`, `ffprobe` must be available on system PATH

---

*Stack analysis: 2026-06-16*
