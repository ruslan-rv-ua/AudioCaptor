<script lang="ts">
  import { onMount } from "svelte";
  import type { RecordingProfile } from "./lib/types";
  import { getDevices, refreshDevices, initDeviceListener } from "./lib/stores/devices.svelte";
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
    applyProfile,
  } from "./lib/stores/recording.svelte";
  import {
    getProfiles,
    loadProfiles,
    saveProfile,
    deleteProfile,
    selectProfile,
  } from "./lib/stores/profiles.svelte";
  import { saveSettings } from "./lib/utils/invoke";
  import DeviceSelect from "./lib/components/DeviceSelect.svelte";
  import VolumeSlider from "./lib/components/VolumeSlider.svelte";
  import RecordControls from "./lib/components/RecordControls.svelte";
  import StatusIndicator from "./lib/components/StatusIndicator.svelte";
  import ProfileSelector from "./lib/components/ProfileSelector.svelte";
  import ProfileDialog from "./lib/components/ProfileDialog.svelte";

  const devices = getDevices();
  const recording = getRecording();
  const profileStore = getProfiles();

  let dialogOpen = $state(false);
  let editingProfile = $state<RecordingProfile | null>(null);

  let isRecording = $derived(recording.state !== "Idle");

  let liveRegionText = $state("");
  let capturingHotkey = $state(false);

  let saveTimeout: ReturnType<typeof setTimeout> | undefined;

  function scheduleSave() {
    clearTimeout(saveTimeout);
    saveTimeout = setTimeout(async () => {
      await saveSettings({
        version: 2,
        selectedMic: recording.selectedMic,
        selectedLoopback: recording.selectedLoopback,
        hotkey: recording.hotkey,
        soundEnabled: recording.soundEnabled,
        profiles: profileStore.list,
        activeProfileId: profileStore.activeId,
      });
    }, 500);
  }

  $effect(() => {
    const state = recording.state;
    if (state === "Recording") liveRegionText = "Recording started";
    else if (state === "Paused") liveRegionText = "Recording paused";
    else if (state === "Idle") liveRegionText = "Recording stopped";
  });

  function formatDuration(ms: number): string {
    const totalSeconds = Math.floor(ms / 1000);
    const hours = Math.floor(totalSeconds / 3600);
    const minutes = Math.floor((totalSeconds % 3600) / 60);
    const seconds = totalSeconds % 60;
    const pad = (n: number) => n.toString().padStart(2, "0");
    return `${pad(hours)}:${pad(minutes)}:${pad(seconds)}`;
  }

  function handleMnemonic(e: KeyboardEvent) {
    if (!e.altKey) return;

    const key = e.key.toLowerCase();
    switch (key) {
      case "s":
        if (recording.state === "Idle" && recording.canRecord) { e.preventDefault(); startRecording(); }
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
      case "i":
        e.preventDefault();
        if (recording.state !== "Idle") {
          liveRegionText = `${recording.state === "Paused" ? "Paused" : "Recording"}, ${formatDuration(recording.durationMs)}`;
        } else {
          liveRegionText = "Ready";
        }
        break;
    }
  }

  async function handleProfileSelect(id: string) {
    await selectProfile(id);
    const profile = profileStore.active;
    if (profile) applyProfile(profile);
    scheduleSave();
  }

  function handleProfileCreate() {
    editingProfile = null;
    dialogOpen = true;
  }

  function handleProfileEdit(profile: RecordingProfile) {
    editingProfile = profile;
    dialogOpen = true;
  }

  async function handleProfileSave(profile: RecordingProfile) {
    await saveProfile(profile);
    dialogOpen = false;
    if (profileStore.activeId === profile.id) {
      applyProfile(profile);
    }
    scheduleSave();
  }

  async function handleProfileDelete(id: string) {
    if (!confirm("Delete this profile?")) return;
    await deleteProfile(id);
    const active = profileStore.active;
    if (active) applyProfile(active);
    scheduleSave();
  }

  onMount(() => {
    (async () => {
      await loadSettingsIntoStore();
      await loadProfiles();
      // Apply active profile settings on startup
      const active = profileStore.active;
      if (active) applyProfile(active);
      await refreshDevices();
      await initRecordingListener();
      await initDeviceListener();
    })();
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
</script>

<main role="application" aria-label="AudioCaptor">
  <h1>AudioCaptor</h1>

  <StatusIndicator state={recording.state} durationMs={recording.durationMs} />

  <ProfileSelector
    profiles={profileStore.list}
    activeId={profileStore.activeId}
    disabled={isRecording}
    onselect={handleProfileSelect}
    oncreate={handleProfileCreate}
    onedit={handleProfileEdit}
    ondelete={handleProfileDelete}
  />

  <ProfileDialog
    profile={editingProfile}
    open={dialogOpen}
    onclose={() => dialogOpen = false}
    onsave={handleProfileSave}
  />

  <section aria-label="Audio devices">
    <DeviceSelect
      label="Microphone"
      devices={devices.microphones}
      value={recording.selectedMic}
      onchange={(id) => { recording.selectedMic = id; scheduleSave(); }}
      disabled={isRecording}
    />

    <DeviceSelect
      label="Loopback Device"
      devices={devices.loopbacks}
      value={recording.selectedLoopback}
      onchange={(id) => { recording.selectedLoopback = id; scheduleSave(); }}
      disabled={isRecording}
    />
  </section>

  <section aria-label="Volume controls">
    <VolumeSlider
      label="Microphone Volume"
      value={recording.micVolume}
      onchange={(v) => { updateMicVolume(v); scheduleSave(); }}
    />

    <VolumeSlider
      label="Loopback Volume"
      value={recording.loopbackVolume}
      onchange={(v) => { updateLoopbackVolume(v); scheduleSave(); }}
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
    canRecord={recording.canRecord}
    readinessHint={recording.readinessHint}
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
