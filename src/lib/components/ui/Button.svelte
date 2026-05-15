<script lang="ts">
  export let variant: 'primary' | 'secondary' | 'ghost' | 'danger' = 'secondary';
  export let size: 'sm' | 'md' = 'md';
  export let disabled: boolean = false;
  export let active: boolean = false;
</script>

<button
  class="btn btn-{variant} btn-{size}"
  class:active
  {disabled}
  on:click
>
  <slot />
</button>

<style>
  .btn {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    font-family: var(--font-ui);
    cursor: pointer;
    transition: all 0.15s ease;
    white-space: nowrap;
  }
  .btn:disabled { opacity: 0.5; cursor: not-allowed; }
  .btn-sm { padding: 2px 8px; font-size: 12px; }
  .btn-md { padding: 4px 12px; font-size: 13px; }
  .btn-primary {
    background: var(--accent);
    color: white;
    border-color: var(--accent);
  }
  .btn-primary:hover:not(:disabled) { background: var(--accent-hover); }
  .btn-secondary {
    background: var(--bg-tertiary);
    color: var(--text-primary);
  }
  .btn-secondary:hover:not(:disabled) { background: var(--bg-hover); }
  .btn-ghost {
    background: transparent;
    color: var(--text-primary);
    border-color: transparent;
  }
  .btn-ghost:hover:not(:disabled) { background: var(--bg-hover); }
  .btn-danger {
    background: var(--error);
    color: white;
    border-color: var(--error);
  }
  .btn-danger:hover:not(:disabled) { opacity: 0.9; }
  .active {
    background: var(--accent) !important;
    color: white !important;
    border-color: var(--accent) !important;
  }
</style>
