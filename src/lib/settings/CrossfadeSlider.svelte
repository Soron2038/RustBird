<script lang="ts">
  import { setCrossfadeDuration } from '$lib/commands';
  import { onMount } from 'svelte';

  let {
    duration,
  }: {
    duration: number;
  } = $props();

  let cfDuration = $state(2.0);

  onMount(() => {
    cfDuration = duration;
  });

  async function updateCrossfade(value: number) {
    cfDuration = value;
    await setCrossfadeDuration(value);
  }
</script>

<div class="setting-row">
  <span>Crossfade</span>
  <div class="cf-control">
    <input
      type="range"
      min="50"
      max="500"
      value={Math.round(cfDuration * 100)}
      class="slider-track cf-slider"
      oninput={(e) => updateCrossfade(parseInt((e.target as HTMLInputElement).value) / 100)}
    />
    <span class="cf-value">{cfDuration.toFixed(1)}s</span>
  </div>
</div>

<style>
  .setting-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 0;
    border-bottom: 0.5px solid var(--separator);
    font-size: 12px;
  }
  .cf-control {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .cf-slider {
    width: 80px;
  }
  .cf-value {
    font-size: 11px;
    color: var(--text-secondary);
    width: 30px;
  }
</style>
