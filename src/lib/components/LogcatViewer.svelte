<script lang="ts">
  import { onMount, onDestroy, tick } from 'svelte';
  import LogcatRow from './LogcatRow.svelte';
  import LogcatFilterBar from './LogcatFilterBar.svelte';
  import StackGroup from './StackGroup.svelte';
  import Tabs from './ui/Tabs.svelte';
  import { logEntries, addLogEntries, clearLogEntries, getFilteredEntries, autoScroll, isPaused, wordWrap } from '../stores/logcat';
  import { devices, activeDeviceSerial } from '../stores/devices';
  import { startLogcat, stopLogcat, pauseLogcat, resumeLogcat, clearLogcat, onLogcat, onLogcatError } from '../utils/tauri';
  import { tt } from '../i18n';
  import { columnWidths, type ColumnWidths } from '../stores/columnWidths';
  import type { LogEntry } from '../types';
  import type { UnlistenFn } from '@tauri-apps/api/event';

  let container: HTMLDivElement;
  let startedSerials: Set<string> = new Set();

  let resizing: keyof ColumnWidths | null = null;
  let resizeStartX = 0;
  let resizeStartW = 0;

  function onResizeStart(col: keyof ColumnWidths, e: MouseEvent) {
    e.preventDefault();
    resizing = col;
    resizeStartX = e.clientX;
    resizeStartW = $columnWidths[col];
    window.addEventListener('mousemove', onResizeMove);
    window.addEventListener('mouseup', onResizeEnd);
    document.body.style.cursor = 'col-resize';
    document.body.style.userSelect = 'none';
  }

  function onResizeMove(e: MouseEvent) {
    if (!resizing) return;
    const delta = e.clientX - resizeStartX;
    columnWidths.resize(resizing, resizeStartW + delta);
  }

  function onResizeEnd() {
    resizing = null;
    window.removeEventListener('mousemove', onResizeMove);
    window.removeEventListener('mouseup', onResizeEnd);
    document.body.style.cursor = '';
    document.body.style.userSelect = '';
  }

  function onResizeDblClick(col: keyof ColumnWidths) {
    columnWidths.reset();
  }
  let logcatUnlisten: UnlistenFn | null = null;
  let errorUnlisten: UnlistenFn | null = null;
  let logcatDiag: string[] = [];
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
    if (startedSerials.has(serial)) return;
    startedSerials.add(serial);
    try {
      try { await stopLogcat(serial); } catch (_) { /* ignore */ }
      await startLogcat(serial);
    } catch (e) {
      console.error(`Failed to start logcat for ${serial}:`, e);
      startedSerials.delete(serial);
    }
  }

  /** Force restart logcat for a serial */
  async function restartLogcat(serial: string) {
    startedSerials.delete(serial);
    try { await stopLogcat(serial); } catch (_) { /* ignore */ }
    await subscribeLogcat(serial);
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
      `${e.timestamp} ${e.pid} ${e.tid} ${e.package_name || '-'} ${e.level.charAt(0).toUpperCase()} ${e.tag}: ${e.message}`
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

  onMount(async () => {
    if (container) {
      containerHeight = container.clientHeight;
      const ro = new ResizeObserver(() => {
        containerHeight = container.clientHeight;
      });
      ro.observe(container);
    }

    // Single global listener for ALL logcat data (avoids colon issues in event names)
    logcatUnlisten = await onLogcat((serial, entries) => {
      addLogEntries(serial, entries);
      if ($autoScroll && container) {
        tick().then(() => {
          container.scrollTop = container.scrollHeight;
        });
      }
    });

    // Listen for logcat diagnostic/error messages from backend
    errorUnlisten = await onLogcatError((msg) => {
      console.warn('[logcat-diag]', msg);
      logcatDiag = [...logcatDiag.slice(-19), msg]; // keep last 20
    });

    // Start logcat for current device if already selected
    if ($activeDeviceSerial) {
      subscribeLogcat($activeDeviceSerial);
    }
  });

  onDestroy(() => {
    logcatUnlisten?.();
    errorUnlisten?.();
    startedSerials.forEach(serial => {
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
    entryCount={filtered.length}
  />

  <!-- Column header -->
  <div class="log-header">
    <span class="hdr-col" style="width:{$columnWidths.timestamp}px">
      {$tt('logcat.hdr_timestamp')}
      <span class="resize-handle" role="separator" aria-orientation="vertical" on:mousedown={(e) => onResizeStart('timestamp', e)} on:dblclick={() => onResizeDblClick('timestamp')}></span>
    </span>
    <span class="hdr-col" style="width:{$columnWidths.pidtid}px">
      {$tt('logcat.hdr_pidtid')}
      <span class="resize-handle" role="separator" aria-orientation="vertical" on:mousedown={(e) => onResizeStart('pidtid', e)} on:dblclick={() => onResizeDblClick('pidtid')}></span>
    </span>
    <span class="hdr-col" style="width:{$columnWidths.package_name}px">
      {$tt('logcat.hdr_package')}
      <span class="resize-handle" role="separator" aria-orientation="vertical" on:mousedown={(e) => onResizeStart('package_name', e)} on:dblclick={() => onResizeDblClick('package_name')}></span>
    </span>
    <span class="hdr-col" style="width:{$columnWidths.tag}px">
      {$tt('logcat.hdr_tag')}
      <span class="resize-handle" role="separator" aria-orientation="vertical" on:mousedown={(e) => onResizeStart('tag', e)} on:dblclick={() => onResizeDblClick('tag')}></span>
    </span>
    <span class="hdr-col" style="width:{$columnWidths.level}px">
      {$tt('logcat.hdr_level')}
      <span class="resize-handle" role="separator" aria-orientation="vertical" on:mousedown={(e) => onResizeStart('level', e)} on:dblclick={() => onResizeDblClick('level')}></span>
    </span>
    <span class="hdr-col hdr-message">{$tt('logcat.hdr_message')}</span>
  </div>

  <div
    class="log-container"
    class:wrap-mode={$wordWrap}
    bind:this={container}
    on:scroll={handleScroll}
  >
    <div class="log-scroll-spacer" style="height: {$wordWrap ? 'auto' : totalHeight + 'px'}">
      {#if $wordWrap}
        {#each filtered as entry, i (entry.id)}
          <LogcatRow {entry} wrap={true} even={i % 2 === 0} />
          {#if entry.stack_frames && entry.stack_frames.length > 0}
            <StackGroup frames={entry.stack_frames} />
          {/if}
        {/each}
      {:else}
        <div class="log-visible" style="transform: translateY({startIndex * ROW_HEIGHT}px)">
          {#each visibleEntries as entry, i (entry.id)}
            <LogcatRow {entry} wrap={false} even={(startIndex + i) % 2 === 0} />
            {#if entry.stack_frames && entry.stack_frames.length > 0}
              <StackGroup frames={entry.stack_frames} />
            {/if}
          {/each}
        </div>
      {/if}
    </div>
  </div>

  {#if filtered.length === 0 && allEntries.length > 0}
    <div class="empty-filter">{$tt('logcat.no_match')}</div>
  {:else if allEntries.length === 0}
    <div class="empty-filter">
      {currentSerial ? $tt('logcat.waiting') : $tt('logcat.select_device')}
      {#if logcatDiag.length > 0}
        <div class="logcat-diag">
          {#each logcatDiag as msg}
            <div class="diag-line">{msg}</div>
          {/each}
        </div>
      {/if}
      {#if currentSerial}
        <button class="retry-btn" on:click={() => { logcatDiag = []; restartLogcat(currentSerial); }}>
          Retry Logcat
        </button>
      {/if}
    </div>
  {/if}
</div>

<style>
  .logcat-viewer {
    display: flex;
    flex-direction: column;
    height: 100%;
  }
  .log-header {
    display: flex;
    align-items: center;
    padding: 0 10px;
    height: 22px;
    font-family: var(--font-ui);
    font-size: 11px;
    font-weight: 600;
    color: var(--text-secondary);
    background: var(--bg-tertiary);
    border-bottom: 1px solid var(--border-color);
    white-space: nowrap;
    flex-shrink: 0;
    text-transform: uppercase;
    letter-spacing: 0.3px;
  }
  .hdr-col {
    position: relative;
    flex-shrink: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    padding-right: 8px;
    box-sizing: border-box;
  }
  .hdr-message { flex: 1; min-width: 0; }
  .resize-handle {
    position: absolute;
    top: 0;
    right: 0;
    width: 5px;
    height: 100%;
    cursor: col-resize;
    z-index: 1;
  }
  .resize-handle:hover,
  .resize-handle:active {
    background: var(--accent);
    opacity: 0.5;
  }

  .log-container {
    flex: 1;
    overflow-y: auto;
    overflow-x: auto;
    background: var(--bg-primary);
  }
  .log-container.wrap-mode {
    overflow-x: hidden;
  }
  .log-scroll-spacer { position: relative; min-width: fit-content; }
  .log-visible { position: absolute; top: 0; left: 0; min-width: fit-content; }
  .empty-filter {
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    color: var(--text-secondary);
    font-size: 14px;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 10px;
    text-align: center;
  }
  .logcat-diag {
    font-size: 11px;
    font-family: var(--font-mono);
    color: var(--warning);
    background: rgba(255, 200, 50, 0.06);
    border: 1px solid rgba(255, 200, 50, 0.2);
    border-radius: var(--radius-sm);
    padding: 8px 12px;
    max-width: 500px;
    max-height: 160px;
    overflow-y: auto;
    text-align: left;
    word-break: break-all;
  }
  .diag-line {
    padding: 2px 0;
    border-bottom: 1px solid rgba(255, 200, 50, 0.08);
  }
  .diag-line:last-child { border-bottom: none; }
  .retry-btn {
    font-size: 12px;
    padding: 6px 16px;
    background: var(--accent);
    color: white;
    border: none;
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition: opacity 0.15s;
  }
  .retry-btn:hover { opacity: 0.85; }
</style>
