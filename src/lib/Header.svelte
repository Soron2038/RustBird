<script>
  let { isPaused, showSettings, onTogglePause, onToggleSettings } = $props();
</script>

<header>
  <span class="title">BackBird</span>
  <div class="actions">
    <button onclick={onTogglePause} title={isPaused ? 'Resume' : 'Pause'}>
      {isPaused ? '▶' : '⏸'}
    </button>
    <button onclick={onToggleSettings} title="Settings" class:active={showSettings}>
      ⚙
    </button>
  </div>
</header>

<style>
  header {
    display: flex;
    align-items: center;
    padding: 12px 14px 8px;
  }
  .title {
    flex: 1;
    font-size: 13px;
    font-weight: 600;
  }
  .actions {
    display: flex;
    gap: 6px;
  }
  .active {
    color: var(--text);
  }
</style>
