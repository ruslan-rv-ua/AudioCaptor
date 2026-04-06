<script lang="ts">
  import * as m from "../../paraglide/messages";

  interface Props {
    open: boolean;
    onstopandexit: () => void;
    oncancel: () => void;
  }

  let { open, onstopandexit, oncancel }: Props = $props();

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") { oncancel(); return; }
    if (e.key === "Tab") {
      const dialog = e.currentTarget as HTMLElement;
      const focusable = dialog.querySelectorAll<HTMLElement>("button");
      if (!focusable.length) return;
      const first = focusable[0];
      const last = focusable[focusable.length - 1];
      if (e.shiftKey && document.activeElement === first) { e.preventDefault(); last.focus(); }
      else if (!e.shiftKey && document.activeElement === last) { e.preventDefault(); first.focus(); }
    }
  }

  $effect(() => {
    if (open) requestAnimationFrame(() => document.getElementById("confirm-exit-cancel")?.focus());
  });
</script>

{#if open}
  <div class="dialog-backdrop">
    <div
      class="dialog"
      role="alertdialog"
      aria-modal="true"
      aria-labelledby="confirm-exit-title"
      aria-describedby="confirm-exit-body"
      tabindex="-1"
      onkeydown={handleKeydown}
    >
      <h2 id="confirm-exit-title">{m.confirm_exit_title()}</h2>
      <p id="confirm-exit-body">{m.confirm_exit_body()}</p>
      <div class="actions">
        <button id="confirm-exit-cancel" type="button" class="btn-secondary" onclick={oncancel}>
          {m.btn_cancel()}
        </button>
        <button type="button" class="btn-danger" onclick={onstopandexit}>
          {m.btn_stop_and_exit()}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .dialog-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 200;
  }
  .dialog {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 10px;
    padding: 22px 24px 20px;
    width: 340px;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
    color: var(--text-primary);
  }
  h2 { margin: 0 0 8px; font-size: 1.05rem; color: var(--text-primary); }
  p  { margin: 0 0 20px; font-size: 0.9rem; color: var(--text-secondary); }
  .actions { display: flex; gap: 8px; justify-content: flex-end; }
  .btn-secondary {
    padding: 8px 16px; border: 1px solid var(--border); border-radius: 6px;
    font-size: 0.875rem; font-weight: 600; cursor: pointer;
    background: var(--surface); color: var(--text-primary);
  }
  .btn-secondary:hover { background: var(--surface-hover); }
  .btn-danger {
    padding: 8px 16px; border: none; border-radius: 6px;
    font-size: 0.875rem; font-weight: 600; cursor: pointer;
    background: var(--btn-stop-bg); color: #ffffff;
  }
  .btn-danger:hover { filter: brightness(1.1); }
</style>
