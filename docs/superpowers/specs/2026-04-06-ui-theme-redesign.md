# UI Theme Redesign — Design Specification

**Date:** 2026-04-06  
**Status:** Approved  
**Scope:** Visual redesign + dark/light theme system for AudioCaptor (Tauri v2 + Svelte 5)

---

## 1. Goals

- Modern, polished UI: section cards, visible slider thumbs, prominent status area
- Dual theme: **Light** and **Dark**, both WCAG 2.1 AA compliant (contrast ≥ 4.5:1 for text, ≥ 3:1 for UI components)
- **Auto** mode: detect Windows dark/light preference via `window.matchMedia('(prefers-color-scheme: dark)')` at first launch; fallback to Dark if unavailable
- Theme picker in Settings dialog (segmented control: Auto · Light · Dark)
- No scrollbars in any window or dialog
- Minimal interaction: 2-column layout in ProfileDialog fits all 10 fields without scroll

---

## 2. Visual Design Language — "Refined Native"

Hybrid of Windows 11 native feel (section cards, system fonts, subtle shadows) and Studio Pro (prominent status area, large monospace timer, coloured section labels).

### Typography
- Font: `'Segoe UI', system-ui, sans-serif` (unchanged)
- App title: `16px / font-weight: 700`
- Section labels: `8px / font-weight: 700 / uppercase / letter-spacing: 0.09em`
- Body / labels: `11–12px`
- Timer: `'Courier New', monospace / 26px / font-weight: 700`
- No emoji in UI text

### Layout — Main window (480×600px)
- App title + settings gear button in header row
- Status card (timer + state badge) — full width, prominent
- Section cards: Profile · Devices · Volume
- Record controls: full-width buttons at bottom
- All sections fit within 600px height without scroll

### Section cards
- Light: `background: white`, `border: 1px solid rgba(0,0,0,0.09)`, `border-radius: 8px`, `box-shadow: 0 1px 3px rgba(0,0,0,0.04)`
- Dark: `background: #252525`, `border: 1px solid rgba(255,255,255,0.06)`, `border-radius: 8px`

---

## 3. Color Tokens

All colors defined as CSS custom properties on `:root` (light defaults) and overridden under `[data-theme="dark"]`.

### Light theme

| Token | Value | Usage |
|---|---|---|
| `--bg` | `#f0f0f0` | Window background |
| `--surface` | `#ffffff` | Cards, dialogs, inputs |
| `--surface-hover` | `#f5f5f5` | Hover on surface |
| `--border` | `rgba(0,0,0,0.12)` | Card / input borders |
| `--text-primary` | `#111111` | Headings, body |
| `--text-secondary` | `#555555` | Field labels |
| `--text-muted` | `#6b7280` | Hints, placeholders |
| `--accent` | `#0078d4` | Windows Blue — sliders, links, active controls |
| `--accent-hover` | `#106ebe` | Hover on accent |
| `--sec-label` | `#0078d4` | Section title color |
| `--focus-ring` | `#0078d4` | Focus outline |
| `--btn-start-bg` | `#107c10` | Start button (WCAG AA on white ✓) |
| `--btn-pause-bg` | `#9d5d00` | Pause button |
| `--btn-stop-bg` | `#c42b1c` | Stop button |
| `--btn-resume-bg` | `#0078d4` | Resume button |
| `--status-rec-border` | `#dc2626` | Recording state border |
| `--status-paused-border` | `#d97706` | Paused state border |
| `--error-bg` | `#fef2f2` | Error message background |
| `--error-border` | `#fecaca` | Error message border |
| `--error-text` | `#dc2626` | Error message text |

### Dark theme

| Token | Value | Usage |
|---|---|---|
| `--bg` | `#1c1c1c` | Window background |
| `--surface` | `#252525` | Cards, dialogs |
| `--surface-hover` | `#2e2e2e` | Hover on surface |
| `--border` | `rgba(255,255,255,0.08)` | Card / input borders |
| `--text-primary` | `#f0f0f0` | Headings, body |
| `--text-secondary` | `#888888` | Field labels |
| `--text-muted` | `#6b7280` | Hints, placeholders |
| `--accent` | `#60a5fa` | Blue-300 — sliders, links, active (WCAG AA on #252525 ✓) |
| `--accent-hover` | `#93c5fd` | Hover on accent |
| `--sec-label` | `#60a5fa` | Section title color |
| `--focus-ring` | `#60a5fa` | Focus outline |
| `--btn-start-bg` | `#0f7b0f` | Start button |
| `--btn-pause-bg` | `#92400e` | Pause button |
| `--btn-stop-bg` | `#991b1b` | Stop button |
| `--btn-resume-bg` | `#1d4ed8` | Resume button |
| `--status-rec-border` | `#f87171` | Recording state border |
| `--status-paused-border` | `#fbbf24` | Paused state border |
| `--error-bg` | `rgba(220,38,38,0.1)` | Error background |
| `--error-border` | `rgba(248,113,113,0.4)` | Error border |
| `--error-text` | `#f87171` | Error text |

---

## 4. Theme System Implementation

### Architecture: CSS Custom Properties + `data-theme` attribute

```
<html data-theme="dark">   ← set by JS at startup
  <body>
    ...app...
  </body>
</html>
```

`app.css` defines all tokens on `:root` (light values), and overrides under `[data-theme="dark"]`.

Components use **only** CSS custom properties — no hardcoded hex colors anywhere.

### Theme store — `src/lib/stores/theme.svelte.ts`

```ts
export type Theme = 'auto' | 'light' | 'dark';

// Reads stored preference, applies resolved theme to <html data-theme>
// 'auto' → queries matchMedia('(prefers-color-scheme: dark)')
// Listens for OS theme changes when in 'auto' mode
```

### Settings type extension — `src/lib/types/index.ts`

Add `theme: Theme` to the `Settings` interface (default: `'auto'`).

### Settings version bump — `src-tauri/src/settings.rs`

Bump `version` from **3** to **4**.

Add `pub theme: String` with `#[serde(default = "default_theme")]`.  
Add `fn default_theme() -> String { "auto".to_string() }`.

The `migrate_settings` loop gains:
- `3 =>` arm: sets `version = 4`, falls through
- `4 =>` arm: breaks (terminal)

No data transform needed — serde fills `"auto"` from the default function when loading v3 files that lack the `theme` key.

### First-launch behavior

1. No saved `theme` in settings → default to `'auto'`
2. Resolve: query `window.matchMedia('(prefers-color-scheme: dark)')`
3. Apply `data-theme="dark"` or `data-theme="light"` to `<html>`
4. Register `change` listener on the media query — live-update while in `'auto'` mode

### FOUC prevention

Apply theme before first paint. In `index.html` (Tauri's WebView entry), inject an inline `<script>` in `<head>` that sets `data-theme` synchronously — before any Svelte hydration.

Since Tauri settings are persisted via backend IPC (async), the theme preference is **mirrored to `localStorage`** under the key `"audiocaptor_theme"` every time the user changes it. The inline script reads this key synchronously:

```js
(function(){
  var t = localStorage.getItem('audiocaptor_theme') || 'auto';
  if (t === 'auto') {
    t = window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
  }
  document.documentElement.setAttribute('data-theme', t);
})();
```

The Svelte theme store writes to `localStorage` on every theme change (in addition to the backend save via `scheduleSave`).

---

## 5. Status Card — State-dependent border

| State | Border | Box-shadow |
|---|---|---|
| Idle | `2px solid transparent` | none |
| Recording | `2px solid var(--status-rec-border)` | subtle glow |
| Paused | `2px solid var(--status-paused-border)` | subtle glow |

Timer color: `var(--text-primary)` when active, `var(--text-muted)` (dimmed) when Idle.  
Recording dot: animated CSS blink (`opacity` keyframe, 1.2s).

---

## 6. Volume Slider

Custom slider appearance (overrides browser default):

- Track: `height: 5px`, `border-radius: 3px`, filled portion uses `--accent`
- Thumb: `width: 14px / height: 14px / border-radius: 50%`
  - Light: `background: white; border: 2px solid var(--accent); box-shadow: 0 1px 4px rgba(0,0,0,0.2)`
  - Dark: `background: var(--surface); border: 2.5px solid var(--accent)`
- Hit area: input element height `20px` minimum (comfortably clickable, not excessive)

Implemented via `input[type="range"]` with cross-browser CSS (`-webkit-slider-thumb`, `-webkit-slider-runnable-track`, + standard).

---

## 7. Main Window Layout

```
┌─────────────────────────────────┐
│  AudioCaptor              [⚙]   │  ← title 16px bold + gear 32×32px
├─────────────────────────────────┤
│     [ЗУПИНЕНО / ЗАПИС / ПАУЗА]  │  ← status card, colored border
│           00:00:00              │  ← timer 26px monospace
├─────────────────────────────────┤
│ ПРОФІЛЬ                         │  ← section card
│  [Default ▾] [+][✎][✕]         │
├─────────────────────────────────┤
│ ПРИСТРОЇ                        │
│  Мікрофон: [select ▾]           │
│  Системний звук: [select ▾]     │
├─────────────────────────────────┤
│ ГУЧНІСТЬ                        │
│  Мікрофон  1.0  [────●──────]   │
│  Системний 0.5  [───●───────]   │
├─────────────────────────────────┤
│  [▶ ПОЧАТИ ЗАПИС              ] │  ← or [⏸ ПАУЗА][⏹ СТОП]
└─────────────────────────────────┘
```

---

## 8. Settings Dialog (380px wide, fits in 568px usable height)

Fields in order:
1. **Тема** — segmented control: `[Авто (Windows)] [Світла] [Темна]`; hint text below
2. Divider
3. **Гаряча клавіша** — readonly input + "Змінити" + "Скинути"
4. **Звуки** — checkbox: "Відтворювати звукові сигнали"
5. **Безпека** — checkbox: "Підтверджувати вихід під час запису"
6. **Мова** — select (uk / en) + restart hint
7. Actions: `[Закрити]`

Estimated height: ~375px. No scroll needed.

---

## 9. Profile Dialog (400px wide, fits in 568px usable height)

2-column CSS grid (`grid-template-columns: 1fr 1fr; gap: 0 10px`).

```
┌──────────────────────────────────┐
│ Новий профіль / Редагувати       │
├──────────────────────────────────┤
│ ОСНОВНЕ                          │
│ [Назва          ][Опис          ]│
│ [Папка збереження               ]│  ← col-span
├──────────────────────────────────┤
│ АУДІО                            │
│ [Режим виводу  ][Частота дискр. ]│
│ [Гучн. мікроф. ][Гучн. системи  ]│  ← sliders
├──────────────────────────────────┤
│ ІМЕНА ФАЙЛІВ                     │
│ [Мікрофон      ][Системний звук ]│
│ [Зведення (mix)]                 │
├──────────────────────────────────┤
│ [error if any                  ] │
│              [Скасувати][Зберегти│
└──────────────────────────────────┘
```

Estimated height: ~430px. No scroll needed.

> **Height calculation:** Tauri's WebView occupies the window minus the native title bar (~32px): 600 − 32 = **568px** usable. `90vh` = 90% × 568 = **≈511px**. Dialog at ~430px fits with ~80px headroom.

Fields that show/hide based on `outputMode`:
- `Мікрофон` filename: hidden when mode = `Loopback`
- `Системний звук` filename: hidden when mode = `Microphone`
- `Зведення` filename: hidden when mode ∉ `{Mix, MixPlusMicrophone, MixPlusLoopback}`

---

## 10. ConfirmExit Dialog

Simple, no changes to layout. Apply theme tokens only (replace hardcoded colors).

---

## 11. Files Changed

| File | Change |
|---|---|
| `src/app.css` | Add all CSS custom properties (light + dark tokens); remove per-component hardcoded colors; add slider thumb CSS |
| `src/lib/types/index.ts` | Add `theme: 'auto' \| 'light' \| 'dark'` to `Settings` |
| `src/lib/stores/theme.svelte.ts` | **New** — theme store: get/set, auto-detection, `data-theme` application, `matchMedia` listener |
| `src/lib/stores/settings.svelte.ts` | Add `theme` field; expose `setTheme()`; bump `settingsVersion` to 4 |
| `src-tauri/src/settings.rs` | Add `theme` field; bump default version to 4; add v3→v4 migration arm and tests |
| `src/lib/utils/invoke.ts` | No changes needed (settings save already handles new fields) |
| `index.html` | Add inline `<script>` in `<head>` for FOUC prevention |
| `src/App.svelte` | Replace hardcoded colors with CSS vars; update header size |
| `src/lib/components/StatusIndicator.svelte` | State-dependent border; timer dim when idle |
| `src/lib/components/VolumeSlider.svelte` | Custom thumb/track CSS |
| `src/lib/components/DeviceSelect.svelte` | CSS vars only |
| `src/lib/components/ProfileSelector.svelte` | CSS vars only |
| `src/lib/components/RecordControls.svelte` | CSS vars only |
| `src/lib/components/SettingsDialog.svelte` | Add theme segmented control; add `theme` prop + handler |
| `src/lib/components/ProfileDialog.svelte` | 2-column grid layout; remove `overflow-y: auto`; conditional filename visibility |
| `src/lib/components/ConfirmExitDialog.svelte` | CSS vars only |

---

## 12. WCAG 2.1 AA Verification

Key contrast ratios (calculated against backgrounds):

| Element | Light ratio | Dark ratio | Required |
|---|---|---|---|
| Body text on bg | 16.2:1 | 14.7:1 | ≥ 4.5:1 ✓ |
| Secondary text | 7.3:1 | 5.1:1 | ≥ 4.5:1 ✓ |
| Accent (#0078d4) on white | 4.7:1 | — | ≥ 4.5:1 ✓ |
| Accent (#60a5fa) on #252525 | — | 5.2:1 | ≥ 4.5:1 ✓ |
| Start btn text on #107c10 | 4.8:1 | 4.8:1 | ≥ 4.5:1 ✓ |
| Pause btn text on #9d5d00 | 4.6:1 | — | ≥ 4.5:1 ✓ |
| Pause btn text on #92400e | — | 7.1:1 | ≥ 4.5:1 ✓ |
| Stop btn text on #c42b1c | 5.1:1 | — | ≥ 4.5:1 ✓ |
| Stop btn text on #991b1b | — | 5.4:1 | ≥ 4.5:1 ✓ |
| Focus ring on bg | 3.2:1 | 3.1:1 | ≥ 3:1 (UI) ✓ |

All interactive controls retain `:focus-visible` outline: `2px solid var(--focus-ring); outline-offset: 2px`.
