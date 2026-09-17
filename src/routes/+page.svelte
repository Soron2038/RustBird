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
    setDialogOpen,
    setDialogClosed,
  } from '$lib/commands';
  import UpdateBanner from '$lib/UpdateBanner.svelte';
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
    const unlistenUpdate = listen<string>('update-ready', (e) => {
      updateVersion = e.payload;
      setDialogOpen();
    });
    return () => {
      unlistenPlayback.then((fn) => fn());
      unlistenUpdate.then((fn) => fn());
    };
  });

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
    <UpdateBanner version={updateVersion} onDismiss={dismissUpdate} />
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
