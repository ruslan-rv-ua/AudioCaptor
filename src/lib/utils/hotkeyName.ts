// Names a pressed key for the global hotkey from the physical key
// (`KeyboardEvent.code`), so that the name does not depend on the keyboard
// layout: the Q key is "Q" under the Ukrainian layout too (where it types Й),
// and Shift+1 stays "1" (not "!"). Every name below is one that the shortcut
// parser in src-tauri/src/hotkey.rs accepts; its tests list samples of them.

/// Keys whose `code` is already the name the parser takes and NVDA reads well.
const SAME_NAME = new Set([
  "Space", "Enter", "Tab", "Backspace", "Delete", "Insert",
  "Home", "End", "PageUp", "PageDown",
  "ArrowUp", "ArrowDown", "ArrowLeft", "ArrowRight",
  "Pause", "PrintScreen", "ScrollLock", "NumLock", "CapsLock",
  // Punctuation keys go by word, not by symbol: NVDA skips many symbols at
  // its default punctuation level, and the symbol changes with the layout.
  "Minus", "Equal", "BracketLeft", "BracketRight", "Backslash",
  "Semicolon", "Quote", "Comma", "Period", "Slash", "Backquote",
  "NumpadAdd", "NumpadSubtract", "NumpadMultiply", "NumpadDivide",
  "NumpadEnter", "NumpadEqual",
  "AudioVolumeUp", "AudioVolumeDown", "AudioVolumeMute",
  "MediaPlayPause", "MediaStop", "MediaTrackNext", "MediaTrackPrevious",
]);

/// Keys a numeric keypad key stands for while Num Lock is off.
const NUMPAD_OFF = new Set([
  "Home", "End", "PageUp", "PageDown", "Insert", "Delete",
  "ArrowUp", "ArrowDown", "ArrowLeft", "ArrowRight",
]);

/// The hotkey name of the key (without modifiers), or `null` when the parser
/// has no code for it (such as the Menu key, `ContextMenu`).
export function hotkeyKeyName(code: string, key: string): string | null {
  // No physical key (input made up by a program without a scan code): name
  // what the key types, if that is a Latin letter, a digit or a named key.
  if (code === "") {
    if (key === " ") return "Space";
    if (/^[a-zA-Z0-9]$/.test(key)) return key.toUpperCase();
    return SAME_NAME.has(key) || /^F([1-9]|1[0-9]|2[0-4])$/.test(key) ? key : null;
  }
  let m = /^Key([A-Z])$/.exec(code);
  if (m) return m[1];
  m = /^Digit([0-9])$/.exec(code);
  if (m) return m[1];
  if (/^F([1-9]|1[0-9]|2[0-4])$/.test(code)) return code;
  if (/^Numpad([0-9]|Decimal)$/.test(code)) {
    // With Num Lock off the key is an arrow, Home and the like: name what it does.
    if (/^[0-9.,]$/.test(key)) return code;
    return NUMPAD_OFF.has(key) ? key : null;
  }
  return SAME_NAME.has(code) ? code : null;
}

/// The name of a key that cannot be a hotkey, for the refusal message.
export function refusedKeyName(code: string, key: string): string {
  return code || (key === " " ? "Space" : key);
}
