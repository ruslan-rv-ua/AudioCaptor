# Design: SeparateFiles Recording Mode

**Date:** 2026-04-07  
**Status:** Approved  

## Summary

Add a new recording output mode `SeparateFiles` that captures microphone and system audio (loopback) into two independent WAV files simultaneously, with no mixing. The mode appears in the profile dialog output mode combobox after "System audio only" (Loopback).

---

## UI Label

| Locale | Label |
|--------|-------|
| EN | `Separate: Mic + System audio (2 files)` |
| UK | `Окремо: Мікрофон + Системний звук (2 файли)` |

Position in combobox (after change):
1. Microphone only
2. System audio only
3. **Separate: Mic + System audio (2 files)** ← new
4. Mix (Mic + System)
5. Mix + Microphone (2 files)
6. Mix + System audio (2 files)

---

## Section 1: Types

### Rust — `src-tauri/src/audio/types.rs`

Add `SeparateFiles` variant to `OutputMode` enum:

```rust
pub enum OutputMode {
    Microphone,
    Loopback,
    Mix,
    MixPlusMicrophone,
    MixPlusLoopback,
    SeparateFiles,  // new
}
```

Serde `PascalCase` serializes as `"SeparateFiles"`, matching the TypeScript side automatically.

### TypeScript — `src/lib/types/index.ts`

```ts
export type OutputMode =
  | "Microphone"
  | "Loopback"
  | "Mix"
  | "MixPlusMicrophone"
  | "MixPlusLoopback"
  | "SeparateFiles";  // new
```

---

## Section 2: Localization

### `messages/en.json`

Add after `"mode_loopback"`:
```json
"mode_separate_files": "Separate: Mic + System audio (2 files)"
```

### `messages/uk.json`

Add after `"mode_loopback"`:
```json
"mode_separate_files": "Окремо: Мікрофон + Системний звук (2 файли)"
```

### `src/paraglide/messages/mode_separate_files.js`

New file following the existing paraglide message pattern (see `mode_mix_plus_loopback.js` as template).

### `src/paraglide/messages/_index.js`

Add export after `mode_loopback` line:
```js
export * from './mode_separate_files.js'
```

---

## Section 3: Frontend

### `src/lib/components/ProfileDialog.svelte`

**outputModes array** — insert after Loopback entry:
```ts
const outputModes: { value: OutputMode; label: string }[] = [
  { value: "Microphone",        label: m.mode_microphone() },
  { value: "Loopback",          label: m.mode_loopback() },
  { value: "SeparateFiles",     label: m.mode_separate_files() },  // new
  { value: "Mix",               label: m.mode_mix() },
  { value: "MixPlusMicrophone", label: m.mode_mix_plus_mic() },
  { value: "MixPlusLoopback",   label: m.mode_mix_plus_loopback() },
];
```

**Filename visibility derived values** — `showMicFilename` and `showLoopbackFilename` are already correct as-is (both show for `SeparateFiles` since it is neither `"Loopback"` nor `"Microphone"`). `showMixFilename` is already an explicit allowlist that excludes `SeparateFiles` — no change needed.

### `src/lib/stores/recording.svelte.ts`

`SeparateFiles` requires both mic and loopback. Update all 4 locations that check `needsMic`/`needsLoopback`. The four locations have slightly different shapes:

- **`canRecord`** (lines ~34–36) — inline `const needsMic` and `const needsLoopback` locals: append `|| mode === "SeparateFiles"` to each.
- **`needsMic` getter** (lines ~40–42) — the condition before `&& !selectedMic`: append `|| outputMode === "SeparateFiles"`.
- **`needsLoopback` getter** (lines ~43–47) — the condition before `&& !selectedLoopback`: append `|| outputMode === "SeparateFiles"`.
- **`readinessHint`** (lines ~48–56) — inline `const needsMic` and `const needsLoopback` locals: append `|| mode === "SeparateFiles"` to each.

Example for the `canRecord` inline locals (no `&& !selected*` suffix — these are pure boolean flags):
```ts
const needsMic = mode === "Microphone" || mode === "Mix"
  || mode === "MixPlusMicrophone" || mode === "MixPlusLoopback"
  || mode === "SeparateFiles";
const needsLoopback = mode === "Loopback" || mode === "Mix"
  || mode === "MixPlusMicrophone" || mode === "MixPlusLoopback"
  || mode === "SeparateFiles";
```

Example for the `readinessHint` inline locals (these **include** `&& !selected*` — they mean "mode needs device AND device is missing"):
```ts
const needsMic = (mode === "Microphone" || mode === "Mix"
  || mode === "MixPlusMicrophone" || mode === "MixPlusLoopback"
  || mode === "SeparateFiles") && !selectedMic;
const needsLoopback = (mode === "Loopback" || mode === "Mix"
  || mode === "MixPlusMicrophone" || mode === "MixPlusLoopback"
  || mode === "SeparateFiles") && !selectedLoopback;
```

Example for the `needsMic` getter body (keeps the `&& !selectedMic` suffix):
```ts
return (outputMode === "Microphone" || outputMode === "Mix"
  || outputMode === "MixPlusMicrophone" || outputMode === "MixPlusLoopback"
  || outputMode === "SeparateFiles")
  && !selectedMic;
```

---

## Section 4: Rust Backend

### `src-tauri/src/lib.rs`

**1. Device validation (2 match arms)**

Both existing arms use `_ => {}` wildcards. `SeparateFiles` would silently skip validation via the wildcard — add it explicitly to the correct arm in each match. **The `_ => {}` wildcard is retained** after the change; it now covers only the one remaining single-source mode (see below).

Add `SeparateFiles` to the mic-required arm. The `_ => {}` arm is retained and covers only `Loopback` (the only mode that does not need a mic):
```rust
OutputMode::Microphone | OutputMode::Mix
| OutputMode::MixPlusMicrophone | OutputMode::MixPlusLoopback
| OutputMode::SeparateFiles => {
    if mic_id.is_none() {
        return Err("DEVICE_NOT_FOUND: Microphone device required for this mode".into());
    }
}
_ => {}   // covers Loopback only
```

Add `SeparateFiles` to the loopback-required arm. The `_ => {}` arm is retained and covers only `Microphone` (the only mode that does not need loopback):
```rust
OutputMode::Loopback | OutputMode::Mix
| OutputMode::MixPlusMicrophone | OutputMode::MixPlusLoopback
| OutputMode::SeparateFiles => {
    if loopback_id.is_none() {
        return Err("DEVICE_NOT_FOUND: Loopback device required for this mode".into());
    }
}
_ => {}   // covers Microphone only
```

**2. Capture thread startup (2 match arms)**

Same wildcard issue. Both capture match blocks (`mic_id` and `loopback_id`) use `_ => (None, None)`. Without explicit `SeparateFiles` arms, no capture threads would start. Add `SeparateFiles` to the explicit arm in each match. **The `_ => (None, None)` wildcard is retained** and covers only the one opposite single-source mode.

Mic capture match — add `| OutputMode::SeparateFiles` (wildcard then covers only `Loopback`):
```rust
OutputMode::Microphone | OutputMode::Mix
| OutputMode::MixPlusMicrophone | OutputMode::MixPlusLoopback
| OutputMode::SeparateFiles => {
    let (handle, consumer) = capture::start_mic_capture(id).map_err(|e| e.to_string())?;
    (Some(handle), Some(consumer))
}
_ => (None, None)   // covers Loopback only
```

Loopback capture match — add `| OutputMode::SeparateFiles` (wildcard then covers only `Microphone`):
```rust
OutputMode::Loopback | OutputMode::Mix
| OutputMode::MixPlusMicrophone | OutputMode::MixPlusLoopback
| OutputMode::SeparateFiles => {
    let (handle, consumer) = capture::start_loopback_capture(id).map_err(|e| e.to_string())?;
    (Some(handle), Some(consumer))
}
_ => (None, None)   // covers Microphone only
```

**3. WAV writer creation**

Add new match arm:
```rust
OutputMode::SeparateFiles => {
    let mic_path  = output_dir.join(format!("{}_{}.wav", profile.mic_filename, timestamp));
    let loop_path = output_dir.join(format!("{}_{}.wav", profile.loopback_filename, timestamp));
    let w1 = Box::new(WavOutputWriter::new(mic_path, sample_rate, channels).map_err(|e| e.to_string())?);
    let w2 = Box::new(WavOutputWriter::new(loop_path, sample_rate, channels).map_err(|e| e.to_string())?);
    (w1, Some(w2))
}
```

Files produced: `<mic_filename>_<timestamp>.wav` and `<loopback_filename>_<timestamp>.wav`.

### `src-tauri/src/audio/mixer.rs`

**Approach:** Synchronized — both sources must accumulate data before writing (same as Mix modes). Primary writer receives mic-only, secondary writer receives loopback-only.

Also update the `MixerConfig.secondary_writer` doc comment to include `SeparateFiles`:
```rust
/// Secondary writer for parallel modes (MixPlusMicrophone, MixPlusLoopback, SeparateFiles).
```

**1. Sync guard — `matches!` for "wait for both sources"**

Add `SeparateFiles` to the existing check:
```rust
if matches!(config.output_mode,
    OutputMode::Mix | OutputMode::MixPlusMicrophone | OutputMode::MixPlusLoopback
    | OutputMode::SeparateFiles)
```

**2. Drain frames — second `matches!`**

Same addition in the drain-frames block. Also update the `// In Mix mode:` comment on that block to reflect that `SeparateFiles` is now included:
```rust
// In Mix / SeparateFiles mode: drain equal frame counts from both sources to keep them in sync.
```

**3. Output match — new arm**

```rust
OutputMode::SeparateFiles => {
    // Write loopback to secondary writer
    let loop_only: Vec<f32> = loop_processed.iter()
        .map(|&s| (s * loopback_volume).clamp(-1.0, 1.0))
        .collect();
    if !loop_only.is_empty() {
        if let Some(ref mut sw) = config.secondary_writer {
            if let Err(e) = sw.write_samples(&loop_only) {
                log::error!("Secondary WAV write error: {}", e);
            }
        }
    }
    // Return mic for primary writer
    mic_processed.iter()
        .map(|&s| (s * mic_volume).clamp(-1.0, 1.0))
        .collect()
}
```

Primary writer receives mic samples; secondary writer receives loopback samples. Both finalized on stop (existing `finalize` logic already handles `secondary_writer`).

---

## Files Changed

| File | Change |
|------|--------|
| `src-tauri/src/audio/types.rs` | Add `SeparateFiles` variant + 2 serialization/roundtrip tests |
| `src-tauri/src/lib.rs` | Device validation (×2 arms), capture startup (×2 arms), WAV writer arm |
| `src-tauri/src/audio/mixer.rs` | Doc comment, sync guards (×2), output match arm |
| `src/lib/types/index.ts` | Add `"SeparateFiles"` to union type |
| `src/lib/stores/recording.svelte.ts` | Add `SeparateFiles` to 4 mode checks |
| `src/lib/components/ProfileDialog.svelte` | Add mode to combobox array |
| `messages/en.json` | Add `mode_separate_files` key |
| `messages/uk.json` | Add `mode_separate_files` key |
| `src/paraglide/messages/mode_separate_files.js` | New paraglide message file |
| `src/paraglide/messages/_index.js` | Export new message file |

## Tests

Add to `src-tauri/src/audio/types.rs` (following existing `MixPlusMicrophone`/`MixPlusLoopback` test pattern):

```rust
#[test]
fn output_mode_separate_files_serializes() {
    let json = serde_json::to_string(&OutputMode::SeparateFiles).unwrap();
    assert_eq!(json, "\"SeparateFiles\"");
}

#[test]
fn output_mode_separate_files_roundtrip() {
    let mode = OutputMode::SeparateFiles;
    let json = serde_json::to_string(&mode).unwrap();
    let deserialized: OutputMode = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized, OutputMode::SeparateFiles);
}
```
