<script lang="ts">
  export let open: boolean = false;
  export let title: string = '';

  function handleBackdrop(e: MouseEvent) {
    if (e.target === e.currentTarget) open = false;
  }
</script>

{#if open}
  <div class="backdrop" on:click={handleBackdrop} role="presentation">
    <div class="dialog">
      <div class="dialog-header">
        <span class="dialog-title">{title}</span>
        <button class="close-btn" on:click={() => open = false}>✕</button>
      </div>
      <div class="dialog-body">
        <slot />
      </div>
    </div>
  </div>
{/if}

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }
  .dialog {
    background: var(--bg-secondary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-lg);
    min-width: 400px;
    max-width: 90vw;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
  }
  .dialog-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 16px;
    border-bottom: 1px solid var(--border-color);
  }
  .dialog-title { font-weight: 600; color: var(--text-bright); }
  .close-btn {
    background: none;
    border: none;
    color: var(--text-secondary);
    cursor: pointer;
    font-size: 16px;
    padding: 2px 6px;
    border-radius: var(--radius-sm);
  }
  .close-btn:hover { background: var(--bg-hover); color: var(--text-primary); }
  .dialog-body { padding: 16px; }
</style>
