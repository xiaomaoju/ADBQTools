<script lang="ts">
  import type { LogEntry } from '../types';

  export let entry: LogEntry;
  export let wrap: boolean = false;
  export let even: boolean = false;

  const levelLetters: Record<string, string> = {
    verbose: 'V', debug: 'D', info: 'I', warn: 'W', error: 'E', fatal: 'F',
  };

  function sourceIcon(source: string): string {
    switch (source) {
      case 'unity': return 'U';
      case 'il2cpp': return 'IL';
      case 'mono': return 'M';
      default: return '';
    }
  }
</script>

<div
  class="log-row level-{entry.level}"
  class:wrap
  class:even
  class:is-error={entry.level === 'error' || entry.level === 'fatal'}
>
  <span class="col-timestamp">{entry.timestamp}</span>
  <span class="col-pidtid">{entry.pid}-{entry.tid}</span>
  <span class="col-tag" title={entry.tag}>{entry.tag}</span>
  {#if entry.source !== 'system'}
    <span class="col-source source-{entry.source}">{sourceIcon(entry.source)}</span>
  {/if}
  <span class="col-level">
    <span class="level-badge level-bg-{entry.level}">{levelLetters[entry.level]}</span>
  </span>
  <span class="col-message">{entry.message}</span>
</div>

<style>
  .log-row {
    display: flex;
    align-items: baseline;
    padding: 0 10px;
    font-family: var(--font-mono);
    font-size: 12px;
    line-height: 20px;
    white-space: nowrap;
    border-bottom: 1px solid transparent;
  }
  .log-row.even {
    background: rgba(255, 255, 255, 0.015);
  }
  .log-row:hover {
    background: rgba(255, 255, 255, 0.04);
  }

  /* Error/Fatal rows get a subtle tinted background */
  .log-row.is-error {
    background: rgba(244, 71, 71, 0.06);
  }
  .log-row.is-error.even {
    background: rgba(244, 71, 71, 0.08);
  }
  .log-row.is-error:hover {
    background: rgba(244, 71, 71, 0.12);
  }

  /* Columns — fixed widths matching Android Studio proportions */
  .col-timestamp {
    color: var(--text-secondary);
    min-width: 175px;
    max-width: 175px;
    overflow: hidden;
    text-overflow: ellipsis;
    flex-shrink: 0;
    padding-right: 8px;
  }
  .col-pidtid {
    color: var(--text-secondary);
    min-width: 90px;
    max-width: 90px;
    overflow: hidden;
    text-overflow: ellipsis;
    flex-shrink: 0;
    padding-right: 8px;
  }
  .col-tag {
    min-width: 140px;
    max-width: 180px;
    overflow: hidden;
    text-overflow: ellipsis;
    flex-shrink: 0;
    font-weight: 600;
    padding-right: 8px;
  }
  /* Tag colors per level (Android Studio style) */
  .level-verbose .col-tag { color: #888; }
  .level-debug .col-tag   { color: #4fc1ff; }
  .level-info .col-tag    { color: #4ec9b0; }
  .level-warn .col-tag    { color: #cca700; }
  .level-error .col-tag   { color: #f44747; }
  .level-fatal .col-tag   { color: #c586c0; }

  /* Unity / IL2CPP / Mono source badge */
  .col-source {
    flex-shrink: 0;
    padding: 0 4px;
    margin-right: 6px;
    border-radius: 3px;
    font-size: 10px;
    font-weight: 700;
    font-family: var(--font-ui);
    line-height: 16px;
    vertical-align: middle;
  }
  .source-unity {
    background: #3d8b37;
    color: #fff;
  }
  .source-il2cpp {
    background: #7b5ea7;
    color: #fff;
  }
  .source-mono {
    background: #2a6fb5;
    color: #fff;
  }

  /* Level badge — colored pill like Android Studio */
  .col-level {
    flex-shrink: 0;
    min-width: 22px;
    max-width: 22px;
    display: flex;
    align-items: center;
    justify-content: center;
    margin-right: 8px;
  }
  .level-badge {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 18px;
    height: 14px;
    border-radius: 2px;
    font-size: 10px;
    font-weight: 800;
    font-family: var(--font-ui);
    color: #fff;
    line-height: 1;
  }
  .level-bg-verbose { background: #6a6a6a; }
  .level-bg-debug   { background: #3369cc; }
  .level-bg-info    { background: #2e8b57; }
  .level-bg-warn    { background: #b8860b; }
  .level-bg-error   { background: #c62828; }
  .level-bg-fatal   { background: #8e24aa; }

  /* Message */
  .col-message {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    color: var(--text-primary);
  }
  .level-verbose .col-message { color: #888; }
  .level-debug .col-message   { color: var(--text-primary); }
  .level-info .col-message    { color: var(--text-primary); }
  .level-warn .col-message    { color: #cca700; }
  .level-error .col-message   { color: #f44747; }
  .level-fatal .col-message   { color: #c586c0; }

  /* Wrap mode */
  .log-row.wrap {
    white-space: normal;
    min-height: 20px;
    height: auto;
    align-items: flex-start;
  }
  .log-row.wrap .col-message {
    overflow: visible;
    text-overflow: unset;
    word-break: break-all;
  }
</style>
