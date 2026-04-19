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
  } from '$lib/commands';
  import Header from '$lib/Header.svelte';
  import Mixer from '$lib/Mixer.svelte';
  import SoundList from '$lib/SoundList.svelte';
  import Settings from '$lib/Settings.svelte';

  let appState: AppState | null = $state(null);
  let showSettings = $state(false);
  onMount(() => {
    getState().then((s) => (appState = s));
    const unlisten = listen('sound-playback-failed', () => refreshState());
    return () => {
      unlisten.then((fn) => fn());
    };
  });

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

  {#if showSettings}
    <Settings
      autostartEnabled={appState.autostart_enabled}
      autopauseOnLock={appState.autopause_on_lock}
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
