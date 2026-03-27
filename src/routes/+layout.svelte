<script>
  import '../app.css';
  let { children } = $props();
</script>

<div class="app-window">
  {@render children()}
</div>

<style>
  .app-window {
    height: 100%;
    display: flex;
    flex-direction: column;
    background: var(--bg);
    border-radius: 10px;
    overflow: hidden;
  }
</style>
