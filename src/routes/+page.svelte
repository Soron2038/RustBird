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

  let state = $state<AppState | null>(null);
  let showSettings = $state(false);
  let activeSounds = $derived(
    state ? state.sounds.filter((s: { is_active: boolean }) => s.is_active) : [],
  );

  onMount(() => {
    getState().then((s) => (state = s));
    const unlisten = listen('sound-playback-failed', () => refreshState());
    return () => {
      unlisten.then((fn) => fn());
    };
  });

  async function refreshState() {
    state = await getState();
  }

  function toggleSettings() {
    showSettings = !showSettings;
  }
</script>

{#if state}
  <Header
    isPaused={state.is_paused}
    {showSettings}
    onTogglePause={async () => {
      if (state!.is_paused) {
        state = await resumeAll();
      } else {
        state = await pauseAll();
      }
    }}
    onToggleSettings={toggleSettings}
  />

  {#if showSettings}
    <Settings
      autostartEnabled={state.autostart_enabled}
      autopauseOnLock={state.autopause_on_lock}
      crossfadeDuration={state.crossfade_duration}
      onBack={() => (showSettings = false)}
    />
  {:else}
    <Mixer
      sounds={activeSounds}
      masterVolume={state.master_volume}
      onVolumeChange={async (id, volume) => {
        state = await setVolume(id, volume);
      }}
      onMasterVolumeChange={async (volume) => {
        state = await setMasterVolume(volume);
      }}
    />

    <SoundList
      sounds={state.sounds}
      onToggle={async (id) => {
        state = await toggleSound(id);
      }}
      onImport={refreshState}
      onRemove={async (id) => {
        state = await removeSound(id);
      }}
    />
  {/if}
{/if}
