<script lang="ts">
  import { tick } from "svelte";
  import type { Theme } from "../types";
  import * as m from "../../paraglide/messages";
  import { updateHotkey, updateSoundEnabled, setLanguage, setConfirmExitDuringRecording } from "../stores/settings.svelte";
  import * as api from "../utils/invoke";

  interface Props {
    open: boolean;
    hotkey: string | null;
    soundEnabled: boolean;
    confirmExitDuringRecording: boolean;
    minimizeToTrayOnFocusLoss: boolean;
    language: "en" | "uk";
    theme: Theme;
    onclose: () => void;
    onsave: (patch: Partial<import("../types").Settings>) => void;
    onthemechange: (t: Theme) => void;
    onminimizetotraytoggle: (v: boolean) => void;
  }

  let {
    open,
    hotkey,
    soundEnabled,
    confirmExitDuringRecording,
    minimizeToTrayOnFocusLoss,
    language,
    theme,
    onclose,
    onsave,
    onthemechange,
    onminimizetotraytoggle,
  }: Props = $props();

  const themeOptions: { value: Theme; label: () => string }[] = [
    { value: "auto",  label: () => m.settings_theme_auto() },
    { value: "light", label: () => m.settings_theme_light() },
    { value: "dark",  label: () => m.settings_theme_dark() },
  ];

  let capturingHotkey = $state(false);
  let localHotkey = $state<string | null>(null);
  $effect.pre(() => { localHotkey = hotkey; });

  async function startHotkeyCapture() {
    capturingHotkey = true;
    await api.unregisterHotkey();

    async function onKeyDown(e: KeyboardEvent) {
      e.preventDefault();
      e.stopPropagation();
      if (["Control", "Shift", "Alt", "Meta"].includes(e.key)) return;

      const parts: string[] = [];
      if (e.ctrlKey)  parts.push("Ctrl");
      if (e.altKey)   parts.push("Alt");
      if (e.shiftKey) parts.push("Shift");
      let key = e.key;
      if (key === " ")         key = "Space";
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
      await tick();
      document.getElementById("settings-hotkey-display")?.focus();
    }

    window.addEventListener("keydown", onKeyDown, true);
  }

  async function cancelCapture() {
    capturingHotkey = false;
    if (localHotkey === null) {
      await api.unregisterHotkey();
    } else {
      await api.setHotkey(localHotkey);
    }
  }

  async function resetHotkey() {
    localHotkey = null;
    await updateHotkey(null);
    onsave({ hotkey: null });
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
    if (e.key === "Escape" &&  capturingHotkey) { void cancelCapture(); return; }
    if (e.key === "Enter"  && !capturingHotkey) { e.preventDefault(); onclose(); return; }
    if (e.key === "Tab") {
      const dialog = e.currentTarget as HTMLElement;
      const focusable = dialog.querySelectorAll<HTMLElement>(
        'input, select, button, [tabindex]:not([tabindex="-1"])'
      );
      if (!focusable.length) return;
      const first = focusable[0];
      const last  = focusable[focusable.length - 1];
      if (e.shiftKey && document.activeElement === first) { e.preventDefault(); last.focus(); }
      else if (!e.shiftKey && document.activeElement === last)  { e.preventDefault(); first.focus(); }
    }
  }

  $effect(() => {
    if (open) requestAnimationFrame(() => document.getElementById("settings-hotkey-display")?.focus());
  });
</script>

{#if open}
  <div class="dialog-backdrop">
    <div
      class="dialog"
      role="dialog"
      aria-modal="true"
      aria-labelledby="settings-dialog-title"
      tabindex="-1"
      onkeydown={handleKeydown}
    >
      <h2 id="settings-dialog-title">{m.settings_dialog_title()}</h2>

      <!-- Theme -->
      <div class="field">
        <span class="field-label">{m.settings_theme_label()}</span>
        <div class="theme-seg" role="group" aria-label={m.settings_theme_label()}>
          {#each themeOptions as opt}
            <button
              type="button"
              class="theme-btn"
              class:active={theme === opt.value}
              onclick={() => onthemechange(opt.value)}
              aria-pressed={theme === opt.value}
            >{opt.label()}</button>
          {/each}
        </div>
        <p class="hint">{m.settings_theme_hint()}</p>
      </div>

      <div class="divider"></div>

      <!-- Hotkey -->
      <div class="field">
        <label for="settings-hotkey-display" class="field-label">{m.settings_hotkey_label()}</label>
        <div class="hotkey-row">
          <input
            id="settings-hotkey-display"
            type="text"
            value={localHotkey ?? ""}
            placeholder={m.settings_hotkey_none()}
            readonly
            aria-label={localHotkey
              ? m.settings_hotkey_current_aria({ hotkey: localHotkey })
              : m.settings_hotkey_none()}
            class="hotkey-input"
          />
          <button type="button" class="btn-sm"
            aria-label={capturingHotkey ? m.settings_hotkey_capturing() : m.settings_hotkey_capture_btn_aria()}
            onclick={startHotkeyCapture} disabled={capturingHotkey}>
            {capturingHotkey ? m.settings_hotkey_capturing() : m.settings_hotkey_capture_btn()}
          </button>
          <button type="button" class="btn-sm btn-secondary"
            aria-label={m.settings_hotkey_reset_btn_aria()}
            onclick={resetHotkey} disabled={capturingHotkey}>
            {m.settings_hotkey_reset_btn()}
          </button>
        </div>
      </div>

      <!-- Sound -->
      <div class="field">
        <label class="checkbox-label">
          <input type="checkbox" checked={soundEnabled} onchange={handleSoundToggle} />
          {m.settings_sound_label()}
        </label>
      </div>

      <!-- Confirm exit -->
      <div class="field">
        <label class="checkbox-label">
          <input type="checkbox" checked={confirmExitDuringRecording} onchange={handleConfirmExitToggle} />
          {m.settings_confirm_exit_label()}
        </label>
      </div>

      <!-- Minimize to tray on focus loss -->
      <div class="field">
        <label class="checkbox-label">
          <input type="checkbox" checked={minimizeToTrayOnFocusLoss}
                 onchange={(e) => onminimizetotraytoggle((e.target as HTMLInputElement).checked)} />
          {m.settings_minimize_on_focus_loss()}
        </label>
      </div>

      <!-- Language -->
      <div class="field">
        <label for="settings-language" class="field-label">{m.settings_language_label()}</label>
        <select id="settings-language" value={language} onchange={handleLanguageChange}>
          <option value="en">{m.settings_language_en()}</option>
          <option value="uk">{m.settings_language_uk()}</option>
        </select>
        <p class="hint">{m.settings_language_restart_note()}</p>
      </div>

      <div class="actions">
        <button
          type="button"
          class="btn-primary"
          onclick={async () => { if (capturingHotkey) await cancelCapture(); onclose(); }}
        >{m.btn_close()}</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .dialog-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
  }
  .dialog {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 10px;
    padding: 22px 22px 20px;
    width: 380px;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
    color: var(--text-primary);
  }
  h2 { margin: 0 0 16px; font-size: 1.1rem; color: var(--text-primary); }

  .field { display: flex; flex-direction: column; gap: 5px; margin-bottom: 14px; }
  .field:last-of-type { margin-bottom: 0; }
  .field-label { font-weight: 700; font-size: 0.8rem; color: var(--text-secondary); }

  /* Theme segmented control */
  .theme-seg { display: flex; border-radius: 7px; overflow: hidden; border: 1px solid var(--border); }
  .theme-btn {
    flex: 1; padding: 6px 4px; text-align: center;
    font-size: 0.8rem; font-weight: 600; cursor: pointer;
    border: none; border-right: 1px solid var(--border);
    background: var(--surface); color: var(--text-secondary);
    transition: background 0.1s, color 0.1s;
  }
  .theme-btn:last-child { border-right: none; }
  .theme-btn:hover:not(.active) { background: var(--surface-hover); }
  .theme-btn.active { background: var(--accent); color: #ffffff; }

  .divider { height: 1px; background: var(--border); margin: 2px 0 14px; }

  select {
    padding: 7px 8px; border: 1px solid var(--border); border-radius: 5px;
    font-size: 0.875rem; background: var(--surface); color: var(--text-primary);
  }
  .hotkey-row { display: flex; gap: 6px; align-items: center; }
  .hotkey-input {
    flex: 1; padding: 7px 8px; border: 1px solid var(--border); border-radius: 5px;
    font-size: 0.875rem; background: var(--surface-hover); color: var(--text-primary);
    cursor: default;
  }
  .checkbox-label {
    display: flex; align-items: center; gap: 8px;
    cursor: pointer; font-size: 0.875rem; font-weight: normal;
    color: var(--text-primary);
  }
  .hint { margin: 0; font-size: 0.75rem; color: var(--text-muted); }

  .btn-sm {
    padding: 6px 10px; border: 1px solid var(--border); border-radius: 5px;
    font-size: 0.8rem; font-weight: 600; cursor: pointer;
    background: var(--surface-hover); color: var(--text-primary); white-space: nowrap;
  }
  .btn-sm:hover:not(:disabled) { background: var(--border); }
  .btn-sm:disabled { opacity: 0.5; cursor: not-allowed; }
  .btn-sm.btn-secondary { background: var(--surface); }

  .actions { display: flex; justify-content: flex-end; margin-top: 16px; }
  .btn-primary {
    padding: 8px 18px; border: none; border-radius: 6px;
    font-size: 0.875rem; font-weight: 700; cursor: pointer;
    background: var(--accent); color: #ffffff;
  }
  .btn-primary:hover { background: var(--accent-hover); }
</style>
