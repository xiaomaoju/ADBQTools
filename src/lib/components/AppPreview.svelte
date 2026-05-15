<script lang="ts">
  import { formatFileSize } from '../utils/format';

  export let filename: string = '';
  export let filesize: number = 0;
  export let filetype: 'apk' | 'aab' = 'apk';
</script>

<div class="app-preview">
  <div class="app-icon">{filetype === 'apk' ? '📦' : '📋'}</div>
  <div class="app-info">
    <span class="app-name">{filename}</span>
    <span class="app-meta">{filetype.toUpperCase()} · {formatFileSize(filesize)}</span>
  </div>
</div>

<style>
  .app-preview {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 16px;
    background: var(--bg-tertiary);
    border-radius: var(--radius-md);
    border: 1px solid var(--border-color);
  }
  .app-icon { font-size: 32px; }
  .app-info { display: flex; flex-direction: column; gap: 2px; }
  .app-name { font-weight: 600; color: var(--text-bright); font-size: 14px; }
  .app-meta { font-size: 12px; color: var(--text-secondary); }
</style>
