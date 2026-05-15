<script lang="ts">
  import { filterLevel, filterSearch, filterSearchRegex, unityMode, autoScroll, isPaused, wordWrap } from '../stores/logcat';
  import type { LogLevel } from '../types';

  export let onClear: () => void = () => {};
  export let onExport: () => void = () => {};
  export let onPauseToggle: () => void = () => {};
  export let entryCount: number = 0;

  const levelOptions: { value: string; label: string; letter: string; cls: string }[] = [
    { value: '',        label: 'All',     letter: '',  cls: '' },
    { value: 'verbose', label: 'Verbose', letter: 'V', cls: 'lv' },
    { value: 'debug',   label: 'Debug',   letter: 'D', cls: 'ld' },
    { value: 'info',    label: 'Info',    letter: 'I', cls: 'li' },
    { value: 'warn',    label: 'Warn',    letter: 'W', cls: 'lw' },
    { value: 'error',   label: 'Error',   letter: 'E', cls: 'le' },
    { value: 'fatal',   label: 'Fatal',   letter: 'F', cls: 'lf' },
  ];

  let levelValue = '';
  $: $filterLevel = (levelValue || null) as LogLevel | null;

  // --- Query suggestions ---
  interface Suggestion {
    prefix: string;
    description: string;
    example: string;
  }

  const suggestions: Suggestion[] = [
    { prefix: 'tag:',     description: 'Filter by log tag',        example: 'tag:Unity' },
    { prefix: 'pid:',     description: 'Filter by process ID',     example: 'pid:12345' },
    { prefix: 'tid:',     description: 'Filter by thread ID',      example: 'tid:6789' },
    { prefix: 'message:', description: 'Filter by message content', example: 'message:error' },
    { prefix: 'level:',   description: 'Filter by min level',      example: 'level:warn' },
    { prefix: 'package:', description: 'Filter by package name',   example: 'package:com.example' },
  ];

  let showSuggestions = false;
  let queryInputEl: HTMLInputElement;
  let selectedSuggestionIndex = -1;

  $: filteredSuggestions = $filterSearch
    ? suggestions.filter(s => s.prefix.startsWith($filterSearch.toLowerCase()) || s.description.toLowerCase().includes($filterSearch.toLowerCase()))
    : suggestions;

  function onQueryFocus() {
    showSuggestions = true;
    selectedSuggestionIndex = -1;
  }

  function onQueryBlur() {
    // Delay to allow click on suggestion
    setTimeout(() => { showSuggestions = false; }, 150);
  }

  function applySuggestion(s: Suggestion) {
    $filterSearch = s.prefix;
    showSuggestions = false;
    queryInputEl?.focus();
  }

  function onQueryKeydown(e: KeyboardEvent) {
    if (!showSuggestions || filteredSuggestions.length === 0) return;
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      selectedSuggestionIndex = (selectedSuggestionIndex + 1) % filteredSuggestions.length;
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      selectedSuggestionIndex = selectedSuggestionIndex <= 0 ? filteredSuggestions.length - 1 : selectedSuggestionIndex - 1;
    } else if (e.key === 'Enter' && selectedSuggestionIndex >= 0) {
      e.preventDefault();
      applySuggestion(filteredSuggestions[selectedSuggestionIndex]);
    } else if (e.key === 'Escape') {
      showSuggestions = false;
    }
  }
</script>

<div class="filter-bar">
  <!-- Query input area with suggestions dropdown -->
  <div class="query-area">
    <svg class="search-icon" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
      <circle cx="11" cy="11" r="8"></circle>
      <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
    </svg>
    <input
      class="query-input"
      placeholder="Filter logcat... (tag:, pid:, message:)"
      bind:value={$filterSearch}
      bind:this={queryInputEl}
      on:focus={onQueryFocus}
      on:blur={onQueryBlur}
      on:keydown={onQueryKeydown}
    />
    <button
      class="regex-toggle"
      class:active={$filterSearchRegex}
      on:click={() => $filterSearchRegex = !$filterSearchRegex}
    >
      <span class="has-tooltip">
        .*
        <span class="tooltip">Toggle regex mode</span>
      </span>
    </button>

    <!-- Suggestions dropdown -->
    {#if showSuggestions && filteredSuggestions.length > 0 && $filterSearch.length < 10 && !$filterSearch.includes(':')}
      <div class="suggestions-dropdown">
        {#each filteredSuggestions as s, i}
          <button
            class="suggestion-item"
            class:highlighted={i === selectedSuggestionIndex}
            on:mousedown|preventDefault={() => applySuggestion(s)}
          >
            <span class="suggestion-prefix">{s.prefix}</span>
            <span class="suggestion-desc">{s.description}</span>
            <span class="suggestion-example">{s.example}</span>
          </button>
        {/each}
      </div>
    {/if}
  </div>

  <!-- Level filter chips -->
  <div class="level-chips">
    {#each levelOptions as opt}
      <button
        class="level-chip {opt.cls}"
        class:active={levelValue === opt.value}
        on:click={() => levelValue = opt.value}
      >
        <span class="has-tooltip">
          {#if opt.letter}
            <span class="chip-letter">{opt.letter}</span>
          {/if}
          {opt.label}
          <span class="tooltip">{opt.value ? `Show ${opt.label} and above` : 'Show all log levels'}</span>
        </span>
      </button>
    {/each}
  </div>

  <div class="bar-divider"></div>

  <!-- Unity toggle -->
  <button
    class="icon-btn unity-toggle"
    class:active={$unityMode}
    on:click={() => $unityMode = !$unityMode}
  >
    <span class="has-tooltip">
      <span class="unity-icon">U</span>
      <span class="tooltip">Unity filter — show only Unity, IL2CPP, Mono logs</span>
    </span>
  </button>

  <div class="bar-divider"></div>

  <!-- Action buttons -->
  <div class="action-group">
    <button class="icon-btn" class:active={$wordWrap} on:click={() => $wordWrap = !$wordWrap}>
      <span class="has-tooltip">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M3 6h18"></path>
          <path d="M3 12h15a3 3 0 1 1 0 6h-4"></path>
          <polyline points="13 16 11 18 13 20"></polyline>
        </svg>
        <span class="tooltip">Soft-Wrap — wrap long lines</span>
      </span>
    </button>
    <button class="icon-btn" class:active={$autoScroll} on:click={() => $autoScroll = !$autoScroll}>
      <span class="has-tooltip">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <polyline points="7 13 12 18 17 13"></polyline>
          <line x1="12" y1="6" x2="12" y2="18"></line>
        </svg>
        <span class="tooltip">Scroll to end — auto-follow new logs</span>
      </span>
    </button>
    <button class="icon-btn" on:click={onPauseToggle}>
      <span class="has-tooltip">
        {#if $isPaused}
          <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor"><polygon points="5 3 19 12 5 21 5 3"></polygon></svg>
          <span class="tooltip">Resume — continue receiving logs</span>
        {:else}
          <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor"><rect x="6" y="4" width="4" height="16"></rect><rect x="14" y="4" width="4" height="16"></rect></svg>
          <span class="tooltip">Pause — stop receiving new logs</span>
        {/if}
      </span>
    </button>
    <button class="icon-btn" on:click={onClear}>
      <span class="has-tooltip">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <polyline points="3 6 5 6 21 6"></polyline>
          <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path>
        </svg>
        <span class="tooltip">Clear — delete all log entries</span>
      </span>
    </button>
    <button class="icon-btn" on:click={onExport}>
      <span class="has-tooltip">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path>
          <polyline points="7 10 12 15 17 10"></polyline>
          <line x1="12" y1="15" x2="12" y2="3"></line>
        </svg>
        <span class="tooltip">Export — save logs to .txt file</span>
      </span>
    </button>
  </div>

  {#if entryCount > 0}
    <span class="entry-count">{entryCount.toLocaleString()}</span>
  {/if}
</div>

<style>
  .filter-bar {
    display: flex;
    align-items: center;
    padding: 4px 8px;
    gap: 6px;
    border-bottom: 1px solid var(--border-color);
    background: var(--bg-secondary);
    flex-shrink: 0;
  }

  /* Query input */
  .query-area {
    display: flex;
    align-items: center;
    gap: 4px;
    flex: 1;
    min-width: 160px;
    max-width: 400px;
    background: var(--bg-primary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    padding: 0 6px;
    height: 26px;
    transition: border-color 0.15s;
    position: relative;
  }
  .query-area:focus-within {
    border-color: var(--accent);
  }
  .search-icon {
    color: var(--text-secondary);
    flex-shrink: 0;
    opacity: 0.6;
  }
  .query-input {
    flex: 1;
    background: transparent;
    border: none;
    outline: none;
    color: var(--text-primary);
    font-family: var(--font-mono);
    font-size: 12px;
    padding: 0;
    height: 100%;
  }
  .query-input::placeholder {
    color: var(--text-secondary);
    opacity: 0.5;
    font-family: var(--font-ui);
  }
  .regex-toggle {
    background: transparent;
    border: 1px solid transparent;
    border-radius: 2px;
    color: var(--text-secondary);
    font-family: var(--font-mono);
    font-size: 11px;
    font-weight: 700;
    padding: 1px 4px;
    cursor: pointer;
    opacity: 0.5;
    transition: all 0.15s;
  }
  .regex-toggle:hover { opacity: 0.8; }
  .regex-toggle.active {
    opacity: 1;
    color: var(--accent);
    border-color: var(--accent);
    background: rgba(0, 122, 204, 0.1);
  }

  /* Suggestions dropdown */
  .suggestions-dropdown {
    position: absolute;
    top: calc(100% + 4px);
    left: 0;
    right: 0;
    background: var(--bg-tertiary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.4);
    z-index: 100;
    overflow: hidden;
    animation: dropIn 0.1s ease;
  }
  @keyframes dropIn {
    from { opacity: 0; transform: translateY(-4px); }
    to { opacity: 1; transform: translateY(0); }
  }
  .suggestion-item {
    display: flex;
    align-items: center;
    gap: 10px;
    width: 100%;
    padding: 6px 10px;
    background: transparent;
    border: none;
    color: var(--text-primary);
    font-family: var(--font-ui);
    font-size: 12px;
    cursor: pointer;
    text-align: left;
    transition: background 0.08s;
  }
  .suggestion-item:hover,
  .suggestion-item.highlighted {
    background: var(--accent);
    color: white;
  }
  .suggestion-prefix {
    font-family: var(--font-mono);
    font-weight: 700;
    color: var(--accent);
    min-width: 70px;
    flex-shrink: 0;
  }
  .suggestion-item:hover .suggestion-prefix,
  .suggestion-item.highlighted .suggestion-prefix {
    color: white;
  }
  .suggestion-desc {
    flex: 1;
    color: var(--text-secondary);
  }
  .suggestion-item:hover .suggestion-desc,
  .suggestion-item.highlighted .suggestion-desc {
    color: rgba(255, 255, 255, 0.8);
  }
  .suggestion-example {
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-secondary);
    opacity: 0.5;
  }
  .suggestion-item:hover .suggestion-example,
  .suggestion-item.highlighted .suggestion-example {
    color: rgba(255, 255, 255, 0.5);
  }

  /* Level filter chips */
  .level-chips {
    display: flex;
    gap: 1px;
    flex-shrink: 0;
  }
  .level-chip {
    background: transparent;
    border: 1px solid transparent;
    border-radius: 3px;
    color: var(--text-secondary);
    font-family: var(--font-ui);
    font-size: 11px;
    padding: 2px 6px;
    cursor: pointer;
    transition: all 0.12s;
  }
  .level-chip:hover { background: var(--bg-hover); color: var(--text-bright); }
  .level-chip.active {
    background: var(--bg-hover);
    color: var(--text-bright);
    border-color: var(--border-color);
  }
  .chip-letter {
    font-weight: 800;
    font-size: 10px;
    width: 14px;
    height: 12px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    border-radius: 2px;
    color: #fff;
  }
  .lv .chip-letter { background: #6a6a6a; }
  .ld .chip-letter { background: #3369cc; }
  .li .chip-letter { background: #2e8b57; }
  .lw .chip-letter { background: #b8860b; }
  .le .chip-letter { background: #c62828; }
  .lf .chip-letter { background: #8e24aa; }

  .bar-divider {
    width: 1px;
    height: 18px;
    background: var(--border-color);
    flex-shrink: 0;
  }

  /* Unity toggle */
  .unity-toggle .unity-icon {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 16px;
    height: 14px;
    border-radius: 2px;
    background: #555;
    color: #fff;
    font-size: 10px;
    font-weight: 800;
    font-family: var(--font-ui);
    line-height: 1;
  }
  .unity-toggle.active .unity-icon {
    background: #3d8b37;
  }

  /* Icon buttons */
  .icon-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 26px;
    height: 26px;
    background: transparent;
    border: 1px solid transparent;
    border-radius: var(--radius-sm);
    color: var(--text-secondary);
    cursor: pointer;
    transition: all 0.12s;
    flex-shrink: 0;
  }
  .icon-btn:hover {
    background: var(--bg-hover);
    color: var(--text-bright);
  }
  .icon-btn.active {
    color: var(--accent);
    background: rgba(0, 122, 204, 0.1);
    border-color: rgba(0, 122, 204, 0.3);
  }

  .action-group {
    display: flex;
    gap: 2px;
    flex-shrink: 0;
  }

  .entry-count {
    font-size: 10px;
    color: var(--text-secondary);
    opacity: 0.6;
    flex-shrink: 0;
    font-family: var(--font-mono);
  }

  /* Tooltip system — appears after 0.5s hover */
  .has-tooltip {
    position: relative;
    display: inline-flex;
    align-items: center;
    gap: 3px;
  }
  .tooltip {
    position: absolute;
    bottom: calc(100% + 8px);
    left: 50%;
    transform: translateX(-50%);
    background: #1a1a1a;
    color: #e0e0e0;
    font-family: var(--font-ui);
    font-size: 11px;
    font-weight: 400;
    padding: 4px 8px;
    border-radius: 4px;
    white-space: nowrap;
    pointer-events: none;
    opacity: 0;
    transition: opacity 0.15s ease;
    transition-delay: 0s;
    z-index: 200;
    border: 1px solid var(--border-color);
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.4);
    line-height: 1.3;
  }
  .tooltip::after {
    content: '';
    position: absolute;
    top: 100%;
    left: 50%;
    transform: translateX(-50%);
    border: 4px solid transparent;
    border-top-color: #1a1a1a;
  }
  .has-tooltip:hover .tooltip {
    opacity: 1;
    transition-delay: 0.5s;
  }
</style>
