<script lang="ts">
  import { installHistory } from '../stores/installer';
  import { tt } from '../i18n';

  function formatTime(ts: number): string {
    return new Date(ts).toLocaleString();
  }

  function clearHistory() {
    $installHistory = [];
  }
</script>

<div class="history">
  <div class="history-header">
    <span>{$tt('history.title')}</span>
    {#if $installHistory.length > 0}
      <button class="clear-btn" on:click={clearHistory} title={$tt('history.clear')}>&times;</button>
    {/if}
  </div>
  <div class="history-list">
    {#each $installHistory as record (record.id)}
      <div class="history-item">
        <span class="result-dot" class:success={record.result === 'success'} class:failed={record.result === 'failed'}></span>
        <div class="history-info">
          <span class="history-name">{record.filename}</span>
          <span class="history-meta">{record.device_model} · {formatTime(record.timestamp)}</span>
          {#if record.error}
            <span class="history-error">{record.error}</span>
          {/if}
        </div>
      </div>
    {:else}
      <div class="empty">{$tt('history.empty')}</div>
    {/each}
  </div>
</div>

<style>
  .history { display: flex; flex-direction: column; height: 100%; }
  .history-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    font-size: 12px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--text-secondary);
    padding: 8px 0;
  }
  .clear-btn {
    background: none;
    border: none;
    color: var(--text-secondary);
    font-size: 16px;
    cursor: pointer;
    padding: 0 4px;
    line-height: 1;
    border-radius: var(--radius-sm);
  }
  .clear-btn:hover {
    background: var(--bg-hover);
    color: var(--text-bright);
  }
  .history-list { flex: 1; overflow-y: auto; }
  .history-item {
    display: flex;
    align-items: flex-start;
    gap: 8px;
    padding: 8px 0;
    border-bottom: 1px solid var(--border-color);
  }
  .result-dot {
    width: 8px; height: 8px; border-radius: 50%;
    margin-top: 4px; flex-shrink: 0;
  }
  .result-dot.success { background: var(--success); }
  .result-dot.failed { background: var(--error); }
  .history-info { display: flex; flex-direction: column; gap: 2px; }
  .history-name { font-size: 13px; color: var(--text-primary); }
  .history-meta { font-size: 11px; color: var(--text-secondary); }
  .history-error { font-size: 11px; color: var(--error); }
  .empty { padding: 20px 0; text-align: center; color: var(--text-secondary); font-size: 12px; }
</style>
