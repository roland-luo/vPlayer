# Design System — vPlayer

## Product Context
- **What this is:** A cross-platform desktop video player built with Tauri 2 + libmpv. The UI overlays a native OpenGL video render surface via WebView.
- **Who it's for:** Desktop users on Windows, macOS, and Linux who want a modern, immersive video playback experience with extensible plugin support.
- **Space/industry:** Video/media player software. Peers: IINA, PotPlayer, MPV, VLC.
- **Project type:** Desktop application (web-based frontend over native render layer)

## Aesthetic Direction
- **Direction:** Neo-Industrial Cyber-Minimal
- **Decoration level:** Intentional — subtle glassmorphism panels, 1px borders, controlled neon glow only on interactive states
- **Mood:** Like sitting in a spacecraft cockpit watching a screen. Precise, alive, and unobtrusive. The UI should feel like instrumentation that enhances the video rather than decoration that competes with it.
- **Anti-patterns:** No purple gradients, no generic icon grids, no centered marketing layouts, no bubbly border-radius on everything. This is a tool, not a landing page.

## Typography
- **Display/Hero:** Rajdhani (Google Fonts) — wide, technical, uppercase. Evokes NASA instrument panels and sci-fi HUDs. Used for titles, HUD labels, buttons, and any text that needs to feel like a control panel readout.
- **Body:** DM Sans (Google Fonts) — clean, readable at small sizes, modern grotesque. Used for descriptions, settings labels, playlist item names, and all body copy.
- **UI/Labels:** Same as body (DM Sans) — consistent with the display font used for structural labels
- **Data/Tables:** JetBrains Mono (Google Fonts) — monospace with programming aesthetics. Critical for timecodes (`00:14:32`), resolutions (`1920x1080`), frame rates (`23.976fps`), and codec info (`DTS-HD MA`). Supports tabular-nums for aligned numeric columns.
- **Code:** JetBrains Mono
- **Loading:** Google Fonts CDN via `<link>` tags. All three families loaded in one request.
- **Scale:**
  - `xs`: 10px — HUD micro labels, status line items
  - `sm`: 12px — badges, metadata, playlist secondary text
  - `base`: 14px — body text, settings labels, button labels
  - `lg`: 16px — descriptions, form inputs
  - `xl`: 18px — section headings, card titles
  - `2xl`: 24px — error titles, modal headings
  - `3xl`: 32px — page titles
  - `4xl`: 48px — hero display (brand moments, empty states)

**Text treatment rules:**
- All display text (Rajdhani) is uppercase with `letter-spacing: 0.04em` minimum
- Button labels: uppercase, `letter-spacing: 0.06em`, font-weight 600
- HUD labels: uppercase, `letter-spacing: 0.08em`, font-size 10–13px
- Timecodes always use tabular nums: `font-variant-numeric: tabular-nums`

## Color
- **Approach:** Restrained — 2 accent colors with strict semantic separation, dark neutrals dominate
- **Primary:** `#00E5FF` (Electric Cyan) — interactive states, progress bars, play indicators, active toggles, focus rings. Represents "normal/forward/active".
- **Secondary:** `#FF006E` (Alert Magenta) — errors, recording states, warnings, destructive actions. Represents "alert/recording/reverse". Used sparingly.
- **Neutrals (cool grays with slight blue tint):**
  - Base: `#08080C` — main background, video canvas backing
  - Surface: `#12121A` — panels, cards, controls background
  - Elevated: `#1A1A24` — hover states, inputs, elevated surfaces
  - Border subtle: `rgba(255,255,255,0.06)` — default borders, dividers
  - Border glow: `rgba(0,229,255,0.15)` — active/focus borders
- **Text:**
  - Primary: `#E8ECF1` — headings, body, primary UI text
  - Muted: `#6B7280` — secondary labels, metadata, disabled states
- **Semantic:**
  - Success: `#00E5FF` (cyan doubles as success — "system nominal")
  - Warning: `#FF006E` (magenta doubles as warning — "attention required")
  - Error: `#FF006E`
  - Info: `#6B7280`
- **Dark mode:** This is the default. Light mode inverts the neutral scale and desaturates accents by ~10%.

**Color usage rules:**
- Cyan is the default accent. Magenta only appears for alerts, recording badges, and error states.
- Never use both accents on the same element.
- Glow effects (`box-shadow`) should never exceed 24px blur — subtlety is the goal.

## Spacing
- **Base unit:** 4px
- **Density:** Comfortable — video player UI needs breathing room, not data-dense dashboards
- **Scale:**
  - `2xs`: 2px — icon gaps, hairline offsets
  - `xs`: 4px — tight internal padding
  - `sm`: 8px — button padding-y, list item gaps
  - `md`: 12px — card internal padding
  - `lg`: 16px — section gaps, container padding
  - `xl`: 24px — major section separations
  - `2xl`: 32px — page-level padding
  - `3xl`: 48px — hero spacing

## Layout
- **Approach:** Immersive-Hybrid — video is the absolute focus. UI exists as floating HUD layers that appear on demand.
- **Grid:** Not strictly grid-disciplined — the player interface is freeform overlay. Settings and playlist panels use a loose 12-column grid when in dedicated views.
- **Max content width:** 100% (fullscreen-first), settings panels max 480px
- **Border radius:**
  - `sm`: 4px — inputs, badges, small tags
  - `md`: 8px — buttons, cards, list items
  - `lg`: 12px — panels, modals, sidebars
  - `full`: 9999px — avatars, toggle switches, circular buttons

**Player interface structure:**
1. **Video layer** — occupies 100% of window, black backing `#08080C`
2. **Top HUD** (36px height) — always visible during playback. Left: video title (Rajdhani uppercase). Right: technical readouts in mono (codec, resolution, fps, audio format) + status pulse dot. Fades out after 3s of mouse inactivity unless pinned.
3. **Bottom controls** — appears on mouse move. Gradient fade from bottom. Contains: progress bar (full width), control buttons row (play/pause, skip, volume, time display, settings, fullscreen).
4. **Side panels** (playlist, settings) — slide in from right, 280px width, glassmorphism background with `backdrop-filter: blur(16px)`.
5. **Error overlays** — centered modal, glassmorphism, magenta border tint for fatal errors.

## Motion
- **Approach:** Intentional — every animation serves a functional purpose (feedback, orientation, state change). No decorative motion.
- **Easing:**
  - Enter: `ease-out` — elements arrive quickly and settle
  - Exit: `ease-in` — elements leave decisively (faster than they arrive)
  - Move: `ease-in-out` — smooth transitions for position/size changes
- **Duration tokens:**
  - Micro: 50–100ms — button press feedback, color changes
  - Short: 150–250ms — hover states, opacity changes, control bar fade
  - Medium: 250–400ms — panel slides, sidebar open/close
  - Long: 400–700ms — page transitions, major state changes

**Specific animations:**

1. **Title Sweep (signature effect)** — `background-clip: text` gradient animation. A highlight sweeps across display text from left to right.
   ```css
   background: linear-gradient(90deg, var(--text-primary) 30%, var(--accent-cyan) 45%, #80F7FF 50%, var(--accent-cyan) 55%, var(--text-primary) 70%);
   background-size: 200% auto;
   -webkit-background-clip: text;
   -webkit-text-fill-color: transparent;
   animation: sweep 2.5s linear infinite;
   ```
   Used for: brand logo, hero headings, loading screens, major section titles.

2. **Breathing Pulse** — opacity oscillation on idle indicators.
   ```css
   animation: breathe 3s ease-in-out infinite;
   /* 0%,100% { opacity: 0.35; } 50% { opacity: 1; } */
   ```
   Used for: status dot in top HUD, paused play button glow, background download indicator.

3. **Progress Bar Glow** — the fill's box-shadow pulses subtly during playback.
   ```css
   animation: barGlow 3s ease-in-out infinite;
   /* 0%,100% { box-shadow: 0 0 6px rgba(0,229,255,0.2); }
      50% { box-shadow: 0 0 14px rgba(0,229,255,0.5); } */
   ```

4. **Control Bar Enter/Exit:**
   - Enter: `opacity 0→1` + `translateY(8px→0)`, 200ms ease-out
   - Exit: `opacity 1→0`, 150ms ease-in
   - Delay before exit: 2.5s of mouse inactivity

5. **Panel Slide:** `translateX(100%→0)` with 250ms ease-out, backdrop blur fades in simultaneously.

6. **Button Hover:** border color transition + `box-shadow` glow activation, 150ms ease-out.

## Glassmorphism Specification
Used for: control overlays, side panels, error modals.
```css
.glass-panel {
  background: rgba(8, 8, 12, 0.85);
  backdrop-filter: blur(16px);
  -webkit-backdrop-filter: blur(16px);
  border: 1px solid rgba(255, 255, 255, 0.06);
}
```
On light theme, background becomes `rgba(255, 255, 255, 0.85)` with `border: 1px solid rgba(0, 0, 0, 0.06)`.

## Component Patterns

### Buttons
- **Primary:** Electric cyan bg (`#00E5FF`), dark text, uppercase Rajdhani 600, `letter-spacing: 0.06em`. Hover: glow shadow `0 0 24px rgba(0,229,255,0.35)`.
- **Secondary:** Transparent bg, white border `1px solid rgba(255,255,255,0.06)`. Hover: cyan border + cyan text + glow.
- **Ghost:** Transparent, no border. Hover: white text.

### Progress Bar
- Track: `height: 4px`, `background: rgba(255,255,255,0.08)`, `border-radius: 2px`
- Fill: electric cyan with breathing glow animation
- Thumb: 14px circle, cyan, glow shadow. Hidden when mouse is away from control bar, appears on hover.
- Hover expansion: track height animates to 6px on hover (100ms).

### Toggle Switch
- Track: 40px x 22px, rounded. Off: dark bg + gray thumb. On: cyan-tinted bg + cyan thumb with glow.
- Transition: 200ms ease-out for thumb position and color.

### Badges
- Font: JetBrains Mono, 11px, uppercase, `letter-spacing: 0.05em`
- Cyan badge: `rgba(0,229,255,0.1)` bg + `rgba(0,229,255,0.2)` border
- Magenta badge: `rgba(255,0,110,0.1)` bg + `rgba(255,0,110,0.2)` border

## Decisions Log
| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-04-22 | Initial design system created | Created by /design-consultation based on vPlayer architecture docs and user preference for cyberpunk + breathing UI |
| 2026-04-22 | Rajdhani chosen as display font | User requested "flowing light" titles + tech feel; Rajdhani's wide stance and space-agency aesthetic fits HUD instrumentation better than Space Grotesk |
| 2026-04-22 | Electric cyan `#00E5FF` as primary accent | Revised from standard cyan `#00F0FF` to be more electric/blue-tinted for stronger cold tech feel |
| 2026-04-22 | Top HUD + bottom controls layout | User wanted cleaner bottom bar. Technical readouts moved to persistent top HUD, bottom reserved for playback controls only |
| 2026-04-22 | Title sweep animation implemented | CSS `background-clip: text` + gradient shift. Signature cyberpunk effect for brand moments and major headings |
| 2026-04-22 | Dual-accent system (cyan + magenta) | Cyan = normal/active, magenta = alert/recording. Provides immediate semantic color coding without extra UI chrome |
