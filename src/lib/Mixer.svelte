<script>
  let { sounds, masterVolume, onVolumeChange, onMasterVolumeChange } = $props();
</script>

<section class="mixer">
  <div class="section-label">Aktiver Mix</div>

  <div class="faders">
    {#each sounds as sound (sound.id)}
      <div class="fader-column">
        <span class="fader-label">{sound.name}</span>
        <div class="fader-track-wrapper">
          <div class="fader-track">
            <div class="fader-fill" style="height: {sound.volume * 100}%"></div>
          </div>
          <input
            type="range"
            min="0"
            max="100"
            value={Math.round(sound.volume * 100)}
            class="vertical-slider"
            oninput={(e) => onVolumeChange(sound.id, parseInt(e.target.value) / 100)}
          />
        </div>
      </div>
    {/each}
  </div>

  <div class="master">
    <span class="master-icon">🔈</span>
    <input
      type="range"
      min="0"
      max="100"
      value={Math.round(masterVolume * 100)}
      class="master-slider"
      oninput={(e) => onMasterVolumeChange(parseInt(e.target.value) / 100)}
    />
    <span class="master-icon">🔊</span>
  </div>
</section>

<style>
  .mixer {
    padding: 0 14px 8px;
    border-bottom: 0.5px solid var(--separator);
  }
  .faders {
    display: flex;
    gap: 16px;
    justify-content: center;
    padding: 8px 0;
  }
  .fader-column {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
  }
  .fader-label {
    font-size: 11px;
    max-width: 70px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    text-align: center;
  }
  .fader-track-wrapper {
    position: relative;
    width: 20px;
    height: 60px;
  }
  .fader-track {
    position: absolute;
    left: 50%;
    transform: translateX(-50%);
    width: 3px;
    height: 60px;
    background: var(--fader-track);
    border-radius: 2px;
    overflow: hidden;
  }
  .fader-fill {
    position: absolute;
    bottom: 0;
    width: 100%;
    background: var(--fader-fill);
    border-radius: 2px;
    transition: height 0.1s ease;
  }
  .vertical-slider {
    position: absolute;
    top: 0;
    left: 0;
    width: 60px;
    height: 20px;
    opacity: 0;
    cursor: pointer;
    transform: rotate(-90deg) translateX(-60px);
    transform-origin: top left;
  }
  .master {
    display: flex;
    align-items: center;
    gap: 6px;
    justify-content: center;
    padding-top: 8px;
  }
  .master-icon {
    font-size: 10px;
    color: var(--text-secondary);
  }
  .master-slider {
    width: 120px;
    height: 2px;
    -webkit-appearance: none;
    appearance: none;
    background: var(--fader-track);
    border-radius: 2px;
    outline: none;
    cursor: pointer;
  }
  .master-slider::-webkit-slider-thumb {
    -webkit-appearance: none;
    appearance: none;
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--fader-fill);
    cursor: pointer;
  }
</style>
