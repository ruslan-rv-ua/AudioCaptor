<script lang="ts">
  import { onMount } from "svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
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
  import { saveSettings, loadSettings } from "./lib/utils/invoke";
  import { initLanguage } from "./lib/i18n";
  import { getSettings, loadSettingsFields } from "./lib/stores/settings.svelte";
  import * as m from "./paraglide/messages";
  import DeviceSelect from "./lib/components/DeviceSelect.svelte";
  import VolumeSlider from "./lib/components/VolumeSlider.svelte";
  import RecordControls from "./lib/components/RecordControls.svelte";
  import StatusIndicator from "./lib/components/StatusIndicator.svelte";
  import ProfileSelector from "./lib/components/ProfileSelector.svelte";
  import ProfileDialog from "./lib/components/ProfileDialog.svelte";
  import SettingsDialog from "./lib/components/SettingsDialog.svelte";
  import ConfirmExitDialog from "./lib/components/ConfirmExitDialog.svelte";

  const devices = getDevices();
  const recording = getRecording();
  const profileStore = getProfiles();

  let initialized = $state(false);
  const appSettings = getSettings();
  const appWindow = getCurrentWindow();

  let dialogOpen = $state(false);
  let editingProfile = $state<RecordingProfile | null>(null);
  let settingsOpen = $state(false);
  let confirmExitOpen = $state(false);

  let isRecording = $derived(recording.state !== "Idle");

  let liveRegionText = $state("");
  let saveTimeout: ReturnType<typeof setTimeout> | undefined;

  function scheduleSave() {
    clearTimeout(saveTimeout);
    saveTimeout = setTimeout(async () => {
      await saveSettings({
        version: appSettings.version,
        selectedMic: recording.selectedMic,
        selectedLoopback: recording.selectedLoopback,
        hotkey: appSettings.hotkey,
        soundEnabled: appSettings.soundEnabled,
        language: appSettings.language,
        confirmExitDuringRecording: appSettings.confirmExitDuringRecording,
        profiles: profileStore.list,
        activeProfileId: profileStore.activeId,
      });
    }, 500);
  }

  $effect(() => {
    if (!initialized) return;
    const state = recording.state;
    if (state === "Recording") liveRegionText = m.live_recording_started();
    else if (state === "Paused") liveRegionText = m.live_recording_paused();
    else if (state === "Idle") liveRegionText = m.live_recording_stopped();
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
          liveRegionText = m.live_status_info({
            state: recording.state === "Paused" ? m.status_paused() : m.status_recording(),
            duration: formatDuration(recording.durationMs),
          });
        } else {
          liveRegionText = m.status_ready();
        }
        break;
      case "f4":
        // WebView2 may consume WM_SYSKEYDOWN before it reaches the native
        // window proc, preventing the OS from generating WM_CLOSE for Alt+F4.
        // Explicitly trigger the same close flow as the title-bar X button.
        e.preventDefault();
        void appWindow.close();
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
    if (!confirm(m.delete_profile_confirm())) return;
    await deleteProfile(id);
    const active = profileStore.active;
    if (active) applyProfile(active);
    scheduleSave();
  }

  function handleSettingsSave(_patch: Partial<import("./lib/types").Settings>) {
    scheduleSave();
  }

  async function handleStopAndExit() {
    confirmExitOpen = false;
    try { await stopRecording(); } catch { /* already stopped */ }
    await appWindow.close();
  }

  onMount(() => {
    (async () => {
      const rawSettings = await loadSettings();
      loadSettingsFields(rawSettings);
      initLanguage(appSettings.language);
      await loadSettingsIntoStore();
      await loadProfiles();
      const active = profileStore.active;
      if (active) applyProfile(active);
      await refreshDevices();
      await initRecordingListener();
      await initDeviceListener();

      await appWindow.onCloseRequested(async (event) => {
        if (appSettings.confirmExitDuringRecording && isRecording) {
          event.preventDefault();
          confirmExitOpen = true;
        }
      });

      initialized = true;
    })();
    window.addEventListener("keydown", handleMnemonic);
    return () => window.removeEventListener("keydown", handleMnemonic);
  });
</script>

{#if initialized}
<main role="application" aria-label="AudioCaptor">
  <div class="header-row">
    <h1>{m.app_title()}</h1>
    <button
      type="button"
      class="btn-settings"
      aria-label={m.settings_btn_aria()}
      onclick={() => settingsOpen = true}
    >⚙</button>
  </div>

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

  <section aria-label={m.audio_devices_section()}>
    <DeviceSelect
      label={m.mic_label()}
      devices={devices.microphones}
      value={recording.selectedMic}
      onchange={(id) => { recording.selectedMic = id; scheduleSave(); }}
      disabled={isRecording}
    />
    <DeviceSelect
      label={m.loopback_label()}
      devices={devices.loopbacks}
      value={recording.selectedLoopback}
      onchange={(id) => { recording.selectedLoopback = id; scheduleSave(); }}
      disabled={isRecording}
    />
  </section>

  <section aria-label={m.volume_controls_section()}>
    <VolumeSlider
      label={m.mic_volume_label()}
      value={recording.micVolume}
      onchange={(v) => { updateMicVolume(v); scheduleSave(); }}
    />
    <VolumeSlider
      label={m.loopback_volume_label()}
      value={recording.loopbackVolume}
      onchange={(v) => { updateLoopbackVolume(v); scheduleSave(); }}
    />
  </section>

  <RecordControls
    recordingState={recording.state}
    canRecord={recording.canRecord}
    needsMic={recording.needsMic}
    needsLoopback={recording.needsLoopback}
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

  <div role="status" aria-atomic="true" class="visually-hidden">{liveRegionText}</div>

  <SettingsDialog
    open={settingsOpen}
    hotkey={appSettings.hotkey}
    soundEnabled={appSettings.soundEnabled}
    confirmExitDuringRecording={appSettings.confirmExitDuringRecording}
    language={appSettings.language}
    onclose={() => settingsOpen = false}
    onsave={handleSettingsSave}
  />

  <ConfirmExitDialog
    open={confirmExitOpen}
    onstopandexit={handleStopAndExit}
    oncancel={() => confirmExitOpen = false}
  />
</main>
{/if}

<style>
  main {
    max-width: 480px;
    margin: 0 auto;
    padding: 24px 16px;
  }

  .header-row {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 12px;
    margin: 0 0 16px;
  }

  .header-row h1 { margin: 0; }

  .btn-settings {
    background: none;
    border: 1px solid #ccc;
    border-radius: 4px;
    padding: 4px 8px;
    cursor: pointer;
    font-size: 1rem;
  }

  .btn-settings:hover { background: #f0f0f0; }

  section {
    margin-bottom: 16px;
    display: flex;
    flex-direction: column;
    gap: 12px;
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

  .visually-hidden {
    position: absolute;
    width: 1px;
    height: 1px;
    padding: 0;
    margin: -1px;
    overflow: hidden;
    clip: rect(0,0,0,0);
    white-space: nowrap;
    border: 0;
  }
</style>
