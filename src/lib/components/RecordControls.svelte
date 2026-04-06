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
    padding: 16px 0;
  }
  .btn {
    padding: 12px 24px;
    border: none;
    border-radius: 6px;
    font-size: 1rem;
    font-weight: 600;
    cursor: pointer;
    min-width: 100px;
  }
  .btn:focus-visible {
    outline: 2px solid #0066cc;
    outline-offset: 2px;
  }
  .btn-start { background: #22c55e; color: white; }
  .btn-start:hover:not(.disabled) { background: #16a34a; }
  .btn-start.disabled {
    opacity: 0.5;
    cursor: not-allowed;
    background: #86efac;
  }
  .hint {
    width: 100%;
    text-align: center;
    font-size: 0.8rem;
    color: #6b7280;
    margin: 4px 0 0;
  }
  .btn-pause { background: #f59e0b; color: white; }
  .btn-pause:hover { background: #d97706; }
  .btn-resume { background: #3b82f6; color: white; }
  .btn-resume:hover { background: #2563eb; }
  .btn-stop { background: #ef4444; color: white; }
  .btn-stop:hover { background: #dc2626; }
</style>
