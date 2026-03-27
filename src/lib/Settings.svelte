<script lang="ts">
  import AutostartToggle from '$lib/settings/AutostartToggle.svelte';
  import AutopauseLockToggle from '$lib/settings/AutopauseLockToggle.svelte';
  import CrossfadeSlider from '$lib/settings/CrossfadeSlider.svelte';
  import AboutSection from '$lib/settings/AboutSection.svelte';

  let {
    autostartEnabled,
    autopauseOnLock,
    crossfadeDuration,
    onBack,
  }: {
    autostartEnabled: boolean;
    autopauseOnLock: boolean;
    crossfadeDuration: number;
    onBack: () => void;
  } = $props();
</script>

<section class="settings">
  <button class="back-btn" onclick={onBack}>← Back</button>
  <AutostartToggle enabled={autostartEnabled} />
  <AutopauseLockToggle enabled={autopauseOnLock} />
  <CrossfadeSlider duration={crossfadeDuration} />
  <AboutSection />
</section>

<style>
  .settings {
    flex: 1;
    padding: 8px 14px;
  }
  .back-btn {
    font-size: 11px;
    margin-bottom: 12px;
  }
</style>
