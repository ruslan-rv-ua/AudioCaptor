<script lang="ts">
  import type { RecordingState } from "../types";
  import * as m from "../../paraglide/messages";

  interface Props {
    recordingState: RecordingState;
    canRecord: boolean;
    needsMic: boolean;
    needsLoopback: boolean;
    onstart: () => void;
    onpause: () => void;
    onresume: () => void;
    onstop: () => void;
  }

  let { recordingState, canRecord, needsMic, needsLoopback, onstart, onpause, onresume, onstop }: Props = $props();

  let readinessHint = $derived(
    needsMic && needsLoopback ? m.hint_select_both() :
    needsMic ? m.hint_select_mic() :
    needsLoopback ? m.hint_select_loopback() : ""
  );
</script>

<div class="record-controls" role="group" aria-label={m.recording_controls()}>
  {#if recordingState === "Idle"}
    <button
      type="button"
      onclick={canRecord ? onstart : undefined}
      aria-disabled={!canRecord || undefined}
      aria-label={canRecord
        ? m.start_recording_aria()
        : m.start_recording_disabled_aria({ hint: readinessHint })}
      class="btn btn-start"
      class:disabled={!canRecord}
    >
      {m.btn_start()}
    </button>
    {#if !canRecord}
      <p id="start-hint" class="hint" role="note">{readinessHint}</p>
    {/if}
  {:else if recordingState === "Recording"}
    <button type="button" onclick={onpause} aria-label={m.pause_recording_aria()} class="btn btn-pause">
      {m.btn_pause()}
    </button>
    <button type="button" onclick={onstop} aria-label={m.stop_recording_aria()} class="btn btn-stop">
      {m.btn_stop()}
    </button>
  {:else if recordingState === "Paused"}
    <button type="button" onclick={onresume} aria-label={m.resume_recording_aria()} class="btn btn-resume">
      {m.btn_resume()}
    </button>
    <button type="button" onclick={onstop} aria-label={m.stop_recording_aria()} class="btn btn-stop">
      {m.btn_stop()}
    </button>
  {/if}
</div>

<style>
  .record-controls {
    display: flex;
    gap: 8px;
    justify-content: center;
    flex-wrap: wrap;
    padding: 8px 0 4px;
  }
  .btn {
    flex: 1;
    padding: 11px 20px;
    border: none;
    border-radius: 8px;
    font-size: 0.9rem;
    font-weight: 700;
    cursor: pointer;
    min-width: 90px;
    color: #ffffff;
  }
  .btn:focus-visible {
    outline: 2px solid var(--focus-ring);
    outline-offset: 2px;
  }
  .btn-start { background: var(--btn-start-bg); }
  .btn-start:hover:not(.disabled) { filter: brightness(1.1); }
  .btn-start.disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  .hint {
    width: 100%;
    text-align: center;
    font-size: 0.8rem;
    color: var(--text-muted);
    margin: 2px 0 0;
  }
  .btn-pause  { background: var(--btn-pause-bg); }
  .btn-pause:hover  { filter: brightness(1.1); }
  .btn-resume { background: var(--btn-resume-bg); }
  .btn-resume:hover { filter: brightness(1.1); }
  .btn-stop   { background: var(--btn-stop-bg); }
  .btn-stop:hover   { filter: brightness(1.1); }
</style>
