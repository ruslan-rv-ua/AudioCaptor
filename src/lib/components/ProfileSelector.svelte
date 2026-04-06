<script lang="ts">
  import type { RecordingProfile } from "../types";
  import * as m from "../../paraglide/messages";

  interface Props {
    profiles: RecordingProfile[];
    activeId: string;
    disabled?: boolean;
    onselect: (id: string) => void;
    oncreate: () => void;
    onedit: (profile: RecordingProfile) => void;
    ondelete: (id: string) => void;
  }

  let { profiles, activeId, disabled = false, onselect, oncreate, onedit, ondelete }: Props = $props();

  let activeProfile = $derived(profiles.find(p => p.id === activeId));
  let canDelete = $derived(profiles.length > 1);

  function handleChange(e: Event) {
    const target = e.target as HTMLSelectElement;
    onselect(target.value);
  }
</script>

<div class="profile-controls">
  <select
    aria-label={m.profile_section()}
    {disabled}
    onchange={handleChange}
  >
    {#each profiles as profile}
      <option value={profile.id} selected={profile.id === activeId}>
        {profile.name}
      </option>
    {/each}
  </select>
  <button
    type="button"
    class="btn-icon"
    aria-label={m.create_profile_aria()}
    {disabled}
    onclick={oncreate}
  >+</button>
  <button
    type="button"
    class="btn-icon"
    aria-label={m.edit_profile_aria()}
    {disabled}
    onclick={() => activeProfile && onedit(activeProfile)}
  >✎</button>
  <button
    type="button"
    class="btn-icon btn-danger"
    aria-label={m.delete_profile_aria()}
    disabled={disabled || !canDelete}
    onclick={() => ondelete(activeId)}
  >✕</button>
</div>

<style>
  .profile-controls {
    display: flex;
    gap: 4px;
    align-items: center;
  }
  .profile-controls select {
    flex: 1;
    padding: 6px 8px;
    border: 1px solid var(--border);
    border-radius: 5px;
    font-size: 0.875rem;
    background: var(--surface);
    color: var(--text-primary);
    appearance: none;
    background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='10' height='6'%3E%3Cpath d='M0 0l5 6 5-6z' fill='%23888'/%3E%3C/svg%3E");
    background-repeat: no-repeat;
    background-position: right 8px center;
    padding-right: 24px;
  }
  .profile-controls select:focus-visible {
    outline: 2px solid var(--focus-ring);
    outline-offset: 2px;
  }
  .btn-icon {
    width: 30px;
    height: 30px;
    border: 1px solid var(--border);
    border-radius: 5px;
    background: var(--surface);
    color: var(--text-primary);
    cursor: pointer;
    font-size: 1rem;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }
  .btn-icon:hover:not(:disabled) {
    background: var(--surface-hover);
  }
  .btn-icon:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
  .btn-danger:hover:not(:disabled) {
    color: var(--error-text);
    border-color: var(--error-border);
  }
</style>
