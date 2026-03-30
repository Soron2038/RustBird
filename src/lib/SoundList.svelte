<script lang="ts">
  import { open } from '@tauri-apps/plugin-dialog';
  import { importSound, setDialogOpen, setDialogClosed } from '$lib/commands';
  import type { Sound } from '$lib/types';

  let {
    sounds,
    hasActiveSounds,
    onToggle,
    onImport,
    onRemove,
  }: {
    sounds: Sound[];
    hasActiveSounds: boolean;
    onToggle: (id: string) => Promise<void>;
    onImport: () => Promise<void>;
    onRemove: (id: string) => Promise<void>;
  } = $props();

  let importError = $state<string | null>(null);

  let listEl = $state<HTMLElement | null>(null);
  let canScrollUp = $state(false);
  let canScrollDown = $state(false);

  function updateScroll() {
    if (!listEl) return;
    canScrollUp = listEl.scrollTop > 4;
    canScrollDown = listEl.scrollTop < listEl.scrollHeight - listEl.clientHeight - 4;
  }

  $effect(() => {
    updateScroll();
  });

  async function handleImport() {
    importError = null;
    await setDialogOpen();
    try {
      const path = await open({
        filters: [{ name: 'Audio', extensions: ['mp3', 'wav'] }],
      });
      if (path) {
        await importSound(path as string);
        await onImport();
      }
    } catch (err) {
      importError = err instanceof Error ? err.message : String(err);
    } finally {
      await setDialogClosed();
    }
  }
</script>

<section class="sound-list" class:expanded={!hasActiveSounds}>
  <div class="section-label">Sounds</div>

  <div class="list-wrapper">
    <div class="scroll-fade top" class:visible={canScrollUp}></div>
    <div class="list" bind:this={listEl} onscroll={updateScroll}>
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
              onclick={(e) => {
                e.stopPropagation();
                onRemove(sound.id);
              }}
              onkeydown={(e) => {
                if (e.key === 'Enter' || e.key === ' ') {
                  e.stopPropagation();
                  onRemove(sound.id);
                }
              }}
              title="Remove">×</span
            >
          {/if}
          <span class="dot" class:active={sound.is_active}>
            {sound.is_active ? '●' : '○'}
          </span>
        </button>
      {/each}
    </div>
    <div class="scroll-fade bottom" class:visible={canScrollDown}></div>
  </div>

  <button class="import-btn" onclick={handleImport}> ＋ Add Sound </button>
  {#if importError}
    <p class="import-error">{importError}</p>
  {/if}
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
  .list-wrapper {
    flex: 1;
    position: relative;
    min-height: 0;
    display: flex;
    flex-direction: column;
  }

  .list {
    flex: 1;
    overflow-y: auto;
    min-height: 0;
    scrollbar-width: none;
  }

  .list::-webkit-scrollbar {
    display: none;
  }

  .scroll-fade {
    position: absolute;
    left: 0;
    right: 0;
    height: 28px;
    pointer-events: none;
    opacity: 0;
    transition: opacity 0.15s ease;
    z-index: 1;
  }

  .scroll-fade.top {
    top: 0;
    background: linear-gradient(to bottom, var(--bg), transparent);
  }

  .scroll-fade.bottom {
    bottom: 0;
    background: linear-gradient(to top, var(--bg), transparent);
  }

  .scroll-fade.visible {
    opacity: 1;
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
  .import-error {
    font-size: 10px;
    color: #e05252;
    text-align: center;
    padding: 4px 8px;
    word-break: break-word;
  }
</style>
