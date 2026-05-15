<script lang="ts">
  import Button from './ui/Button.svelte';
  import Input from './ui/Input.svelte';
  import Select from './ui/Select.svelte';
  import { filterLevel, filterSearch, filterSearchRegex, unityMode, autoScroll, isPaused, wordWrap } from '../stores/logcat';
  import type { LogLevel } from '../types';

  export let onClear: () => void = () => {};
  export let onExport: () => void = () => {};
  export let onPauseToggle: () => void = () => {};

  const levelOptions = [
    { value: '', label: 'All Levels' },
    { value: 'verbose', label: 'Verbose' },
    { value: 'debug', label: 'Debug' },
    { value: 'info', label: 'Info' },
    { value: 'warn', label: 'Warn' },
    { value: 'error', label: 'Error' },
    { value: 'fatal', label: 'Fatal' },
  ];

  let levelValue = '';
  $: $filterLevel = (levelValue || null) as LogLevel | null;
</script>

<div class="filter-bar">
  <div class="filter-group">
    <Select options={levelOptions} bind:value={levelValue} />

    <div class="search-wrapper">
      <Input
        size="sm"
        placeholder="Search / Regex..."
        bind:value={$filterSearch}
      />
      <Button size="sm" variant="ghost" active={$filterSearchRegex} on:click={() => $filterSearchRegex = !$filterSearchRegex}>
        .*
      </Button>
    </div>

    <Button size="sm" active={$unityMode} on:click={() => $unityMode = !$unityMode}>
      Unity
    </Button>
  </div>

  <div class="filter-actions">
    <Button size="sm" variant="ghost" on:click={onPauseToggle}>
      {$isPaused ? '▶ Resume' : '⏸ Pause'}
    </Button>
    <Button size="sm" variant="ghost" active={$wordWrap} on:click={() => $wordWrap = !$wordWrap}>
      Wrap
    </Button>
    <Button size="sm" variant="ghost" active={$autoScroll} on:click={() => $autoScroll = !$autoScroll}>
      Auto-scroll
    </Button>
    <Button size="sm" variant="ghost" on:click={onClear}>Clear</Button>
    <Button size="sm" variant="ghost" on:click={onExport}>Export</Button>
  </div>
</div>

<style>
  .filter-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 4px 12px;
    gap: 8px;
    border-bottom: 1px solid var(--border-color);
    background: var(--bg-secondary);
  }
  .filter-group { display: flex; align-items: center; gap: 6px; }
  .search-wrapper { display: flex; align-items: center; gap: 2px; width: 250px; }
  .filter-actions { display: flex; align-items: center; gap: 4px; }
</style>
