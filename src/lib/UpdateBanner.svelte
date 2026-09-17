<script lang="ts">
  import { installUpdate } from '$lib/commands';

  let {
    version,
    onDismiss,
  }: {
    version: string;
    onDismiss: () => void;
  } = $props();

  let installing = $state(false);
  let error = $state<string | null>(null);

  async function install() {
    installing = true;
    error = null;
    try {
      // Resolves only if the install fails — on success the app restarts.
      await installUpdate();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
      installing = false;
    }
  }
</script>

<div class="update-overlay" role="dialog" aria-modal="true">
  <p>RustBird {version} ist bereit</p>
  {#if error}
    <p class="error" role="alert">Installation fehlgeschlagen: {error}</p>
  {/if}
  <div class="update-buttons">
    <button class="btn-install" onclick={install} disabled={installing}>
      {installing ? 'Wird installiert…' : 'Jetzt neu starten'}
    </button>
    <button class="btn-later" onclick={onDismiss} disabled={installing}>Später</button>
  </div>
</div>

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
  .error {
    font-size: 11px;
    font-weight: 400;
    color: var(--text-secondary);
    word-break: break-word;
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
  button:disabled {
    opacity: 0.6;
    cursor: default;
  }
</style>
