<script lang="ts">
  import type { RecordingProfile, OutputMode } from "../types";
  import * as m from "../../paraglide/messages";
  import { open as openDialog } from "@tauri-apps/plugin-dialog";
  import { getRecordingsDir } from "../utils/invoke";

  interface Props {
    profile: RecordingProfile | null;
    open: boolean;
    onclose: () => void;
    onsave: (profile: RecordingProfile) => void;
  }

  let { profile, open, onclose, onsave }: Props = $props();

  let name         = $state("");
  let description  = $state("");
  let outputFolder = $state("Recordings");
  let outputMode   = $state<OutputMode>("Mix");
  let sampleRate   = $state(48000);
  let micVolume    = $state(1.0);
  let loopbackVolume = $state(0.5);
  let micFilename      = $state("mic");
  let loopbackFilename = $state("loopback");
  let mixFilename      = $state("mix");
  let error  = $state("");
  let editId = $state<string | null>(null);
  let folderInputEl = $state<HTMLInputElement | null>(null);

  async function browseFolder() {
    let defaultPath: string | undefined;
    try { defaultPath = await getRecordingsDir(); } catch { /* ignore */ }
    const selected = await openDialog({
      directory: true,
      multiple: false,
      defaultPath,
    });
    if (selected && typeof selected === "string") {
      outputFolder = selected;
      requestAnimationFrame(() => folderInputEl?.focus());
    }
  }

  const outputModes: { value: OutputMode; label: string }[] = [
    { value: "Microphone",        label: m.mode_microphone() },
    { value: "Loopback",          label: m.mode_loopback() },
    { value: "SeparateFiles",     label: m.mode_separate_files() },
    { value: "Mix",               label: m.mode_mix() },
    { value: "MixPlusMicrophone", label: m.mode_mix_plus_mic() },
    { value: "MixPlusLoopback",   label: m.mode_mix_plus_loopback() },
  ];
  const sampleRates = [8000, 16000, 44100, 48000];

  // Filename visibility depends on which streams are produced
  let showMicFilename = $derived(outputMode !== "Loopback");
  let showLoopbackFilename = $derived(outputMode !== "Microphone");
  let showMixFilename = $derived(
    outputMode === "Mix" || outputMode === "MixPlusMicrophone" || outputMode === "MixPlusLoopback"
  );

  let micFillPct      = $derived(((micVolume     - 0) / (4 - 0)) * 100);
  let loopbackFillPct = $derived(((loopbackVolume - 0) / (4 - 0)) * 100);

  $effect(() => {
    if (open && profile) {
      editId           = profile.id;
      name             = profile.name;
      description      = profile.description;
      outputFolder     = profile.outputFolder;
      outputMode       = profile.outputMode;
      sampleRate       = profile.sampleRate;
      micVolume        = profile.micVolume;
      loopbackVolume   = profile.loopbackVolume;
      micFilename      = profile.micFilename;
      loopbackFilename = profile.loopbackFilename;
      mixFilename      = profile.mixFilename;
    } else if (open && !profile) {
      editId = null; name = ""; description = ""; outputFolder = "Recordings";
      outputMode = "Mix"; sampleRate = 48000; micVolume = 1.0; loopbackVolume = 0.5;
      micFilename = "mic"; loopbackFilename = "loopback"; mixFilename = "mix";
    }
    error = "";
  });

  let isEdit = $derived(editId !== null);
  let title  = $derived(isEdit ? m.profile_dialog_edit_title() : m.profile_dialog_create_title());

  function handleSubmit() {
    if (!name.trim()) { error = m.error_profile_name_required(); return; }
    const forbidden = /[<>:"/\\|?*]/;
    const filenames = [
      ...(showMicFilename      ? [micFilename]      : []),
      ...(showLoopbackFilename ? [loopbackFilename]  : []),
      ...(showMixFilename      ? [mixFilename]       : []),
    ];
    for (const fn of filenames) {
      if (!fn.trim() || forbidden.test(fn)) { error = m.error_filename_invalid(); return; }
    }
    onsave({
      id: editId ?? crypto.randomUUID(),
      name: name.trim(), description: description.trim(),
      outputFolder, outputMode, sampleRate,
      micVolume, loopbackVolume,
      micFilename: micFilename.trim(),
      loopbackFilename: loopbackFilename.trim(),
      mixFilename: mixFilename.trim(),
    });
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") { onclose(); return; }
    if (e.key === "Tab") {
      const dialog = e.currentTarget as HTMLElement;
      const focusable = dialog.querySelectorAll<HTMLElement>(
        'input, select, button, [tabindex]:not([tabindex="-1"])'
      );
      if (!focusable.length) return;
      const first = focusable[0];
      const last  = focusable[focusable.length - 1];
      if (e.shiftKey && document.activeElement === first) { e.preventDefault(); last.focus(); }
      else if (!e.shiftKey && document.activeElement === last) { e.preventDefault(); first.focus(); }
    }
  }

  $effect(() => {
    if (open) requestAnimationFrame(() => document.getElementById("profile-name")?.focus());
  });
</script>

{#if open}
  <div class="dialog-backdrop">
    <div
      class="dialog"
      role="dialog"
      aria-modal="true"
      aria-label={title}
      tabindex="-1"
      onkeydown={handleKeydown}
    >
      <h2>{title}</h2>

      <!-- BASIC -->
      <p class="sec-label">{m.profile_section_basic()}</p>
      <div class="grid-2">
        <div class="field">
          <label for="profile-name">{m.field_name()}</label>
          <input id="profile-name" type="text" bind:value={name} />
        </div>
        <div class="field">
          <label for="profile-desc">{m.field_description()}</label>
          <input id="profile-desc" type="text" bind:value={description} />
        </div>
        <div class="field col-span">
          <label for="profile-folder">{m.field_output_folder()}</label>
          <div class="folder-row">
            <input
              id="profile-folder"
              type="text"
              bind:value={outputFolder}
              bind:this={folderInputEl}
              readonly
              aria-readonly="true"
            />
            <button type="button" class="btn-browse" onclick={browseFolder}>
              {m.btn_browse_folder()}
            </button>
          </div>
        </div>
      </div>

      <!-- AUDIO -->
      <p class="sec-label">{m.profile_section_audio()}</p>
      <div class="grid-2">
        <div class="field">
          <label for="profile-mode">{m.field_output_mode()}</label>
          <select id="profile-mode" bind:value={outputMode}>
            {#each outputModes as mode}
              <option value={mode.value}>{mode.label}</option>
            {/each}
          </select>
        </div>
        <div class="field">
          <label for="profile-rate">{m.field_sample_rate()}</label>
          <select id="profile-rate" bind:value={sampleRate}>
            {#each sampleRates as rate}
              <option value={rate}>{rate} Hz</option>
            {/each}
          </select>
        </div>
        <div class="field">
          <label for="profile-mic-vol">
            {m.field_mic_volume()}
            <span class="slider-val" aria-hidden="true">{micVolume.toFixed(1)}</span>
          </label>
          <input
            id="profile-mic-vol" type="range" min="0" max="4" step="0.1"
            bind:value={micVolume}
            style="--fill: {micFillPct}%"
          />
        </div>
        <div class="field">
          <label for="profile-loop-vol">
            {m.field_loopback_volume()}
            <span class="slider-val" aria-hidden="true">{loopbackVolume.toFixed(1)}</span>
          </label>
          <input
            id="profile-loop-vol" type="range" min="0" max="4" step="0.1"
            bind:value={loopbackVolume}
            style="--fill: {loopbackFillPct}%"
          />
        </div>
      </div>

      <!-- FILE NAMES -->
      <p class="sec-label">{m.profile_section_filenames()}</p>
      <div class="grid-2">
        {#if showMicFilename}
          <div class="field">
            <label for="profile-mic-fn">{m.field_mic_filename()}</label>
            <input id="profile-mic-fn" type="text" bind:value={micFilename} />
          </div>
        {/if}
        {#if showLoopbackFilename}
          <div class="field">
            <label for="profile-loop-fn">{m.field_loopback_filename()}</label>
            <input id="profile-loop-fn" type="text" bind:value={loopbackFilename} />
          </div>
        {/if}
        {#if showMixFilename}
          <div class="field">
            <label for="profile-mix-fn">{m.field_mix_filename()}</label>
            <input id="profile-mix-fn" type="text" bind:value={mixFilename} />
          </div>
        {/if}
      </div>

      {#if error}
        <div class="error" role="alert">{error}</div>
      {/if}

      <div class="actions">
        <button type="button" class="btn-secondary" onclick={onclose}>{m.btn_cancel()}</button>
        <button type="button" class="btn-primary"   onclick={handleSubmit}>
          {isEdit ? m.btn_save() : m.btn_create()}
        </button>
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
    padding: 20px 20px 18px;
    width: 400px;
    /* No max-height / overflow — all fields fit in ~430px */
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
    color: var(--text-primary);
  }
  h2 { margin: 0 0 10px; font-size: 1.1rem; color: var(--text-primary); }

  .sec-label {
    font-size: 8px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.09em;
    color: var(--sec-label);
    margin: 8px 0 6px;
  }

  /* 2-column grid */
  .grid-2 {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 0 10px;
  }
  .col-span { grid-column: 1 / -1; }

  .field {
    display: flex;
    flex-direction: column;
    gap: 3px;
    margin-bottom: 8px;
  }
  .field label {
    font-weight: 600;
    font-size: 0.75rem;
    color: var(--text-secondary);
    display: flex;
    justify-content: space-between;
    align-items: baseline;
  }
  .slider-val {
    font-weight: 700;
    color: var(--accent);
  }

  input[type="text"],
  select {
    padding: 5px 7px;
    border: 1px solid var(--border);
    border-radius: 5px;
    font-size: 0.8rem;
    background: var(--surface);
    color: var(--text-primary);
    width: 100%;
  }
  input[type="text"]:focus-visible,
  select:focus-visible {
    outline: 2px solid var(--focus-ring);
    outline-offset: 2px;
  }

  .folder-row {
    display: flex;
    gap: 5px;
    align-items: center;
  }
  .folder-row input[type="text"] {
    flex: 1;
    min-width: 0;
    cursor: default;
    color: var(--text-secondary);
  }
  .btn-browse {
    flex-shrink: 0;
    padding: 5px 10px;
    border: 1px solid var(--border);
    border-radius: 5px;
    background: var(--surface);
    color: var(--text-primary);
    font-size: 0.8rem;
    font-weight: 600;
    cursor: pointer;
    white-space: nowrap;
  }
  .btn-browse:hover {
    background: var(--surface-hover, var(--border));
  }
  .btn-browse:focus-visible {
    outline: 2px solid var(--focus-ring);
    outline-offset: 2px;
  }

  /* range sliders inherit global styles from app.css */
  input[type="range"] {
    margin-top: 2px;
  }

  .error {
    padding: 7px 10px;
    background: var(--error-bg);
    border: 1px solid var(--error-border);
    border-radius: 5px;
    color: var(--error-text);
    font-size: 0.8rem;
    margin-bottom: 10px;
  }
  .actions {
    display: flex;
    gap: 8px;
    justify-content: flex-end;
    margin-top: 10px;
  }
  .btn-primary, .btn-secondary {
    padding: 7px 16px;
    border-radius: 6px;
    font-size: 0.875rem;
    font-weight: 700;
    cursor: pointer;
  }
  .btn-primary  { border: none; background: var(--accent); color: #ffffff; }
  .btn-primary:hover  { background: var(--accent-hover); }
  .btn-secondary { border: 1px solid var(--border); background: var(--surface); color: var(--text-primary); }
  .btn-secondary:hover { background: var(--surface-hover); }
</style>
