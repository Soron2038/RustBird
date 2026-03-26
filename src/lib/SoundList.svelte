<script>
  import { invoke } from '@tauri-apps/api/core';
  import { open } from '@tauri-apps/plugin-dialog';

  let { sounds, hasActiveSounds, onToggle, onImport, onRemove } = $props();

  async function handleImport() {
    await invoke('set_dialog_open');
    try {
      const path = await open({
        filters: [{ name: 'Audio', extensions: ['mp3', 'wav'] }],
      });
      if (path) {
        await invoke('import_sound', { path });
        onImport();
      }
    } finally {
      await invoke('set_dialog_closed');
    }
  }
</script>

<section class="sound-list" class:expanded={!hasActiveSounds}>
  <div class="section-label">Sounds</div>

  <div class="list">
    {#each sounds as sound (sound.id)}
      <button class="sound-row" onclick={() => onToggle(sound.id)}>
        <span class="sound-name" class:inactive={!sound.is_active}>
          {sound.name}
        </span>
        {#if !sound.is_bundled}
          <span
            class="remove-btn"
            role="button"
            tabindex="0"
            onclick={(e) => { e.stopPropagation(); onRemove(sound.id); }}
            onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.stopPropagation(); onRemove(sound.id); } }}
            title="Remove"
          >×</span>
        {/if}
        <span class="dot" class:active={sound.is_active}>
          {sound.is_active ? '●' : '○'}
        </span>
      </button>
    {/each}
  </div>

  <button class="import-btn" onclick={handleImport}>
    ＋ Add Sound
  </button>
</section>

<style>
  .sound-list {
    flex: 1;
    display: flex;
    flex-direction: column;
    padding: 8px 14px;
    min-height: 0;
  }
  .expanded {
    padding-top: 0;
  }
  .list {
    flex: 1;
    overflow-y: auto;
    min-height: 0;
  }
  .sound-row {
    display: flex;
    align-items: center;
    padding: 5px 0;
    border-bottom: 0.5px solid var(--separator);
    width: 100%;
    text-align: left;
    font-size: 12px;
  }
  .sound-name {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .sound-name.inactive {
    color: var(--text-secondary);
  }
  .remove-btn {
    font-size: 14px;
    padding: 0 4px;
    color: var(--text-secondary);
  }
  .remove-btn:hover {
    color: var(--text);
  }
  .dot {
    font-size: 10px;
    color: var(--dot-inactive);
    margin-left: 4px;
  }
  .dot.active {
    color: var(--dot-active);
  }
  .import-btn {
    display: block;
    width: 100%;
    text-align: center;
    padding: 8px;
    font-size: 11px;
    color: var(--text-secondary);
  }
</style>
