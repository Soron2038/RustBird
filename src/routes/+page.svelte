<svelte:options runes={true} />

<script lang="ts">
  import { listen } from '@tauri-apps/api/event';
  import { onMount } from 'svelte';
  import type { AppState } from '$lib/types';
  import {
    getState,
    toggleSound,
    setVolume,
    setMasterVolume,
    pauseAll,
    resumeAll,
    removeSound,
    installUpdate,
    setDialogOpen,
    setDialogClosed,
  } from '$lib/commands';
  import Header from '$lib/Header.svelte';
  import Mixer from '$lib/Mixer.svelte';
  import SoundList from '$lib/SoundList.svelte';
  import Settings from '$lib/Settings.svelte';

  let appState: AppState | null = $state(null);
  let showSettings = $state(false);
  let updateVersion = $state<string | null>(null);

  onMount(() => {
    getState().then((s) => (appState = s));
    const unlistenPlayback = listen('sound-playback-failed', () => refreshState());
    const unlistenUpdate = listen<string>('update-available', (e) => {
      updateVersion = e.payload;
      setDialogOpen();
    });
    return () => {
      unlistenPlayback.then((fn) => fn());
      unlistenUpdate.then((fn) => fn());
    };
  });

  async function doInstallUpdate() {
    await installUpdate();
    updateVersion = null;
    setDialogClosed();
  }

  function dismissUpdate() {
    updateVersion = null;
    setDialogClosed();
  }

  async function refreshState() {
    appState = await getState();
  }

  function toggleSettings() {
    showSettings = !showSettings;
  }
</script>

{#if appState}
  <Header
    isPaused={appState.is_paused}
    {showSettings}
    onTogglePause={async () => {
      if (appState!.is_paused) {
        appState = await resumeAll();
      } else {
        appState = await pauseAll();
      }
    }}
    onToggleSettings={toggleSettings}
  />

  {#if updateVersion}
    <div class="update-overlay" role="dialog" aria-modal="true">
      <p>RustBird {updateVersion} ist verfügbar</p>
      <div class="update-buttons">
        <button class="btn-install" onclick={doInstallUpdate}>Installieren</button>
        <button class="btn-later" onclick={dismissUpdate}>Später</button>
      </div>
    </div>
  {/if}

  {#if showSettings}
    <Settings
      autostartEnabled={appState.autostart_enabled}
      autopauseOnLock={appState.autopause_on_lock}
      autoUpdateEnabled={appState.auto_update_enabled}
      crossfadeDuration={appState.crossfade_duration}
      onBack={() => (showSettings = false)}
    />
  {:else}
    <Mixer
      sounds={appState.sounds.filter((s) => s.is_active)}
      masterVolume={appState.master_volume}
      onVolumeChange={async (id, volume) => {
        appState = await setVolume(id, volume);
      }}
      onMasterVolumeChange={async (volume) => {
        appState = await setMasterVolume(volume);
      }}
    />

    <SoundList
      sounds={appState.sounds}
      onToggle={async (id) => {
        appState = await toggleSound(id);
      }}
      onImport={refreshState}
      onRemove={async (id) => {
        appState = await removeSound(id);
      }}
    />
  {/if}
{/if}

<style>
  .update-overlay {
    position: absolute;
    inset: 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 14px;
    background: var(--bg);
    z-index: 100;
    padding: 24px;
    text-align: center;
  }
  .update-overlay p {
    font-size: 13px;
    font-weight: 500;
    margin: 0;
  }
  .update-buttons {
    display: flex;
    gap: 8px;
  }
  .btn-install {
    font-size: 12px;
    padding: 5px 14px;
    border-radius: 6px;
    background: var(--accent, #4a9eff);
    color: #fff;
    border: none;
    cursor: pointer;
  }
  .btn-later {
    font-size: 12px;
    padding: 5px 14px;
    border-radius: 6px;
    background: transparent;
    border: 0.5px solid var(--separator);
    cursor: pointer;
  }
</style>
