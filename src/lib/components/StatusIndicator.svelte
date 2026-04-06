<script lang="ts">
  import type { RecordingState } from "../types";
  import * as m from "../../paraglide/messages";

  interface Props {
    state: RecordingState;
    durationMs: number;
  }

  let { state, durationMs }: Props = $props();

  let formattedDuration = $derived(formatDuration(durationMs));

  function formatDuration(ms: number): string {
    const totalSeconds = Math.floor(ms / 1000);
    const hours = Math.floor(totalSeconds / 3600);
    const minutes = Math.floor((totalSeconds % 3600) / 60);
    const seconds = totalSeconds % 60;
    const pad = (n: number) => n.toString().padStart(2, "0");
    return `${pad(hours)}:${pad(minutes)}:${pad(seconds)}`;
  }

  let stateLabel = $derived(
    state === "Idle"      ? m.status_ready()     :
    state === "Recording" ? m.status_recording() : m.status_paused()
  );
</script>

<div
  class="status-card"
  class:recording={state === "Recording"}
  class:paused={state === "Paused"}
  aria-label={m.recording_status()}
>
  <div class="state-row">
    {#if state === "Recording"}
      <span class="rec-dot" aria-hidden="true"></span>
    {/if}
    <span class="state-label">{stateLabel}</span>
  </div>
  <span
    class="timer"
    class:timer-idle={state === "Idle"}
    role="timer"
    aria-label={m.recording_duration({ duration: formattedDuration })}
  >{formattedDuration}</span>
</div>

<style>
  .status-card {
    background: var(--surface);
    border: 2px solid transparent;
    border-radius: 10px;
    padding: 10px 14px 12px;
    text-align: center;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
    box-shadow: var(--surface-shadow);
    transition: border-color 0.2s, box-shadow 0.2s;
  }

  .status-card.recording {
    border-color: var(--status-rec-border);
    box-shadow: 0 0 0 3px var(--status-rec-glow);
  }

  .status-card.paused {
    border-color: var(--status-paused-border);
    box-shadow: 0 0 0 3px var(--status-paused-glow);
  }

  .state-row {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
  }

  .state-label {
    font-size: 8.5px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.1em;
    color: var(--text-muted);
  }

  .status-card.recording .state-label {
    color: var(--status-rec-border);
  }

  .status-card.paused .state-label {
    color: var(--status-paused-border);
  }

  /* Blinking recording dot */
  .rec-dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--status-rec-border);
    animation: blink 1.2s ease-in-out infinite;
  }

  @keyframes blink {
    0%, 100% { opacity: 1; }
    50%       { opacity: 0.3; }
  }

  .timer {
    font-family: 'Courier New', monospace;
    font-size: 26px;
    font-weight: 700;
    color: var(--text-primary);
    line-height: 1;
  }

  .timer.timer-idle {
    color: var(--text-muted);
  }
</style>
