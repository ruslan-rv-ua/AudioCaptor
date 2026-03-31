<script lang="ts">
  import type { RecordingState } from "../types";

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
    state === "Idle" ? "Ready" : state === "Recording" ? "Recording" : "Paused"
  );

  let stateClass = $derived(state.toLowerCase());
</script>

<div
  class="status-indicator"
  role="status"
  aria-live="polite"
  aria-label={state !== "Idle" ? `Recording status: ${stateLabel}, duration ${formattedDuration}` : `Recording status: ${stateLabel}`}
>
  <span class="state-badge {stateClass}">{stateLabel}</span>
  {#if state !== "Idle"}
    <span class="duration" aria-label="Recording duration">{formattedDuration}</span>
  {/if}
</div>

<style>
  .status-indicator {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 12px;
    padding: 12px;
    font-size: 1.25rem;
  }
  .state-badge {
    padding: 4px 12px;
    border-radius: 4px;
    font-weight: 600;
    font-size: 0.875rem;
    text-transform: uppercase;
  }
  .idle { background: #e5e7eb; color: #374151; }
  .recording { background: #fecaca; color: #dc2626; }
  .paused { background: #fef3c7; color: #d97706; }
  .duration {
    font-family: monospace;
    font-size: 1.5rem;
    font-weight: 700;
  }
</style>
