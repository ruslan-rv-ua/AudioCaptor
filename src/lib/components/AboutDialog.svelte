<script lang="ts">
  import { getVersion } from "@tauri-apps/api/app";
  import * as m from "../../paraglide/messages";

  interface Props {
    open: boolean;
    hotkey: string | null;
    onclose: () => void;
  }

  let { open, hotkey, onclose }: Props = $props();

  let appVersion = $state("…");
  let dialogEl = $state<HTMLDivElement | undefined>();

  $effect(() => {
    if (open) {
      getVersion().then(v => { appVersion = v; });
      requestAnimationFrame(() =>
        dialogEl?.querySelector<HTMLElement>(".btn-close")?.focus()
      );
    }
  });

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") { e.preventDefault(); onclose(); return; }
    if (e.key === "Enter")  { e.preventDefault(); onclose(); return; }
    if (e.key === "Tab") {
      const focusable = dialogEl?.querySelectorAll<HTMLElement>(
        'button, [tabindex]:not([tabindex="-1"])'
      );
      if (!focusable?.length) return;
      const first = focusable[0];
      const last  = focusable[focusable.length - 1];
      if (e.shiftKey && document.activeElement === first) { e.preventDefault(); last.focus(); }
      else if (!e.shiftKey && document.activeElement === last) { e.preventDefault(); first.focus(); }
    }
  }
</script>

{#if open}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="dialog-backdrop" onkeydown={handleKeydown}>
    <div
      class="dialog"
      role="dialog"
      aria-modal="true"
      aria-labelledby="about-dialog-title"
      tabindex="-1"
      bind:this={dialogEl}
    >
      <h2 id="about-dialog-title">{m.about_title()}</h2>

      <div class="version-row">
        <span class="app-name">AudioCaptor</span>
        <span class="version-badge">v{appVersion}</span>
      </div>

      <div class="divider"></div>

      <!-- Keyboard Shortcuts -->
      <section>
        <h3 class="section-label">{m.about_hotkeys()}</h3>

        <p class="subsection-label">{m.about_hotkey_global()}</p>
        <table class="hotkey-table">
          <tbody>
            <tr>
              <td class="key-cell">{hotkey ?? m.settings_hotkey_none()}</td>
              <td></td>
            </tr>
            <tr>
              <td class="key-cell indent">{m.about_hotkey_short_press()}</td>
              <td>{m.about_hotkey_start_pause()}</td>
            </tr>
            <tr>
              <td class="key-cell indent">{m.about_hotkey_long_press()}</td>
              <td>{m.about_hotkey_stop()}</td>
            </tr>
          </tbody>
        </table>

        <p class="subsection-label">{m.about_in_app_shortcuts()}</p>
        <table class="hotkey-table">
          <tbody>
            <tr><td class="key-cell"><kbd>Alt+S</kbd></td><td>{m.btn_start()}</td></tr>
            <tr><td class="key-cell"><kbd>Alt+P</kbd></td><td>{m.btn_pause()}</td></tr>
            <tr><td class="key-cell"><kbd>Alt+R</kbd></td><td>{m.btn_resume()}</td></tr>
            <tr><td class="key-cell"><kbd>Alt+T</kbd></td><td>{m.btn_stop()}</td></tr>
            <tr><td class="key-cell"><kbd>Alt+I</kbd></td><td>{m.about_shortcut_status()}</td></tr>
            <tr><td class="key-cell"><kbd>Escape</kbd></td><td>{m.about_shortcut_minimize()}</td></tr>
          </tbody>
        </table>
      </section>

      <div class="divider"></div>

      <!-- Quick Start -->
      <section>
        <h3 class="section-label">{m.about_quick_start()}</h3>
        <ol class="quick-start">
          <li>{m.about_step_select_devices()}</li>
          <li>{m.about_step_configure_profile()}</li>
          <li>{m.about_step_press_start()}</li>
          <li>{m.about_step_files_saved()}</li>
        </ol>
      </section>

      <div class="actions">
        <button type="button" class="btn-close btn-primary" onclick={onclose}>
          {m.btn_close()}
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
    z-index: 100;
  }

  .dialog {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 10px;
    padding: 22px 22px 20px;
    width: 400px;
    max-height: 85vh;
    overflow-y: auto;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
    color: var(--text-primary);
  }

  h2 {
    margin: 0 0 12px;
    font-size: 1.1rem;
    color: var(--text-primary);
  }

  .version-row {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 12px;
  }

  .app-name {
    font-size: 1rem;
    font-weight: 700;
    color: var(--text-primary);
  }

  .version-badge {
    font-size: 0.75rem;
    font-weight: 600;
    padding: 2px 7px;
    border-radius: 20px;
    background: var(--accent);
    color: #ffffff;
  }

  .divider {
    height: 1px;
    background: var(--border);
    margin: 2px 0 12px;
  }

  .section-label {
    margin: 0 0 8px;
    font-size: 0.8rem;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.07em;
    color: var(--text-secondary);
  }

  .subsection-label {
    margin: 8px 0 4px;
    font-size: 0.8rem;
    font-weight: 600;
    color: var(--text-muted);
  }

  .hotkey-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.825rem;
    margin-bottom: 4px;
  }

  .hotkey-table tr + tr td {
    padding-top: 3px;
  }

  .key-cell {
    width: 44%;
    padding-right: 8px;
    color: var(--text-secondary);
    vertical-align: top;
  }

  .key-cell.indent {
    padding-left: 12px;
  }

  kbd {
    display: inline-block;
    padding: 1px 5px;
    font-size: 0.775rem;
    font-family: ui-monospace, monospace;
    border: 1px solid var(--border);
    border-radius: 4px;
    background: var(--surface-hover);
    color: var(--text-primary);
    line-height: 1.5;
  }

  .quick-start {
    margin: 0;
    padding-left: 20px;
    font-size: 0.85rem;
    color: var(--text-primary);
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .actions {
    display: flex;
    justify-content: flex-end;
    margin-top: 16px;
  }

  .btn-primary {
    padding: 8px 18px;
    border: none;
    border-radius: 6px;
    font-size: 0.875rem;
    font-weight: 700;
    cursor: pointer;
    background: var(--accent);
    color: #ffffff;
  }

  .btn-primary:hover {
    background: var(--accent-hover);
  }
</style>
