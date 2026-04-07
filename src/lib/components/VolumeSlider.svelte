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

  // Percentage for the filled-track gradient defined in app.css
  let fillPct = $derived(((value - min) / (max - min)) * 100);

  let inputId = $derived(label.toLowerCase().replace(/\s/g, "-"));

  function handleInput(e: Event) {
    const target = e.target as HTMLInputElement;
    onchange(parseFloat(target.value));
  }
</script>

<div class="volume-slider">
  <label for={inputId}>
    {label}
    <span class="value-display" aria-hidden="true">{value.toFixed(1)}</span>
  </label>
  <input
    id={inputId}
    type="range"
    {min}
    {max}
    {step}
    {value}
    {disabled}
    style="--fill: {fillPct}%"
    oninput={handleInput}
  />
</div>

<style>
  .volume-slider {
    display: flex;
    flex-direction: column;
    gap: 3px;
  }
  label {
    font-weight: 600;
    font-size: 0.8rem;
    color: var(--text-secondary);
    display: flex;
    justify-content: space-between;
    align-items: baseline;
  }
  .value-display {
    font-weight: 700;
    color: var(--accent);
    font-size: 0.8rem;
    min-width: 2.5ch;
    text-align: right;
  }
  /* input[type="range"] appearance is handled globally in app.css */
</style>
