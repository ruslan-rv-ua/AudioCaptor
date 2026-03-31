<script lang="ts">
  import { onMount } from "svelte";
  import type { RecordingState } from "../types";

  interface Props {
    recordingState: RecordingState;
    onstart: () => void;
    onpause: () => void;
    onresume: () => void;
    onstop: () => void;
  }

  let { recordingState, onstart, onpause, onresume, onstop }: Props = $props();

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
      onclick={onstart}
      aria-label="Start recording (Alt+S)"
      class="btn btn-start"
    >
      {#if altPressed}<u>S</u>tart{:else}Start{/if}
    </button>
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
  .btn-start:hover { background: #16a34a; }
  .btn-pause { background: #f59e0b; color: white; }
  .btn-pause:hover { background: #d97706; }
  .btn-resume { background: #3b82f6; color: white; }
  .btn-resume:hover { background: #2563eb; }
  .btn-stop { background: #ef4444; color: white; }
  .btn-stop:hover { background: #dc2626; }
</style>
