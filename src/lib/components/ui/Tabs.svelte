<script lang="ts">
  export let tabs: { id: string; label: string }[] = [];
  export let activeTab: string = '';
</script>

<div class="tabs">
  {#each tabs as tab}
    <button
      class="tab"
      class:active={activeTab === tab.id}
      on:click={() => activeTab = tab.id}
    >
      {tab.label}
    </button>
  {/each}
</div>

<style>
  .tabs {
    display: flex;
    gap: 0;
    border-bottom: 1px solid var(--border-color);
  }
  .tab {
    padding: 6px 16px;
    background: transparent;
    border: none;
    border-bottom: 2px solid transparent;
    color: var(--text-secondary);
    font-family: var(--font-ui);
    font-size: 13px;
    cursor: pointer;
    transition: all 0.15s;
  }
  .tab:hover { color: var(--text-primary); }
  .tab.active {
    color: var(--text-bright);
    border-bottom-color: var(--accent);
  }
</style>
