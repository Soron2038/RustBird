<script lang="ts">
  import { enable, disable } from '@tauri-apps/plugin-autostart';
  import { onMount } from 'svelte';

  let {
    enabled,
  }: {
    enabled: boolean;
  } = $props();

  let autostart = $state(false);

  onMount(() => {
    autostart = enabled;
  });

  async function toggle() {
    if (autostart) {
      await disable();
    } else {
      await enable();
    }
    autostart = !autostart;
  }
</script>

<div class="setting-row">
  <span>Autostart</span>
  <div
    class="toggle-pill"
    class:active={autostart}
    onclick={toggle}
    onkeydown={(e) => {
      if (e.key === 'Enter' || e.key === ' ') {
        e.preventDefault();
        toggle();
      }
    }}
    role="switch"
    aria-checked={autostart}
    tabindex="0"
  ></div>
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
  .toggle-pill {
    position: relative;
    width: 38px;
    height: 22px;
    border-radius: 11px;
    background: var(--toggle-bg);
    cursor: pointer;
    flex-shrink: 0;
    transition: background 0.2s ease;
  }
  .toggle-pill::after {
    content: '';
    position: absolute;
    top: 2px;
    left: 2px;
    width: 18px;
    height: 18px;
    background: var(--toggle-knob);
    border-radius: 50%;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.35);
    transition:
      left 0.2s ease,
      background 0.2s ease;
  }
  .toggle-pill.active {
    background: var(--toggle-active-bg);
  }
  .toggle-pill.active::after {
    left: 18px;
    background: var(--toggle-active-knob);
  }
</style>
