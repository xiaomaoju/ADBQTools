<script lang="ts">
  import Button from './ui/Button.svelte';
  import type { ViewMode } from '../types';
  import { tt } from '../i18n';

  export let currentView: ViewMode = 'logcat';
  export let onSettingsClick: () => void = () => {};
</script>

<div class="toolbar">
  <div class="toolbar-left">
    <Button size="sm" active={currentView === 'logcat'} on:click={() => currentView = 'logcat'}>
      {$tt('toolbar.logcat')}
    </Button>
    <Button size="sm" active={currentView === 'installer'} on:click={() => currentView = 'installer'}>
      {$tt('toolbar.installer')}
    </Button>
  </div>
  <div class="toolbar-center">
    <span class="app-title">ADBQTools</span>
  </div>
  <div class="toolbar-right">
    <button class="settings-btn" on:click={onSettingsClick}>
      <span class="has-tooltip">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <circle cx="12" cy="12" r="3"></circle>
          <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"></path>
        </svg>
        <span class="tooltip">{$tt('settings.title')}</span>
      </span>
    </button>
    <slot />
  </div>
</div>

<style>
  .toolbar {
    display: flex;
    align-items: center;
    height: var(--toolbar-height);
    padding: 0 12px;
    gap: 8px;
    flex: 1;
    min-width: 0;
  }
  .toolbar-left { display: flex; gap: 4px; }
  .toolbar-center { flex: 1; text-align: center; }
  .app-title { font-size: 12px; color: var(--text-secondary); }
  .toolbar-right { display: flex; gap: 8px; align-items: center; }
  .settings-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    background: none;
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    color: var(--text-secondary);
    cursor: pointer;
    flex-shrink: 0;
    transition: all 0.15s;
  }
  .settings-btn:hover {
    border-color: var(--accent);
    color: var(--accent);
    background: var(--bg-hover);
  }

  /* Tooltip system */
  .has-tooltip {
    position: relative;
    display: inline-flex;
    align-items: center;
    gap: 3px;
  }
  .tooltip {
    position: absolute;
    top: calc(100% + 8px);
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
    bottom: 100%;
    left: 50%;
    transform: translateX(-50%);
    border: 4px solid transparent;
    border-bottom-color: #1a1a1a;
  }
  .has-tooltip:hover .tooltip {
    opacity: 1;
    transition-delay: 0.5s;
  }
</style>
