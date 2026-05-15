<script lang="ts">
  export let value: string = '';
  export let placeholder: string = '';
  export let type: string = 'text';
  export let size: 'sm' | 'md' = 'md';
</script>

<input
  class="input input-{size}"
  {type}
  {placeholder}
  bind:value
  on:input
  on:keydown
/>

<style>
  .input {
    background: var(--bg-primary);
    color: var(--text-primary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    font-family: var(--font-ui);
    outline: none;
    transition: border-color 0.15s;
    width: 100%;
  }
  .input:focus { border-color: var(--accent); }
  .input::placeholder { color: var(--text-secondary); }
  .input-sm { padding: 2px 8px; font-size: 12px; }
  .input-md { padding: 4px 10px; font-size: 13px; }
</style>
