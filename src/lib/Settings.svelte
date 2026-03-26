<script>
  import { invoke } from '@tauri-apps/api/core';
  import { enable, disable, isEnabled } from '@tauri-apps/plugin-autostart';

  /* eslint-disable state_referenced_locally */
  let { autostartEnabled, crossfadeDuration, onBack } = $props();
  let autostart = $state(autostartEnabled);  // local mutable copy
  let cfDuration = $state(crossfadeDuration);  // local mutable copy

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
    <button class="toggle" onclick={toggleAutostart}>
      {autostart ? 'An' : 'Aus'}
    </button>
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
    <div class="about-title">BackBird</div>
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
  .toggle {
    font-size: 12px;
    padding: 2px 8px;
    border: 0.5px solid var(--separator);
    border-radius: 4px;
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
