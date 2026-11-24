<script lang="ts">
  import { onMount, tick } from "svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { listen } from "@tauri-apps/api/event";
  import type { RecordingProfile, RecordingState } from "./lib/types";
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
  import * as api from "./lib/utils/invoke";
  import { initLanguage } from "./lib/i18n";
  import { getSettings, loadSettingsFields, setMinimizeToTrayOnFocusLoss } from "./lib/stores/settings.svelte";
  import {
    initTheme,
    setTheme,
    cleanupTheme,
  } from "./lib/stores/theme.svelte";
  import { setThemePreference } from "./lib/stores/settings.svelte";
  import type { Theme } from "./lib/types";
  import * as m from "./paraglide/messages";
  import { formatDuration } from "./lib/utils/format";
  import DeviceSelect from "./lib/components/DeviceSelect.svelte";
  import VolumeSlider from "./lib/components/VolumeSlider.svelte";
  import RecordControls from "./lib/components/RecordControls.svelte";
  import StatusIndicator from "./lib/components/StatusIndicator.svelte";
  import ProfileSelector from "./lib/components/ProfileSelector.svelte";
  import ProfileDialog from "./lib/components/ProfileDialog.svelte";
  import SettingsDialog from "./lib/components/SettingsDialog.svelte";
  import ConfirmExitDialog from "./lib/components/ConfirmExitDialog.svelte";
  import ConfirmDeleteDialog from "./lib/components/ConfirmDeleteDialog.svelte";
  import AboutDialog from "./lib/components/AboutDialog.svelte";

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
  let deleteConfirmOpen = $state(false);
  let aboutOpen = $state(false);
  let deletingProfileId = $state<string | null>(null);
  let deletingProfileName = $derived(
    deletingProfileId
      ? (profileStore.list.find(p => p.id === deletingProfileId)?.name ?? "")
      : ""
  );

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
        minimizeToTrayOnFocusLoss: appSettings.minimizeToTrayOnFocusLoss,
        profiles: profileStore.list,
        activeProfileId: profileStore.activeId,
        theme: appSettings.theme,
      });
    }, 500);
  }

  // Announces each recording state change. Recording that follows Paused is a
  // resumed recording, not a new one, whatever resumed it (Alt+R, the Resume
  // button, the global hotkey or the tray menu).
  let announcedState: RecordingState | undefined;
  $effect(() => {
    if (!initialized) return;
    const state = recording.state;
    const previous = announcedState;
    announcedState = state;
    if (state === "Recording" && previous === "Paused") liveRegionText = m.live_recording_resumed();
    else if (state === "Recording") liveRegionText = m.live_recording_started();
    else if (state === "Paused") liveRegionText = m.live_recording_paused();
    else if (state === "Idle") liveRegionText = m.live_recording_stopped();
  });

  // Focus that a recording state change takes away (backlog doc-3): starting a
  // recording disables the profile and device controls, and Start, Pause and
  // Resume each exist in one state only. If the element with focus is now
  // disabled or gone (focus on the page), focus goes to the first record
  // button of the new state (Start, Pause or Resume), and NVDA names it instead
  // of saying "unavailable" or nothing. This runs right after the DOM update,
  // in the same task, so focus moves once and straight to the button. Focus
  // anywhere else stays where it is, and nothing moves when the window loads.
  let previousState: RecordingState | undefined;
  $effect(() => {
    const state = recording.state;
    if (!initialized) return;
    const changed = previousState !== undefined && previousState !== state;
    previousState = state;
    if (!changed) return;
    const focused = document.activeElement;
    if (focused instanceof HTMLElement && focused !== document.body && !focused.matches(":disabled")) return;
    document.querySelector<HTMLElement>(".record-controls button")?.focus();
  });

  // Focus return (APG modal dialog pattern, backlog doc-3): when a dialog opens,
  // remember the element that has focus; when it closes, give focus back to that
  // element, once and straight to it. Each dialog keeps its own element, because
  // About (F1, tray) and the exit confirmation can open on top of another dialog.
  // After Save or Delete of a profile, focus goes to the profile list instead,
  // where NVDA names the active profile (the APG exception for a finished task).
  type Dialog = "about" | "settings" | "profile" | "delete" | "exit";
  const openers = new Map<Dialog, Element | null>();

  function rememberFocus(dialog: Dialog) {
    openers.set(dialog, document.activeElement);
  }

  // Call after the dialog's open flag is cleared. Focus goes to the element with
  // `targetId` when given, otherwise back to the opener. If that element is gone
  // or cannot take focus (removed, disabled, hidden, or nothing had focus), focus
  // goes to the profile list, and while that is disabled (recording) to the first
  // recording button.
  async function returnFocus(dialog: Dialog, targetId?: string) {
    const opener = openers.get(dialog);
    openers.delete(dialog);
    await tick();
    const targets = [
      targetId ? document.getElementById(targetId) : opener,
      document.getElementById("profile-select"),
      document.querySelector(".record-controls button"),
    ];
    for (const el of targets) {
      if (!(el instanceof HTMLElement) || el === document.body) continue;
      el.focus();
      if (document.activeElement === el) return;
    }
  }

  function openAbout() {
    if (aboutOpen) return;
    rememberFocus("about");
    aboutOpen = true;
  }

  function openConfirmExit() {
    if (confirmExitOpen) return;
    rememberFocus("exit");
    confirmExitOpen = true;
  }

  function handleMnemonic(e: KeyboardEvent) {
    if (e.key === "F1") { e.preventDefault(); openAbout(); return; }
    if (e.key === "Escape" && !settingsOpen && !dialogOpen && !confirmExitOpen && !deleteConfirmOpen && !aboutOpen) {
      e.preventDefault();
      void appWindow.hide();
      return;
    }
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
    rememberFocus("profile");
    dialogOpen = true;
  }

  function handleProfileEdit(profile: RecordingProfile) {
    editingProfile = profile;
    rememberFocus("profile");
    dialogOpen = true;
  }

  async function handleProfileSave(profile: RecordingProfile) {
    const isNew = !profileStore.list.some(p => p.id === profile.id);
    await saveProfile(profile);
    if (isNew) {
      await selectProfile(profile.id);
    }
    if (profileStore.activeId === profile.id) {
      applyProfile(profile);
    }
    scheduleSave();
    // Closed only now, so focus goes from the dialog straight to the profile
    // list, where NVDA names the active profile, without resting on the page.
    dialogOpen = false;
    await returnFocus("profile", "profile-select");
  }

  async function handleProfileDelete(id: string) {
    deletingProfileId = id;
    rememberFocus("delete");
    deleteConfirmOpen = true;
  }

  let deleting = false;

  async function handleDeleteConfirm() {
    const id = deletingProfileId;
    if (!id || deleting) return; // a second press while the first one runs
    deleting = true;
    try {
      await deleteProfile(id);
      const active = profileStore.active;
      if (active) applyProfile(active);
      scheduleSave();
    } finally {
      deleting = false;
      // Closed only now, once the list has changed, so focus goes from the
      // dialog straight to the profile list, where NVDA names the active
      // profile, without resting on the page.
      deleteConfirmOpen = false;
      deletingProfileId = null;
      await returnFocus("delete", "profile-select");
    }
  }

  async function handleDeleteCancel() {
    deleteConfirmOpen = false;
    deletingProfileId = null;
    await returnFocus("delete");
  }

  function handleSettingsSave(_patch: Partial<import("./lib/types").Settings>) {
    scheduleSave();
  }

  function handleThemeChange(t: Theme) {
    setTheme(t);             // applies to DOM + writes localStorage
    setThemePreference(t);   // updates settings store reactive state
    scheduleSave();          // persists to Tauri backend
  }

  async function handleStopAndExit() {
    confirmExitOpen = false;
    try { await stopRecording(); } catch { /* already stopped */ }
    await api.quitApp();
  }

  onMount(() => {
    let unlistenFocus: (() => void) | undefined;
    let unlistenTrayQuit: (() => void) | undefined;

    (async () => {
      try {
        const rawSettings = await loadSettings();
        loadSettingsFields(rawSettings);
        initTheme(appSettings.theme);
      } catch {
        // settings load failed — show window with default theme anyway
      } finally {
        await appWindow.show();
      }
      initLanguage(appSettings.language);
      await loadSettingsIntoStore();
      await loadProfiles();
      const active = profileStore.active;
      if (active) applyProfile(active);
      await refreshDevices();
      await initRecordingListener();
      await initDeviceListener();

      await appWindow.onCloseRequested(async (event) => {
        event.preventDefault();
        if (isRecording) {
          if (appSettings.confirmExitDuringRecording) {
            openConfirmExit();
          } else {
            await handleStopAndExit();
          }
        } else {
          void api.quitApp();
        }
      });

      unlistenFocus = await appWindow.onFocusChanged(({ payload: focused }) => {
        if (!focused
            && appSettings.minimizeToTrayOnFocusLoss
            && !settingsOpen && !dialogOpen && !confirmExitOpen && !deleteConfirmOpen && !aboutOpen) {
          void appWindow.hide();
        }
      });

      await listen("tray-about-requested", () => openAbout());

      unlistenTrayQuit = await listen("tray-quit-requested", async () => {
        if (isRecording) {
          if (appSettings.confirmExitDuringRecording) {
            openConfirmExit();
          } else {
            await handleStopAndExit();
          }
        } else {
          void api.quitApp();
        }
      });

      initialized = true;
      await tick();
      document.querySelector<HTMLSelectElement>('.profile-controls select')?.focus();
    })();
    window.addEventListener("keydown", handleMnemonic);
    return () => {
      window.removeEventListener("keydown", handleMnemonic);
      cleanupTheme();
      unlistenFocus?.();
      unlistenTrayQuit?.();
    };
  });
</script>

{#if initialized}
<main aria-label="AudioCaptor">
  <div class="header-row">
    <h1>{m.app_title()}</h1>
    <div class="header-btns">
      <button
        type="button"
        class="btn-tray"
        aria-label={m.btn_minimize_to_tray_aria()}
        onclick={() => appWindow.hide()}
      >⊟</button>
      <button
        type="button"
        class="btn-about"
        aria-label={m.about_btn_aria()}
        onclick={openAbout}
      >ⓘ</button>
      <button
        type="button"
        class="btn-settings"
        aria-label={m.settings_btn_aria()}
        onclick={() => { rememberFocus("settings"); settingsOpen = true; }}
      >⚙</button>
    </div>
  </div>

  <StatusIndicator state={recording.state} durationMs={recording.durationMs} />

  <div class="section-card">
    <h2 class="sec-label">{m.profile_label()}</h2>
    <ProfileSelector
      profiles={profileStore.list}
      activeId={profileStore.activeId}
      disabled={isRecording}
      onselect={handleProfileSelect}
      oncreate={handleProfileCreate}
      onedit={handleProfileEdit}
      ondelete={handleProfileDelete}
    />
  </div>

  <ProfileDialog
    profile={editingProfile}
    open={dialogOpen}
    onclose={async () => { dialogOpen = false; await returnFocus("profile"); }}
    onsave={handleProfileSave}
  />

  <div class="section-card" aria-label={m.audio_devices_section()}>
    <h2 class="sec-label">{m.audio_devices_section()}</h2>
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
  </div>

  <div class="section-card" aria-label={m.volume_controls_section()}>
    <h2 class="sec-label">{m.volume_controls_section()}</h2>
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
  </div>

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

  <AboutDialog
    open={aboutOpen}
    hotkey={appSettings.hotkey}
    onclose={async () => { aboutOpen = false; await returnFocus("about"); }}
  />

  <SettingsDialog
    open={settingsOpen}
    hotkey={appSettings.hotkey}
    soundEnabled={appSettings.soundEnabled}
    confirmExitDuringRecording={appSettings.confirmExitDuringRecording}
    minimizeToTrayOnFocusLoss={appSettings.minimizeToTrayOnFocusLoss}
    language={appSettings.language}
    theme={appSettings.theme}
    onclose={async () => { settingsOpen = false; await returnFocus("settings"); }}
    onsave={handleSettingsSave}
    onthemechange={handleThemeChange}
    onminimizetotraytoggle={(v) => { setMinimizeToTrayOnFocusLoss(v); scheduleSave(); }}
  />

  <ConfirmExitDialog
    open={confirmExitOpen}
    onstopandexit={handleStopAndExit}
    oncancel={async () => { confirmExitOpen = false; await returnFocus("exit"); }}
  />

  <ConfirmDeleteDialog
    open={deleteConfirmOpen}
    profileName={deletingProfileName}
    onconfirm={handleDeleteConfirm}
    oncancel={handleDeleteCancel}
  />
</main>
{/if}

<style>
  main {
    max-width: 480px;
    margin: 0 auto;
    padding: 20px 16px 16px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .header-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 4px;
  }

  .header-row h1 {
    margin: 0;
    font-size: 18px;
    font-weight: 700;
    color: var(--text-primary);
    letter-spacing: -0.01em;
  }

  .header-btns {
    display: flex;
    gap: 6px;
    align-items: center;
  }

  .btn-tray,
  .btn-about,
  .btn-settings {
    width: 36px;
    height: 36px;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 7px;
    cursor: pointer;
    font-size: 20px;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-primary);
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.06);
    transition: background 0.1s;
  }

  .btn-tray:hover,
  .btn-about:hover,
  .btn-settings:hover {
    background: var(--surface-hover);
  }

  /* Section cards — Profile, Devices, Volume */
  .section-card {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 10px 12px 11px;
    box-shadow: var(--section-card-shadow);
  }

  .sec-label {
    font-size: 8px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.09em;
    color: var(--sec-label);
    margin: 0 0 8px;
  }

  .error {
    padding: 10px 12px;
    background: var(--error-bg);
    border: 1px solid var(--error-border);
    border-radius: 6px;
    color: var(--error-text);
    text-align: center;
    font-size: 0.875rem;
  }

</style>
