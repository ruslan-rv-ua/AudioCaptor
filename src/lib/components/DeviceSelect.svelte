<script lang="ts">
  import type { AudioDevice } from "../types";

  interface Props {
    label: string;
    devices: AudioDevice[];
    value: string | null;
    onchange: (id: string | null) => void;
    disabled?: boolean;
  }

  let { label, devices, value, onchange, disabled = false }: Props = $props();

  function handleChange(e: Event) {
    const target = e.target as HTMLSelectElement;
    onchange(target.value || null);
  }
</script>

<div class="device-select">
  <label for={label.toLowerCase().replace(/\s/g, "-")}>
    {label}
  </label>
  <select
    id={label.toLowerCase().replace(/\s/g, "-")}
    value={value ?? ""}
    {disabled}
    onchange={handleChange}
  >
    <option value="">-- Select {label} --</option>
    {#each devices as device}
      <option value={device.id} selected={device.id === value}>
        {device.name}
      </option>
    {/each}
  </select>
</div>

<style>
  .device-select {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  label {
    font-weight: 600;
    font-size: 0.875rem;
  }
  select {
    padding: 8px;
    border: 1px solid #ccc;
    border-radius: 4px;
    font-size: 0.875rem;
    background: #fff;
  }
  select:focus-visible {
    outline: 2px solid #0066cc;
    outline-offset: 2px;
  }
  select:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }
</style>
