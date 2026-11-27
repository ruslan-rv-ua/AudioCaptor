<script lang="ts">
  import { onMount, tick, untrack } from "svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { listen } from "@tauri-apps/api/event";
  import type { RecordingProfile, RecordingState } from "./lib/types";
  import { getDevices, refreshDevices, initDeviceListener } from "./lib/stores/devices.svelte";
  import {
    getRecording,
    initRecordingListener,
    startRecording,
    showError,
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
  import { getConversion, initConversionListener } from "./lib/stores/conversion.svelte";
  import { conversionInfo } from "./lib/utils/conversionText";
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
  const conversion = getConversion();

  let initialized = $state(false);
  const appSettings = getSettings();
  const appWindow = getCurrentWindow();

  let dialogOpen = $state(false);
  let editingProfile = $state<RecordingProfile | null>(null);
  let settingsOpen = $state(false);
  let confirmExitOpen = $state(false);
  // What the exit confirmation asks about: a running recording, or only the
  // conversions that are not finished. Set when the dialog opens.
  let confirmExitRecording = $state(false);
  let deleteConfirmOpen = $state(false);
  let aboutOpen = $state(false);
  let deletingProfileId = $state<string | null>(null);
  let deletingProfileName = $derived(
    deletingProfileId
      ? (profileStore.list.find(p => p.id === deletingProfileId)?.name ?? "")
      : ""
  );

  let isRecording = $derived(recording.state !== "Idle");

  // The line under the timer while recordings are converted. Sighted users see
  // it; a screen reader gets the same from the announcements and Alt+I.
  let conversionLine = $derived.by(() => {
    if (!conversion.busy || !conversion.format) return "";
    const line = m.conversion_progress({
      format: conversion.format,
      percent: Math.floor(conversion.progress * 100),
    });
    return conversion.waiting > 0
      ? `${line}. ${m.conversion_files_waiting({ count: conversion.waiting })}`
      : line;
  });

  let liveRegionText = $state("");

  // A screen reader speaks the announcement region only when its text changes,
  // so the same text twice in a row (Alt+I pressed again in the same state)
  // gets an invisible zero-width space at the end, and the next same text
  // drops it again.
  function announce(text: string) {
    liveRegionText = untrack(() => liveRegionText) === text ? `${text}​` : text;
  }

  // After a recording stops, what the conversions report (the end of the
  // queue, a recording that could not be converted) waits until STOP_READ_MS
  // have passed since the stop announcement, about the time NVDA takes to read
  // it (TASK-39). Without the wait, a conversion that ends at once replaces the
  // stop announcement before NVDA reads it, and NVDA reads the error block (an
  // alert) at once, cutting into what it is saying. Held reports keep their
  // order; reports that come later are made at once.
  const STOP_READ_MS = 3000;
  let holdUntil = 0;
  let held: (() => void)[] = [];
  let holdTimer: ReturnType<typeof setTimeout> | undefined;

  function releaseHeldLater() {
    clearTimeout(holdTimer);
    holdTimer = setTimeout(() => {
      const reports = held;
      held = [];
      for (const report of reports) report();
    }, Math.max(0, holdUntil - performance.now()));
  }

  function holdConversionReports() {
    holdUntil = performance.now() + STOP_READ_MS;
    if (held.length) releaseHeldLater();
  }

  function reportConversion(report: () => void) {
    if (performance.now() >= holdUntil && !held.length) {
      report();
      return;
    }
    held.push(report);
    releaseHeldLater();
  }

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
    if (state === "Recording" && previous === "Paused") announce(m.live_recording_resumed());
    else if (state === "Recording") announce(m.live_recording_started());
    else if (state === "Paused") announce(m.live_recording_paused());
    else if (state === "Idle") {
      // With a compressed format the same announcement says what happens to
      // the recording next: two announcements in a row could lose the first.
      const next = untrack(() => recording.stoppedConversion);
      if (previous !== undefined && next.files > 0 && next.format) {
        announce(next.ahead > 0
          ? m.live_recording_stopped_conversion_waiting({ format: next.format })
          : m.live_recording_stopped_conversion_started({ format: next.format }));
      } else {
        announce(m.live_recording_stopped());
      }
      // What the converter reports on this recording comes after the stop:
      // the backend sends the new state right after it queues the files (a
      // stop from the window or the tray menu runs on the main thread, and
      // the converter's events wait for it).
      if (previous !== undefined) holdConversionReports();
    }
  });

  // Announces that the queue of conversions has emptied, after the stop
  // announcement when the recording has just stopped. A recording that could
  // not be converted has been reported in the error block by then, but NVDA
  // drops that alert when it arrives among focus changes (a failure right
  // after the window opens), so this announcement says by itself that the
  // recordings are kept.
  let wasConverting = false;
  $effect(() => {
    const busy = conversion.busy;
    if (!initialized) return;
    if (wasConverting && !busy) {
      const failed = untrack(() => conversion.failed);
      const text = failed > 0
        ? m.live_conversion_finished_with_errors({ count: failed })
        : m.live_conversion_finished();
      reportConversion(() => announce(text));
    }
    wasConverting = busy;
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
    confirmExitRecording = isRecording;
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
        if (recording.state !== "Idle") break;
        e.preventDefault();
        // Without a device the profile needs, say why nothing starts: the same
        // hint as the name of the disabled Start button. Focus stays put.
        if (recording.canRecord) startRecording();
        else announce(recording.readinessHint);
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
      case "i": {
        e.preventDefault();
        const status = recording.state !== "Idle"
          ? m.live_status_info({
              state: recording.state === "Paused" ? m.status_paused() : m.status_recording(),
              duration: formatDuration(recording.durationMs),
            })
          : m.status_ready();
        // While recordings are converted: that it goes on, about how long all
        // of it will take, and how many files wait.
        announce(conversion.busy && conversion.format
          ? `${status}. ${conversionInfo(conversion.format, conversion.etaSeconds, conversion.waiting)}`
          : status);
        break;
      }
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

  // Exits the app. A running recording is stopped first. Recordings that are
  // not converted yet stay in the queue and are converted at the next start.
  async function exitNow() {
    confirmExitOpen = false;
    if (isRecording) {
      try { await stopRecording(); } catch { /* already stopped */ }
    }
    await api.quitApp();
  }

  // The close button, Alt+F4 and Quit in the tray menu. While a recording runs
  // or recordings are still converted the user is asked first, unless the
  // confirmation is turned off in the settings.
  async function requestExit() {
    if (!isRecording && !conversion.busy) {
      void api.quitApp();
    } else if (appSettings.confirmExitDuringRecording) {
      openConfirmExit();
    } else {
      await exitNow();
    }
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
      try {
        const keptAt = await api.takeSettingsNotice();
        if (keptAt) showError(m.error_settings_unreadable({ path: keptAt }));
      } catch { /* no notice */ }
      await loadSettingsIntoStore();
      await loadProfiles();
      const active = profileStore.active;
      if (active) applyProfile(active);
      await refreshDevices();
      await initRecordingListener();
      await initDeviceListener();
      await initConversionListener((failure) => {
        const what = { format: failure.format, path: failure.wav };
        // Without the encoder (Windows N without the Media Feature Pack) the
        // message says how to get it.
        const message = failure.noEncoder ? m.error_conversion_no_encoder(what) : m.error_conversion_failed(what);
        reportConversion(() => showError(message));
      });

      await appWindow.onCloseRequested(async (event) => {
        event.preventDefault();
        await requestExit();
      });

      unlistenFocus = await appWindow.onFocusChanged(({ payload: focused }) => {
        if (!focused
            && appSettings.minimizeToTrayOnFocusLoss
            && !settingsOpen && !dialogOpen && !confirmExitOpen && !deleteConfirmOpen && !aboutOpen) {
          void appWindow.hide();
        }
      });

      await listen("tray-about-requested", () => openAbout());

      unlistenTrayQuit = await listen("tray-quit-requested", () => requestExit());

      initialized = true;
      await tick();
      document.querySelector<HTMLSelectElement>('.profile-controls select')?.focus();

      // The window listens to the conversion events now, so conversions may
      // run. Recordings the last run left unconverted come first; the user is
      // told before they start, so "finished" cannot come before it.
      try {
        const leftOver = await api.conversionsLeftOver();
        if (leftOver > 0) announce(m.live_conversion_resumed({ count: leftOver }));
        await api.startConversions();
      } catch { /* the recordings stay in the queue as WAV files */ }
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
<!-- A plain block, not a <main> landmark: inside role="application" a landmark is of no use,
     and NVDA would name it every time focus enters it (backlog TASK-18). -->
<div class="main">
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

  <StatusIndicator
    state={recording.state}
    durationMs={recording.durationMs}
    conversion={conversionLine}
  />

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
    recording={confirmExitRecording}
    onexit={exitNow}
    oncancel={async () => { confirmExitOpen = false; await returnFocus("exit"); }}
  />

  <ConfirmDeleteDialog
    open={deleteConfirmOpen}
    profileName={deletingProfileName}
    onconfirm={handleDeleteConfirm}
    oncancel={handleDeleteCancel}
  />
</div>
{/if}

<style>
  .main {
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
