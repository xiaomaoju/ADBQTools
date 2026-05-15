<script lang="ts">
  import { onMount, onDestroy, tick } from 'svelte';
  import LogcatRow from './LogcatRow.svelte';
  import LogcatFilterBar from './LogcatFilterBar.svelte';
  import StackGroup from './StackGroup.svelte';
  import Tabs from './ui/Tabs.svelte';
  import { logEntries, addLogEntries, clearLogEntries, getFilteredEntries, autoScroll, isPaused } from '../stores/logcat';
  import { devices, activeDeviceSerial } from '../stores/devices';
  import { startLogcat, stopLogcat, pauseLogcat, resumeLogcat, clearLogcat, onLogcat } from '../utils/tauri';
  import type { LogEntry } from '../types';
  import type { UnlistenFn } from '@tauri-apps/api/event';

  let container: HTMLDivElement;
  let unlisteners: Map<string, UnlistenFn> = new Map();
  const ROW_HEIGHT = 20;
  let scrollTop = 0;
  let containerHeight = 0;

  $: deviceTabs = $devices
    .filter(d => d.status === 'online')
    .map(d => ({ id: d.serial, label: d.model || d.serial }));

  $: currentSerial = $activeDeviceSerial ?? '';
  $: allEntries = $logEntries.get(currentSerial) ?? [];
  $: filtered = getFilteredEntries(allEntries);
  $: totalHeight = filtered.length * ROW_HEIGHT;
  $: startIndex = Math.floor(scrollTop / ROW_HEIGHT);
  $: visibleCount = Math.ceil(containerHeight / ROW_HEIGHT) + 2;
  $: visibleEntries = filtered.slice(startIndex, startIndex + visibleCount);

  async function subscribeLogcat(serial: string) {
    if (unlisteners.has(serial)) return;
    try {
      await startLogcat(serial);
      const unlisten = await onLogcat(serial, (entries) => {
        addLogEntries(serial, entries);
        if ($autoScroll && container) {
          tick().then(() => {
            container.scrollTop = container.scrollHeight;
          });
        }
      });
      unlisteners.set(serial, unlisten);
    } catch (e) {
      console.error(`Failed to start logcat for ${serial}:`, e);
    }
  }

  function handleScroll() {
    if (container) {
      scrollTop = container.scrollTop;
    }
  }

  async function handlePauseToggle() {
    if (!currentSerial) return;
    if ($isPaused) {
      await resumeLogcat(currentSerial);
      $isPaused = false;
    } else {
      await pauseLogcat(currentSerial);
      $isPaused = true;
    }
  }

  async function handleClear() {
    if (!currentSerial) return;
    await clearLogcat(currentSerial);
    clearLogEntries(currentSerial);
  }

  function handleExport() {
    const lines = filtered.map(e =>
      `${e.timestamp} ${e.pid} ${e.tid} ${e.level.charAt(0).toUpperCase()} ${e.tag}: ${e.message}`
    );
    const blob = new Blob([lines.join('\n')], { type: 'text/plain' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `logcat-${currentSerial}-${Date.now()}.txt`;
    a.click();
    URL.revokeObjectURL(url);
  }

  $: if ($activeDeviceSerial) {
    subscribeLogcat($activeDeviceSerial);
  }

  onMount(() => {
    if (container) {
      containerHeight = container.clientHeight;
      const ro = new ResizeObserver(() => {
        containerHeight = container.clientHeight;
      });
      ro.observe(container);
      return () => ro.disconnect();
    }
  });

  onDestroy(() => {
    unlisteners.forEach((unlisten, serial) => {
      unlisten();
      stopLogcat(serial);
    });
  });
</script>

<div class="logcat-viewer">
  {#if deviceTabs.length > 1}
    <Tabs tabs={deviceTabs} activeTab={currentSerial} />
  {/if}

  <LogcatFilterBar
    onClear={handleClear}
    onExport={handleExport}
    onPauseToggle={handlePauseToggle}
  />

  <div
    class="log-container"
    bind:this={container}
    on:scroll={handleScroll}
  >
    <div class="log-scroll-spacer" style="height: {totalHeight}px">
      <div class="log-visible" style="transform: translateY({startIndex * ROW_HEIGHT}px)">
        {#each visibleEntries as entry (entry.id)}
          <LogcatRow {entry} />
          {#if entry.stack_frames && entry.stack_frames.length > 0}
            <StackGroup frames={entry.stack_frames} />
          {/if}
        {/each}
      </div>
    </div>
  </div>

  {#if filtered.length === 0 && allEntries.length > 0}
    <div class="empty-filter">No entries match current filters</div>
  {:else if allEntries.length === 0}
    <div class="empty-filter">
      {currentSerial ? 'Waiting for logcat data...' : 'Select a device to start'}
    </div>
  {/if}
</div>

<style>
  .logcat-viewer {
    display: flex;
    flex-direction: column;
    height: 100%;
  }
  .log-container {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
    background: var(--bg-primary);
  }
  .log-scroll-spacer { position: relative; }
  .log-visible { position: absolute; top: 0; left: 0; right: 0; }
  .empty-filter {
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    color: var(--text-secondary);
    font-size: 14px;
  }
</style>
