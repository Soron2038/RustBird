<script lang="ts">
  import { getVersion } from '@tauri-apps/api/app';
  import { openUrl } from '@tauri-apps/plugin-opener';
  import AppName from '$lib/AppName.svelte';
  import { onMount } from 'svelte';

  let version = $state('');

  onMount(async () => {
    version = await getVersion();
  });
</script>

<div class="about">
  <div class="about-title">
    <AppName /><span class="about-version">{version ? ` ${version}` : ''}</span>
  </div>
  <div class="about-author">by soron2038</div>
  <div class="about-sounds">
    Sounds from
    <button class="link" onclick={() => openUrl('https://xeno-canto.org')}>
      xeno-canto.org ↗
    </button>
  </div>
</div>

<style>
  .about {
    margin-top: 16px;
    padding-top: 12px;
    border-top: 0.5px solid var(--separator);
    text-align: center;
    color: var(--text-secondary);
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .about-title {
    font-size: 12px;
    font-weight: 600;
    color: var(--text);
  }
  .about-version {
    font-weight: 400;
    color: var(--text-secondary);
  }
  .about-author {
    font-size: 10px;
  }
  .about-sounds {
    font-size: 10px;
  }
  .link {
    color: var(--accent);
    font-size: 10px;
    padding: 0;
    background: none;
    border: none;
    cursor: pointer;
    font-family: inherit;
  }
</style>
