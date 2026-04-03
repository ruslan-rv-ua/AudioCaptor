<script lang="ts">
  import type { RecordingProfile } from "../types";

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

<section aria-label="Recording profile">
  <div class="profile-row">
    <label for="profile-select">Profile</label>
    <div class="profile-controls">
      <select
        id="profile-select"
        aria-label="Recording profile"
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
        aria-label="Create new profile"
        {disabled}
        onclick={oncreate}
      >+</button>
      <button
        type="button"
        class="btn-icon"
        aria-label="Edit profile"
        {disabled}
        onclick={() => activeProfile && onedit(activeProfile)}
      >✎</button>
      <button
        type="button"
        class="btn-icon btn-danger"
        aria-label="Delete profile"
        disabled={disabled || !canDelete}
        onclick={() => ondelete(activeId)}
      >✕</button>
    </div>
  </div>
</section>

<style>
  .profile-row {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .profile-row label {
    font-weight: 600;
    font-size: 0.875rem;
  }

  .profile-controls {
    display: flex;
    gap: 4px;
    align-items: center;
  }

  .profile-controls select {
    flex: 1;
    padding: 8px;
    border: 1px solid #ccc;
    border-radius: 4px;
    font-size: 0.875rem;
    background: #fff;
  }

  .profile-controls select:focus-visible {
    outline: 2px solid #0066cc;
    outline-offset: 2px;
  }

  .btn-icon {
    width: 32px;
    height: 32px;
    border: 1px solid #ccc;
    border-radius: 4px;
    background: #fff;
    cursor: pointer;
    font-size: 1rem;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .btn-icon:hover:not(:disabled) { background: #f0f0f0; }
  .btn-icon:disabled { opacity: 0.4; cursor: not-allowed; }
  .btn-danger:hover:not(:disabled) { background: #fef2f2; color: #dc2626; }
</style>
