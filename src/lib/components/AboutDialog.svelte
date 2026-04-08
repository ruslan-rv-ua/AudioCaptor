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
  let dialogEl   = $state<HTMLDivElement | undefined>();
  let headingEl  = $state<HTMLHeadingElement | undefined>();

  $effect(() => {
    if (open) {
      getVersion().then(v => { appVersion = v; });
      // APG: for dialogs with semantic structure (dl, ol), focus a static
      // element at content start so screen readers can navigate naturally.
      requestAnimationFrame(() => headingEl?.focus());
    }
  });

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") { e.preventDefault(); e.stopPropagation(); onclose(); return; }
    if (e.key === "Enter")  { e.preventDefault(); e.stopPropagation(); onclose(); return; }
    if (e.key === "Tab") {
      // Tab cycle only over tabbable elements (not tabindex="-1" heading)
      const focusable = dialogEl?.querySelectorAll<HTMLElement>(
        'button:not([disabled]), [tabindex="0"]'
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
      <!-- role="document" lets NVDA/JAWS use browse mode (arrow-key navigation)
           even after programmatic focus inside role="dialog".
           The heading stays tabindex="-1": focusable by JS, not by Tab. -->
      <div role="document">
        <!-- tabindex="-1": programmatically focusable but not in Tab cycle -->
        <!-- APG: focus static heading so screen readers announce dialog context first -->
        <h2 id="about-dialog-title" tabindex="-1" bind:this={headingEl}>{m.about_title()}</h2>

        <div class="version-row">
          <span class="app-name">AudioCaptor</span>
          <span class="version-badge">v{appVersion}</span>
        </div>

        <div class="divider"></div>

        <!-- Keyboard Shortcuts -->
        <section>
          <h3 class="section-label">{m.about_hotkeys()}</h3>

          <p class="subsection-label">{m.about_hotkey_global()}</p>
          {#if hotkey}
            <dl class="hotkey-dl">
              <dt><kbd>{hotkey}</kbd> &ndash; {m.about_hotkey_short_press()}</dt>
              <dd>{m.about_hotkey_start_pause()}</dd>
              <dt><kbd>{hotkey}</kbd> &ndash; {m.about_hotkey_long_press()}</dt>
              <dd>{m.about_hotkey_stop()}</dd>
            </dl>
          {:else}
            <p class="hotkey-unset">{m.settings_hotkey_none()}</p>
          {/if}

          <p class="subsection-label">{m.about_in_app_shortcuts()}</p>
          <dl class="hotkey-dl">
            <dt><kbd>Alt+S</kbd></dt><dd>{m.btn_start()}</dd>
            <dt><kbd>Alt+P</kbd></dt><dd>{m.btn_pause()}</dd>
            <dt><kbd>Alt+R</kbd></dt><dd>{m.btn_resume()}</dd>
            <dt><kbd>Alt+T</kbd></dt><dd>{m.btn_stop()}</dd>
            <dt><kbd>Alt+I</kbd></dt><dd>{m.about_shortcut_status()}</dd>
            <dt><kbd>Escape</kbd></dt><dd>{m.about_shortcut_minimize()}</dd>
          </dl>
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
      </div>

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

  .hotkey-dl {
    display: grid;
    grid-template-columns: auto 1fr;
    column-gap: 12px;
    row-gap: 3px;
    margin: 0 0 4px;
    font-size: 0.825rem;
  }

  .hotkey-dl dt {
    margin: 0;
    color: var(--text-secondary);
    white-space: nowrap;
  }

  .hotkey-dl dd {
    margin: 0;
    color: var(--text-primary);
  }

  .hotkey-unset {
    margin: 0 0 4px;
    font-size: 0.825rem;
    color: var(--text-muted);
    font-style: italic;
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
