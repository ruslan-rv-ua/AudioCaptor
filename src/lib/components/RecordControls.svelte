<script lang="ts">
  import { onMount } from "svelte";
  import type { RecordingState } from "../types";

  interface Props {
    recordingState: RecordingState;
    canRecord: boolean;
    readinessHint: string;
    onstart: () => void;
    onpause: () => void;
    onresume: () => void;
    onstop: () => void;
  }

  let { recordingState, canRecord, readinessHint, onstart, onpause, onresume, onstop }: Props = $props();

  let altPressed = $state(false);

  onMount(() => {
    function onKeyDown(e: KeyboardEvent) { if (e.key === "Alt") altPressed = true; }
    function onKeyUp(e: KeyboardEvent) { if (e.key === "Alt") altPressed = false; }
    window.addEventListener("keydown", onKeyDown);
    window.addEventListener("keyup", onKeyUp);
    return () => {
      window.removeEventListener("keydown", onKeyDown);
      window.removeEventListener("keyup", onKeyUp);
    };
  });
</script>

<div class="record-controls" role="group" aria-label="Recording controls">
  {#if recordingState === "Idle"}
    <button
      type="button"
      onclick={canRecord ? onstart : undefined}
      aria-disabled={!canRecord || undefined}
      aria-label={canRecord
        ? "Start recording (Alt+S)"
        : `Start recording (Alt+S). ${readinessHint}`}
      class="btn btn-start"
      class:disabled={!canRecord}
    >
      {#if altPressed}<u>S</u>tart{:else}Start{/if}
    </button>
    {#if !canRecord}
      <p id="start-hint" class="hint" role="note">{readinessHint}</p>
    {/if}
  {:else if recordingState === "Recording"}
    <button
      type="button"
      onclick={onpause}
      aria-label="Pause recording (Alt+P)"
      class="btn btn-pause"
    >
      {#if altPressed}<u>P</u>ause{:else}Pause{/if}
    </button>
    <button
      type="button"
      onclick={onstop}
      aria-label="Stop recording (Alt+T)"
      class="btn btn-stop"
    >
      S{#if altPressed}<u>t</u>op{:else}top{/if}
    </button>
  {:else if recordingState === "Paused"}
    <button
      type="button"
      onclick={onresume}
      aria-label="Resume recording (Alt+R)"
      class="btn btn-resume"
    >
      {#if altPressed}<u>R</u>esume{:else}Resume{/if}
    </button>
    <button
      type="button"
      onclick={onstop}
      aria-label="Stop recording (Alt+T)"
      class="btn btn-stop"
    >
      S{#if altPressed}<u>t</u>op{:else}top{/if}
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
