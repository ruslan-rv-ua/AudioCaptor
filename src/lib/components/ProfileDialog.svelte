<script lang="ts">
  import type { RecordingProfile, OutputMode } from "../types";

  interface Props {
    profile: RecordingProfile | null;
    open: boolean;
    onclose: () => void;
    onsave: (profile: RecordingProfile) => void;
  }

  let { profile, open, onclose, onsave }: Props = $props();

  let name = $state("");
  let description = $state("");
  let outputFolder = $state("Recordings");
  let outputMode = $state<OutputMode>("Mix");
  let sampleRate = $state(48000);
  let micVolume = $state(1.0);
  let loopbackVolume = $state(0.5);
  let micFilename = $state("mic");
  let loopbackFilename = $state("loopback");
  let mixFilename = $state("mix");
  let error = $state("");
  let editId = $state<string | null>(null);

  const outputModes: { value: OutputMode; label: string }[] = [
    { value: "Microphone", label: "Microphone only" },
    { value: "Loopback", label: "System audio only" },
    { value: "Mix", label: "Mix (Mic + System)" },
    { value: "MixPlusMicrophone", label: "Mix + Microphone (2 files)" },
    { value: "MixPlusLoopback", label: "Mix + System audio (2 files)" },
  ];

  const sampleRates = [8000, 16000, 44100, 48000];

  $effect(() => {
    if (open && profile) {
      editId = profile.id;
      name = profile.name;
      description = profile.description;
      outputFolder = profile.outputFolder;
      outputMode = profile.outputMode;
      sampleRate = profile.sampleRate;
      micVolume = profile.micVolume;
      loopbackVolume = profile.loopbackVolume;
      micFilename = profile.micFilename;
      loopbackFilename = profile.loopbackFilename;
      mixFilename = profile.mixFilename;
    } else if (open && !profile) {
      editId = null;
      name = "";
      description = "";
      outputFolder = "Recordings";
      outputMode = "Mix";
      sampleRate = 48000;
      micVolume = 1.0;
      loopbackVolume = 0.5;
      micFilename = "mic";
      loopbackFilename = "loopback";
      mixFilename = "mix";
    }
    error = "";
  });

  let isEdit = $derived(editId !== null);
  let title = $derived(isEdit ? "Edit Profile" : "New Profile");

  function handleSubmit() {
    if (!name.trim()) {
      error = "Profile name is required";
      return;
    }
    const forbidden = /[<>:"/\\|?*]/;
    for (const fn of [micFilename, loopbackFilename, mixFilename]) {
      if (!fn.trim() || forbidden.test(fn)) {
        error = 'Filenames must not be empty or contain < > : " / \\ | ? *';
        return;
      }
    }

    const result: RecordingProfile = {
      id: editId ?? crypto.randomUUID(),
      name: name.trim(),
      description: description.trim(),
      outputFolder,
      outputMode,
      sampleRate,
      micVolume,
      loopbackVolume,
      micFilename: micFilename.trim(),
      loopbackFilename: loopbackFilename.trim(),
      mixFilename: mixFilename.trim(),
    };

    onsave(result);
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") onclose();
    // Focus trapping: cycle Tab within dialog
    if (e.key === "Tab") {
      const dialog = e.currentTarget as HTMLElement;
      const focusable = dialog.querySelectorAll<HTMLElement>(
        'input, select, button, [tabindex]:not([tabindex="-1"])'
      );
      if (focusable.length === 0) return;
      const first = focusable[0];
      const last = focusable[focusable.length - 1];
      if (e.shiftKey && document.activeElement === first) {
        e.preventDefault();
        last.focus();
      } else if (!e.shiftKey && document.activeElement === last) {
        e.preventDefault();
        first.focus();
      }
    }
  }

  // Auto-focus the name field when dialog opens
  $effect(() => {
    if (open) {
      requestAnimationFrame(() => {
        document.getElementById("profile-name")?.focus();
      });
    }
  });
</script>

{#if open}
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <!-- svelte-ignore a11y_interactive_supports_focus -->
  <div class="dialog-backdrop" role="dialog" aria-modal="true" aria-label={title} onkeydown={handleKeydown}>
    <div class="dialog">
      <h2>{title}</h2>

      <div class="field">
        <label for="profile-name">Name</label>
        <input id="profile-name" type="text" bind:value={name} />
      </div>

      <div class="field">
        <label for="profile-desc">Description</label>
        <input id="profile-desc" type="text" bind:value={description} />
      </div>

      <div class="field">
        <label for="profile-folder">Output Folder</label>
        <input id="profile-folder" type="text" bind:value={outputFolder} />
      </div>

      <div class="field">
        <label for="profile-mode">Output Mode</label>
        <select id="profile-mode" bind:value={outputMode}>
          {#each outputModes as mode}
            <option value={mode.value}>{mode.label}</option>
          {/each}
        </select>
      </div>

      <div class="field">
        <label for="profile-rate">Sample Rate</label>
        <select id="profile-rate" bind:value={sampleRate}>
          {#each sampleRates as rate}
            <option value={rate}>{rate} Hz</option>
          {/each}
        </select>
      </div>

      <div class="field-row">
        <div class="field">
          <label for="profile-mic-vol">Mic Volume</label>
          <input id="profile-mic-vol" type="range" min="0" max="4" step="0.1" bind:value={micVolume} />
          <span>{micVolume.toFixed(1)}</span>
        </div>
        <div class="field">
          <label for="profile-loop-vol">Loopback Volume</label>
          <input id="profile-loop-vol" type="range" min="0" max="4" step="0.1" bind:value={loopbackVolume} />
          <span>{loopbackVolume.toFixed(1)}</span>
        </div>
      </div>

      <div class="field">
        <label for="profile-mic-fn">Mic Filename</label>
        <input id="profile-mic-fn" type="text" bind:value={micFilename} />
      </div>

      <div class="field">
        <label for="profile-loop-fn">Loopback Filename</label>
        <input id="profile-loop-fn" type="text" bind:value={loopbackFilename} />
      </div>

      <div class="field">
        <label for="profile-mix-fn">Mix Filename</label>
        <input id="profile-mix-fn" type="text" bind:value={mixFilename} />
      </div>

      {#if error}
        <div class="error" role="alert">{error}</div>
      {/if}

      <div class="actions">
        <button type="button" class="btn-secondary" onclick={onclose}>Cancel</button>
        <button type="button" class="btn-primary" onclick={handleSubmit}>
          {isEdit ? "Save" : "Create"}
        </button>
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
    width: 400px;
    max-height: 90vh;
    overflow-y: auto;
    box-shadow: 0 4px 24px rgba(0, 0, 0, 0.2);
  }

  h2 {
    margin: 0 0 16px;
    font-size: 1.2rem;
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 4px;
    margin-bottom: 12px;
  }

  .field label {
    font-weight: 600;
    font-size: 0.8rem;
  }

  .field input[type="text"],
  .field select {
    padding: 8px;
    border: 1px solid #ccc;
    border-radius: 4px;
    font-size: 0.875rem;
  }

  .field input:focus-visible,
  .field select:focus-visible {
    outline: 2px solid #0066cc;
    outline-offset: 2px;
  }

  .field-row {
    display: flex;
    gap: 12px;
  }

  .field-row .field {
    flex: 1;
  }

  .error {
    padding: 8px;
    background: #fef2f2;
    border: 1px solid #fecaca;
    border-radius: 4px;
    color: #dc2626;
    font-size: 0.85rem;
    margin-bottom: 12px;
  }

  .actions {
    display: flex;
    gap: 8px;
    justify-content: flex-end;
    margin-top: 16px;
  }

  .btn-primary, .btn-secondary {
    padding: 8px 16px;
    border: none;
    border-radius: 4px;
    font-size: 0.875rem;
    font-weight: 600;
    cursor: pointer;
  }

  .btn-primary { background: #2563eb; color: white; }
  .btn-primary:hover { background: #1d4ed8; }
  .btn-secondary { background: #e5e7eb; color: #374151; }
  .btn-secondary:hover { background: #d1d5db; }
</style>
