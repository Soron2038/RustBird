<script>
  import { invoke } from '@tauri-apps/api/core';
  import { enable, disable, isEnabled } from '@tauri-apps/plugin-autostart';
  import { onMount } from 'svelte';

  let { autostartEnabled, crossfadeDuration, onBack } = $props();
  let autostart = $state(false);
  let cfDuration = $state(2.0);

  onMount(() => {
    autostart = autostartEnabled;
    cfDuration = crossfadeDuration;
  });

  async function toggleAutostart() {
    if (autostart) {
      await disable();
    } else {
      await enable();
    }
    autostart = !autostart;
  }

  async function updateCrossfade(value) {
    cfDuration = value;
    await invoke('set_crossfade_duration', { duration: value });
  }
</script>

<section class="settings">
  <button class="back-btn" onclick={onBack}>← Back</button>

  <div class="setting-row">
    <span>Autostart</span>
    <div
      class="toggle-pill"
      class:active={autostart}
      onclick={toggleAutostart}
      onkeydown={(e) => {
        if (e.key === 'Enter' || e.key === ' ') {
          e.preventDefault();
          toggleAutostart();
        }
      }}
      role="switch"
      aria-checked={autostart}
      tabindex="0"
    ></div>
  </div>

  <div class="setting-row">
    <span>Crossfade</span>
    <div class="cf-control">
      <input
        type="range"
        min="50"
        max="500"
        value={Math.round(cfDuration * 100)}
        class="cf-slider"
        oninput={(e) => updateCrossfade(parseInt(e.target.value) / 100)}
      />
      <span class="cf-value">{cfDuration.toFixed(1)}s</span>
    </div>
  </div>

  <div class="about">
    <div class="about-title">RustBird</div>
    <div class="about-version">Version 0.1.0</div>
    <div class="about-desc">Ambient bird songs for focus & relaxation.</div>
  </div>
</section>

<style>
  .settings {
    flex: 1;
    padding: 8px 14px;
  }
  .back-btn {
    font-size: 11px;
    margin-bottom: 12px;
  }
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
    background: #2c2c2e;
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
    background: #48484a;
    border-radius: 50%;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.35);
    transition: left 0.2s ease, background 0.2s ease;
  }

  .toggle-pill.active {
    background: #aeaeb2;
  }

  .toggle-pill.active::after {
    left: 18px;
    background: #ffffff;
  }

  @media (prefers-color-scheme: light) {
    .toggle-pill { background: #d1d1d6; }
    .toggle-pill::after { background: #ffffff; }
    .toggle-pill.active { background: #8e8e93; }
    .toggle-pill.active::after { background: #ffffff; }
  }
  .cf-control {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .cf-slider {
    width: 80px;
    height: 2px;
    -webkit-appearance: none;
    appearance: none;
    background: var(--fader-track);
    border-radius: 2px;
    outline: none;
    cursor: pointer;
  }
  .cf-slider::-webkit-slider-thumb {
    -webkit-appearance: none;
    appearance: none;
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--fader-fill);
  }
  .cf-value {
    font-size: 11px;
    color: var(--text-secondary);
    width: 30px;
  }
  .about {
    margin-top: 24px;
    text-align: center;
    color: var(--text-secondary);
  }
  .about-title {
    font-size: 13px;
    font-weight: 600;
    color: var(--text);
  }
  .about-version {
    font-size: 10px;
    margin-top: 2px;
  }
  .about-desc {
    font-size: 10px;
    margin-top: 4px;
  }
</style>
