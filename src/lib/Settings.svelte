<script lang="ts">
  import AutostartToggle from '$lib/settings/AutostartToggle.svelte';
  import CrossfadeSlider from '$lib/settings/CrossfadeSlider.svelte';
  import AboutSection from '$lib/settings/AboutSection.svelte';

  let {
    autostartEnabled,
    crossfadeDuration,
    onBack,
  }: {
    autostartEnabled: boolean;
    crossfadeDuration: number;
    onBack: () => void;
  } = $props();
</script>

<section class="settings">
  <button class="back-btn" onclick={onBack}>← Back</button>
  <AutostartToggle enabled={autostartEnabled} />
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
