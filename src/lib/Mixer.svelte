<script lang="ts">
  import type { Sound } from '$lib/types';

  let {
    sounds,
    masterVolume,
    onVolumeChange,
    onMasterVolumeChange,
  }: {
    sounds: Sound[];
    masterVolume: number;
    onVolumeChange: (id: string, volume: number) => Promise<void>;
    onMasterVolumeChange: (volume: number) => Promise<void>;
  } = $props();
</script>

<section class="mixer">
  <div class="section-label">Now Playing</div>

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
            oninput={(e) =>
              onVolumeChange(sound.id, parseInt((e.target as HTMLInputElement).value) / 100)}
          />
        </div>
      </div>
    {/each}
  </div>

  <div class="master">
    <span class="master-icon">
      <svg
        width="18"
        height="18"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
      >
        <polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5" />
        <path d="M15.54 8.46a5 5 0 0 1 0 7.07" />
      </svg>
    </span>
    <input
      type="range"
      min="0"
      max="100"
      value={Math.round(masterVolume * 100)}
      class="slider-track master-slider"
      oninput={(e) => onMasterVolumeChange(parseInt((e.target as HTMLInputElement).value) / 100)}
    />
    <span class="master-icon">
      <svg
        width="18"
        height="18"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
      >
        <polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5" />
        <path d="M19.07 4.93a10 10 0 0 1 0 14.14" />
        <path d="M15.54 8.46a5 5 0 0 1 0 7.07" />
      </svg>
    </span>
  </div>
</section>

<style>
  .mixer {
    padding: 10px 14px 8px;
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
    flex-direction: row;
    align-items: flex-end;
    gap: 2px;
  }
  .fader-label {
    font-size: 10px;
    color: var(--text-secondary);
    writing-mode: vertical-rl;
    transform: rotate(180deg);
    white-space: nowrap;
    overflow: hidden;
    max-height: 60px;
    text-overflow: clip;
    line-height: 1;
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
    display: flex;
    align-items: center;
    color: var(--text-secondary);
  }
  .master-slider {
    width: 120px;
  }
</style>
