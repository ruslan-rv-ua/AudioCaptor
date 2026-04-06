<script lang="ts">
  import * as m from "../../paraglide/messages";
  import { updateHotkey, updateSoundEnabled, setLanguage, setConfirmExitDuringRecording } from "../stores/settings.svelte";
  import * as api from "../utils/invoke";

  interface Props {
    open: boolean;
    hotkey: string;
    soundEnabled: boolean;
    confirmExitDuringRecording: boolean;
    language: "en" | "uk";
    onclose: () => void;
    onsave: (patch: Partial<import("../types").Settings>) => void;
  }

  let {
    open,
    hotkey,
    soundEnabled,
    confirmExitDuringRecording,
    language,
    onclose,
    onsave,
  }: Props = $props();

  let capturingHotkey = $state(false);
  // $effect.pre runs before the first DOM paint so there is no visible flash;
  // it also keeps localHotkey in sync whenever the hotkey prop changes.
  let localHotkey = $state('');
  $effect.pre(() => { localHotkey = hotkey; });

  async function startHotkeyCapture() {
    capturingHotkey = true;
    await api.unregisterHotkey();

    async function onKeyDown(e: KeyboardEvent) {
      e.preventDefault();
      e.stopPropagation();

      if (["Control", "Shift", "Alt", "Meta"].includes(e.key)) return;

      const parts: string[] = [];
      if (e.ctrlKey) parts.push("Ctrl");
      if (e.altKey) parts.push("Alt");
      if (e.shiftKey) parts.push("Shift");
      let key = e.key;
      if (key === " ") key = "Space";
      else if (key.length === 1) key = key.toUpperCase();
      else if (key === "Escape") {
        window.removeEventListener("keydown", onKeyDown, true);
        await cancelCapture();
        return;
      }
      parts.push(key);
      const shortcut = parts.join("+");
      localHotkey = shortcut;
      await updateHotkey(shortcut);
      onsave({ hotkey: shortcut });
      capturingHotkey = false;
      window.removeEventListener("keydown", onKeyDown, true);
    }

    window.addEventListener("keydown", onKeyDown, true);
  }

  async function cancelCapture() {
    capturingHotkey = false;
    await api.setHotkey(localHotkey);
  }

  async function resetHotkey() {
    localHotkey = "Pause";
    await updateHotkey("Pause");
    onsave({ hotkey: "Pause" });
  }

  function handleSoundToggle(e: Event) {
    const checked = (e.target as HTMLInputElement).checked;
    updateSoundEnabled(checked);
    onsave({ soundEnabled: checked });
  }

  function handleConfirmExitToggle(e: Event) {
    const checked = (e.target as HTMLInputElement).checked;
    setConfirmExitDuringRecording(checked);
    onsave({ confirmExitDuringRecording: checked });
  }

  function handleLanguageChange(e: Event) {
    const lang = (e.target as HTMLSelectElement).value as "en" | "uk";
    setLanguage(lang);
    onsave({ language: lang });
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape" && !capturingHotkey) { onclose(); return; }
    if (e.key === "Escape" && capturingHotkey) { void cancelCapture(); return; }
    if (e.key === "Tab") {
      const dialog = (e.currentTarget as HTMLElement);
      const focusable = dialog.querySelectorAll<HTMLElement>(
        'input, select, button, [tabindex]:not([tabindex="-1"])'
      );
      if (!focusable.length) return;
      const first = focusable[0];
      const last = focusable[focusable.length - 1];
      if (e.shiftKey && document.activeElement === first) { e.preventDefault(); last.focus(); }
      else if (!e.shiftKey && document.activeElement === last) { e.preventDefault(); first.focus(); }
    }
  }

  $effect(() => {
    if (open) requestAnimationFrame(() => document.getElementById("settings-hotkey-display")?.focus());
  });
</script>

{#if open}
  <div class="dialog-backdrop">
    <div class="dialog" role="dialog" aria-modal="true" aria-labelledby="settings-dialog-title" tabindex="-1" onkeydown={handleKeydown}>
      <h2 id="settings-dialog-title">{m.settings_dialog_title()}</h2>

      <!-- Hotkey section -->
      <div class="field">
        <label for="settings-hotkey-display">{m.settings_hotkey_label()}</label>
        <div class="hotkey-row">
          <input
            id="settings-hotkey-display"
            type="text"
            value={localHotkey}
            readonly
            aria-label={m.settings_hotkey_current_aria({ hotkey: localHotkey })}
            class="hotkey-input"
          />
          <button
            type="button"
            class="btn-small"
            onclick={startHotkeyCapture}
            disabled={capturingHotkey}
          >
            {capturingHotkey ? m.settings_hotkey_capturing() : m.settings_hotkey_capture_btn()}
          </button>
          <button
            type="button"
            class="btn-small btn-secondary"
            onclick={resetHotkey}
            disabled={capturingHotkey}
          >
            {m.settings_hotkey_reset_btn()}
          </button>
        </div>
      </div>

      <!-- Sound toggle -->
      <div class="field">
        <label class="checkbox-label">
          <input
            type="checkbox"
            checked={soundEnabled}
            onchange={handleSoundToggle}
          />
          {m.settings_sound_label()}
        </label>
      </div>

      <!-- Confirm exit toggle -->
      <div class="field">
        <label class="checkbox-label">
          <input
            type="checkbox"
            checked={confirmExitDuringRecording}
            onchange={handleConfirmExitToggle}
          />
          {m.settings_confirm_exit_label()}
        </label>
      </div>

      <!-- Language selector -->
      <div class="field">
        <label for="settings-language">{m.settings_language_label()}</label>
        <select id="settings-language" value={language} onchange={handleLanguageChange}>
          <option value="en">{m.settings_language_en()}</option>
          <option value="uk">{m.settings_language_uk()}</option>
        </select>
        <p class="hint">{m.settings_language_restart_note()}</p>
      </div>

      <div class="actions">
        <button type="button" class="btn-primary" onclick={async () => { if (capturingHotkey) await cancelCapture(); onclose(); }}>{m.btn_close()}</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .dialog-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.4);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
  }
  .dialog {
    background: white;
    border-radius: 8px;
    padding: 24px;
    width: 380px;
    box-shadow: 0 4px 24px rgba(0,0,0,0.2);
  }
  h2 { margin: 0 0 16px; font-size: 1.2rem; }
  .field { display: flex; flex-direction: column; gap: 4px; margin-bottom: 16px; }
  .field label { font-weight: 600; font-size: 0.875rem; }
  .field select { padding: 8px; border: 1px solid #ccc; border-radius: 4px; font-size: 0.875rem; }
  .hotkey-row { display: flex; gap: 8px; align-items: center; }
  .hotkey-input { flex: 1; padding: 8px; border: 1px solid #ccc; border-radius: 4px; font-size: 0.875rem; background: #f5f5f5; cursor: default; }
  .checkbox-label { display: flex; align-items: center; gap: 8px; cursor: pointer; font-size: 0.875rem; font-weight: normal; }
  .hint { margin: 4px 0 0; font-size: 0.8rem; color: #6b7280; }
  .btn-small { padding: 8px 10px; border: none; border-radius: 4px; font-size: 0.8rem; font-weight: 600; cursor: pointer; background: #6b7280; color: white; white-space: nowrap; }
  .btn-small:hover:not(:disabled) { background: #4b5563; }
  .btn-small:disabled { opacity: 0.6; cursor: not-allowed; }
  .btn-small.btn-secondary { background: #e5e7eb; color: #374151; }
  .btn-small.btn-secondary:hover:not(:disabled) { background: #d1d5db; }
  .actions { display: flex; justify-content: flex-end; margin-top: 8px; }
  .btn-primary { padding: 8px 16px; border: none; border-radius: 4px; font-size: 0.875rem; font-weight: 600; cursor: pointer; background: #2563eb; color: white; }
  .btn-primary:hover { background: #1d4ed8; }
</style>
