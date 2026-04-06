<script lang="ts">
  import type { AudioDevice } from "../types";
  import * as m from "../../paraglide/messages";

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
    <option value="">{m.select_device_placeholder({ label })}</option>
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
    font-size: 0.8rem;
    color: var(--text-secondary);
  }
  select {
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
  select:focus-visible {
    outline: 2px solid var(--focus-ring);
    outline-offset: 2px;
  }
  select:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
