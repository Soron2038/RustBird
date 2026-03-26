<svelte:options runes={true} />

<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';
  import type { AppState } from '$lib/types';
  import Header from '$lib/Header.svelte';
  import Mixer from '$lib/Mixer.svelte';
  import SoundList from '$lib/SoundList.svelte';
  import Settings from '$lib/Settings.svelte';

  let state: AppState | null = $state(null);
  let showSettings = $state(false);

  onMount(async () => {
    state = await invoke('get_state');
  });

  async function refreshState() {
    state = await invoke('get_state');
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
        await invoke('resume_all');
      } else {
        await invoke('pause_all');
      }
      await refreshState();
    }}
    onToggleSettings={toggleSettings}
  />

  {#if showSettings}
    <Settings
      autostartEnabled={state.autostart_enabled}
      crossfadeDuration={state.crossfade_duration}
      onBack={() => showSettings = false}
    />
  {:else}
    {#if state.sounds.some(s => s.is_active)}
      <Mixer
        sounds={state.sounds.filter(s => s.is_active)}
        masterVolume={state.master_volume}
        onVolumeChange={async (id, volume) => {
          await invoke('set_volume', { id, volume });
          await refreshState();
        }}
        onMasterVolumeChange={async (volume) => {
          await invoke('set_master_volume', { volume });
          await refreshState();
        }}
      />
    {/if}

    <SoundList
      sounds={state.sounds}
      hasActiveSounds={state.sounds.some(s => s.is_active)}
      onToggle={async (id) => {
        state = await invoke('toggle_sound', { id });
      }}
      onImport={refreshState}
      onRemove={async (id) => {
        await invoke('remove_sound', { id });
        await refreshState();
      }}
    />
  {/if}
{/if}
