<script lang="ts">
  import { onMount } from "svelte";
  import type { OutputMode } from "./lib/types";
  import { getDevices, refreshDevices } from "./lib/stores/devices.svelte";
  import {
    getRecording,
    initRecordingListener,
    startRecording,
    pauseRecording,
    resumeRecording,
    stopRecording,
    updateMicVolume,
    updateLoopbackVolume,
    updateSoundEnabled,
    updateHotkey,
    loadSettingsIntoStore,
  } from "./lib/stores/recording.svelte";
  import { saveSettings } from "./lib/utils/invoke";
  import DeviceSelect from "./lib/components/DeviceSelect.svelte";
  import VolumeSlider from "./lib/components/VolumeSlider.svelte";
  import RecordControls from "./lib/components/RecordControls.svelte";
  import StatusIndicator from "./lib/components/StatusIndicator.svelte";

  const devices = getDevices();
  const recording = getRecording();

  const outputModes: { value: OutputMode; label: string }[] = [
    { value: "Microphone", label: "Microphone only" },
    { value: "Loopback", label: "System audio only" },
    { value: "Mix", label: "Mix (Mic + System)" },
  ];

  const sampleRates = [8000, 16000, 44100, 48000];

  let isRecording = $derived(recording.state !== "Idle");

  let liveRegionText = $state("");
  let capturingHotkey = $state(false);

  let saveTimeout: ReturnType<typeof setTimeout> | undefined;

  function scheduleSave() {
    clearTimeout(saveTimeout);
    saveTimeout = setTimeout(async () => {
      await saveSettings({
        selectedMic: recording.selectedMic,
        selectedLoopback: recording.selectedLoopback,
        micVolume: recording.micVolume,
        loopbackVolume: recording.loopbackVolume,
        outputMode: recording.outputMode,
        sampleRate: recording.sampleRate,
        hotkey: recording.hotkey,
        soundEnabled: recording.soundEnabled,
      });
    }, 500);
  }

  $effect(() => {
    const state = recording.state;
    if (state === "Recording") liveRegionText = "Recording started";
    else if (state === "Paused") liveRegionText = "Recording paused";
    else if (state === "Idle") liveRegionText = "Recording stopped";
  });

  function handleMnemonic(e: KeyboardEvent) {
    if (!e.altKey) return;

    const key = e.key.toLowerCase();
    switch (key) {
      case "s":
        if (recording.state === "Idle") { e.preventDefault(); startRecording(); }
        break;
      case "p":
        if (recording.state === "Recording") { e.preventDefault(); pauseRecording(); }
        break;
      case "r":
        if (recording.state === "Paused") { e.preventDefault(); resumeRecording(); }
        break;
      case "t":
        if (recording.state !== "Idle") { e.preventDefault(); stopRecording(); }
        break;
    }
  }

  onMount(async () => {
    await loadSettingsIntoStore();
    await refreshDevices();
    await initRecordingListener();
    window.addEventListener("keydown", handleMnemonic);
    return () => window.removeEventListener("keydown", handleMnemonic);
  });

  function startHotkeyCapture() {
    capturingHotkey = true;

    function onKeyDown(e: KeyboardEvent) {
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

      parts.push(key);
      const shortcut = parts.join("+");

      updateHotkey(shortcut);
      scheduleSave();
      capturingHotkey = false;
      window.removeEventListener("keydown", onKeyDown, true);
    }

    window.addEventListener("keydown", onKeyDown, true);
  }

  function handleOutputModeChange(e: Event) {
    const target = e.target as HTMLSelectElement;
    recording.outputMode = target.value as OutputMode;
  }

  function handleSampleRateChange(e: Event) {
    const target = e.target as HTMLSelectElement;
    recording.sampleRate = parseInt(target.value);
  }
</script>

<main role="application" aria-label="AudioCaptor">
  <h1>AudioCaptor</h1>

  <StatusIndicator state={recording.state} durationMs={recording.durationMs} />

  <section aria-label="Audio devices">
    <DeviceSelect
      label="Microphone"
      devices={devices.microphones}
      value={recording.selectedMic}
      onchange={(id) => (recording.selectedMic = id)}
      disabled={isRecording}
    />

    <DeviceSelect
      label="Loopback Device"
      devices={devices.loopbacks}
      value={recording.selectedLoopback}
      onchange={(id) => (recording.selectedLoopback = id)}
      disabled={isRecording}
    />
  </section>

  <section aria-label="Output settings">
    <div class="setting-row">
      <label for="output-mode">Output Mode</label>
      <select
        id="output-mode"
        aria-label="Output mode"
        disabled={isRecording}
        onchange={handleOutputModeChange}
      >
        {#each outputModes as mode}
          <option value={mode.value} selected={mode.value === recording.outputMode}>
            {mode.label}
          </option>
        {/each}
      </select>
    </div>

    <div class="setting-row">
      <label for="sample-rate">Sample Rate</label>
      <select
        id="sample-rate"
        aria-label="Sample rate"
        disabled={isRecording}
        onchange={handleSampleRateChange}
      >
        {#each sampleRates as rate}
          <option value={rate} selected={rate === recording.sampleRate}>
            {rate} Hz
          </option>
        {/each}
      </select>
    </div>
  </section>

  <section aria-label="Volume controls">
    <VolumeSlider
      label="Microphone Volume"
      value={recording.micVolume}
      onchange={updateMicVolume}
    />

    <VolumeSlider
      label="Loopback Volume"
      value={recording.loopbackVolume}
      onchange={updateLoopbackVolume}
    />
  </section>

  <section aria-label="Settings">
    <div class="setting-row">
      <label for="hotkey-display">Global Hotkey</label>
      <div class="hotkey-row">
        <input
          id="hotkey-display"
          type="text"
          value={recording.hotkey}
          readonly
          aria-label="Current hotkey: {recording.hotkey}"
          class="hotkey-input"
        />
        <button
          type="button"
          class="btn-small"
          onclick={startHotkeyCapture}
          disabled={isRecording || capturingHotkey}
        >
          {capturingHotkey ? "Press a key..." : "Change"}
        </button>
      </div>
    </div>

    <div class="setting-row">
      <label class="checkbox-label">
        <input
          type="checkbox"
          checked={recording.soundEnabled}
          onchange={(e) => {
            const target = e.target as HTMLInputElement;
            updateSoundEnabled(target.checked);
            scheduleSave();
          }}
        />
        Sound notifications
      </label>
    </div>
  </section>

  <RecordControls
    recordingState={recording.state}
    onstart={startRecording}
    onpause={pauseRecording}
    onresume={resumeRecording}
    onstop={stopRecording}
  />

  {#if recording.error}
    <div class="error" role="alert" aria-live="assertive">
      {recording.error}
    </div>
  {/if}

  <!-- Live region for screen reader announcements -->
  <div aria-live="polite" aria-atomic="true" class="visually-hidden">{liveRegionText}</div>
</main>

<style>
  main {
    max-width: 480px;
    margin: 0 auto;
    padding: 24px 16px;
  }

  h1 {
    text-align: center;
    margin: 0 0 16px;
    font-size: 1.5rem;
  }

  section {
    margin-bottom: 16px;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .setting-row {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .setting-row label {
    font-weight: 600;
    font-size: 0.875rem;
  }

  .setting-row select {
    padding: 8px;
    border: 1px solid #ccc;
    border-radius: 4px;
    font-size: 0.875rem;
    background: #fff;
  }

  .setting-row select:focus-visible {
    outline: 2px solid #0066cc;
    outline-offset: 2px;
  }

  .setting-row select:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .error {
    padding: 12px;
    background: #fef2f2;
    border: 1px solid #fecaca;
    border-radius: 6px;
    color: #dc2626;
    text-align: center;
    margin-top: 8px;
  }

  .hotkey-row {
    display: flex;
    gap: 8px;
    align-items: center;
  }

  .hotkey-input {
    flex: 1;
    padding: 8px;
    border: 1px solid #ccc;
    border-radius: 4px;
    font-size: 0.875rem;
    background: #f5f5f5;
    cursor: default;
  }

  .btn-small {
    padding: 8px 12px;
    border: none;
    border-radius: 4px;
    font-size: 0.8rem;
    font-weight: 600;
    cursor: pointer;
    background: #6b7280;
    color: white;
    white-space: nowrap;
  }

  .btn-small:hover:not(:disabled) { background: #4b5563; }
  .btn-small:disabled { opacity: 0.6; cursor: not-allowed; }

  .checkbox-label {
    display: flex;
    align-items: center;
    gap: 8px;
    cursor: pointer;
    font-size: 0.875rem;
  }
</style>
