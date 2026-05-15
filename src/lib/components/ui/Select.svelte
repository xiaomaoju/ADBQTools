<script lang="ts">
  export let value: string = '';
  export let options: { value: string; label: string }[] = [];
  export let placeholder: string = '';
</script>

<select class="select" bind:value on:change>
  {#if placeholder}
    <option value="" disabled>{placeholder}</option>
  {/if}
  {#each options as opt}
    <option value={opt.value}>{opt.label}</option>
  {/each}
</select>

<style>
  .select {
    background: var(--bg-primary);
    color: var(--text-primary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    padding: 4px 8px;
    font-family: var(--font-ui);
    font-size: 13px;
    outline: none;
    cursor: pointer;
  }
  .select:focus { border-color: var(--accent); }
</style>
