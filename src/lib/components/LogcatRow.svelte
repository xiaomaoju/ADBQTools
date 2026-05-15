<script lang="ts">
  import type { LogEntry } from '../types';

  export let entry: LogEntry;

  const levelColors: Record<string, string> = {
    verbose: 'var(--log-verbose)',
    debug: 'var(--log-debug)',
    info: 'var(--log-info)',
    warn: 'var(--log-warn)',
    error: 'var(--log-error)',
    fatal: 'var(--log-fatal)',
  };

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

<div class="log-row" style="color: {levelColors[entry.level]}">
  <span class="timestamp">{entry.timestamp}</span>
  <span class="pid">{entry.pid}</span>
  <span class="level">{levelLetters[entry.level]}</span>
  {#if entry.source !== 'system'}
    <span class="source-badge">{sourceIcon(entry.source)}</span>
  {/if}
  <span class="tag">{entry.tag}</span>
  <span class="message">{entry.message}</span>
</div>

<style>
  .log-row {
    display: flex;
    align-items: baseline;
    gap: 8px;
    padding: 1px 12px;
    font-family: var(--font-mono);
    font-size: 12px;
    line-height: 1.6;
    white-space: nowrap;
  }
  .log-row:hover { background: var(--bg-tertiary); }
  .timestamp { color: var(--text-secondary); min-width: 140px; }
  .pid { color: var(--text-secondary); min-width: 50px; text-align: right; }
  .level { min-width: 12px; font-weight: 700; }
  .source-badge {
    background: var(--accent);
    color: white;
    border-radius: 2px;
    padding: 0 3px;
    font-size: 10px;
    font-weight: 700;
    font-family: var(--font-ui);
  }
  .tag {
    color: var(--accent);
    min-width: 100px;
    max-width: 200px;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .message {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
  }
</style>
