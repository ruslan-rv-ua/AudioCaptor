<script lang="ts">
  interface Props {
    label: string;
    value: number;
    onchange: (value: number) => void;
    min?: number;
    max?: number;
    step?: number;
    disabled?: boolean;
  }

  let {
    label,
    value,
    onchange,
    min = 0,
    max = 4.0,
    step = 0.1,
    disabled = false,
  }: Props = $props();

  function handleInput(e: Event) {
    const target = e.target as HTMLInputElement;
    onchange(parseFloat(target.value));
  }
</script>

<div class="volume-slider">
  <label for={label.toLowerCase().replace(/\s/g, "-")}>
    {label}: {value.toFixed(1)}
  </label>
  <input
    id={label.toLowerCase().replace(/\s/g, "-")}
    type="range"
    aria-valuemin={min}
    aria-valuemax={max}
    aria-valuenow={value}
    {min}
    {max}
    {step}
    {value}
    {disabled}
    oninput={handleInput}
  />
</div>

<style>
  .volume-slider {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  label {
    font-weight: 600;
    font-size: 0.875rem;
  }
  input[type="range"] {
    width: 100%;
    cursor: pointer;
  }
  input[type="range"]:focus-visible {
    outline: 2px solid #0066cc;
    outline-offset: 2px;
  }
  input[type="range"]:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }
</style>
