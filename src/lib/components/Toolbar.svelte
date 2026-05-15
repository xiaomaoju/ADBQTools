<script lang="ts">
  import Button from './ui/Button.svelte';
  import type { ViewMode } from '../types';

  export let currentView: ViewMode = 'logcat';
</script>

<div class="toolbar">
  <div class="toolbar-left">
    <Button size="sm" active={currentView === 'logcat'} on:click={() => currentView = 'logcat'}>
      Logcat
    </Button>
    <Button size="sm" active={currentView === 'installer'} on:click={() => currentView = 'installer'}>
      Installer
    </Button>
  </div>
  <div class="toolbar-center">
    <span class="app-title">AndroidQTools</span>
  </div>
  <div class="toolbar-right">
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
</style>
