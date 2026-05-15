<script lang="ts">
  import AppPreview from './AppPreview.svelte';
  import InstallHistory from './InstallHistory.svelte';
  import Button from './ui/Button.svelte';
  import Input from './ui/Input.svelte';
  import ProgressBar from './ui/ProgressBar.svelte';
  import { activeDevice, activeDeviceSerial } from '../stores/devices';
  import { installProgress, addInstallRecord, savedKeystore } from '../stores/installer';
  import { installApk, installAab, onInstallProgress } from '../utils/tauri';
  import { open } from '@tauri-apps/plugin-dialog';
  import type { KeystoreConfig } from '../types';
  import { onMount, onDestroy } from 'svelte';
  import { getCurrentWebview } from '@tauri-apps/api/webview';
  import type { UnlistenFn } from '@tauri-apps/api/event';

  let selectedFile: { name: string; path: string; size: number; type: 'apk' | 'aab' } | null = null;
  let isDragging = false;
  let installing = false;
  let showKeystoreConfig = false;
  let unlisten: UnlistenFn | null = null;
  let unlistenDrop: UnlistenFn | null = null;

  let keystoreForm: KeystoreConfig = {
    path: '',
    alias: '',
    store_password: '',
    key_password: '',
  };

  onMount(async () => {
    unlisten = await onInstallProgress((progress) => {
      $installProgress = progress;
      if (progress.stage === 'complete' || progress.stage === 'failed') {
        installing = false;
        if (selectedFile && $activeDevice) {
          addInstallRecord(
            selectedFile.name,
            $activeDevice.serial,
            $activeDevice.model,
            progress.stage === 'complete' ? 'success' : 'failed',
            progress.stage === 'failed' ? progress.message : undefined
          );
        }
      }
    });

    if ($savedKeystore) {
      keystoreForm = { ...$savedKeystore };
    }

    unlistenDrop = await getCurrentWebview().onDragDropEvent((event) => {
      if (event.payload.type === 'over') {
        isDragging = true;
      } else if (event.payload.type === 'drop') {
        isDragging = false;
        const paths: string[] = event.payload.paths;
        const filePath = paths.find(p => p.endsWith('.apk') || p.endsWith('.aab'));
        if (filePath) {
          const name = filePath.split(/[/\\]/).pop() ?? filePath;
          const type = name.endsWith('.aab') ? 'aab' : 'apk';
          selectedFile = { name, path: filePath, size: 0, type };
        }
      } else if (event.payload.type === 'leave') {
        isDragging = false;
      }
    });
  });

  onDestroy(() => {
    unlisten?.();
    unlistenDrop?.();
  });

  async function selectFile() {
    const path = await open({
      filters: [{ name: 'Android Package', extensions: ['apk', 'aab'] }],
    });
    if (path) {
      const name = (path as string).split(/[/\\]/).pop() ?? (path as string);
      const type = name.endsWith('.aab') ? 'aab' : 'apk';
      selectedFile = { name, path: path as string, size: 0, type };
    }
  }


  async function handleInstall() {
    if (!selectedFile || !$activeDeviceSerial) return;
    installing = true;
    $installProgress = { stage: 'installing', message: 'Starting...' };

    try {
      if (selectedFile.type === 'apk') {
        await installApk($activeDeviceSerial, selectedFile.path);
      } else {
        const ks = keystoreForm.path ? keystoreForm : undefined;
        if (ks) $savedKeystore = { ...keystoreForm };
        await installAab($activeDeviceSerial, selectedFile.path, ks);
      }
    } catch (e) {
      $installProgress = { stage: 'failed', message: String(e) };
      installing = false;
    }
  }
</script>

<div class="installer">
  <div class="install-area">
    <div
      class="drop-zone"
      class:dragging={isDragging}
      role="button"
      tabindex="0"
      on:click={selectFile}
    >
      {#if selectedFile}
        <AppPreview
          filename={selectedFile.name}
          filesize={selectedFile.size}
          filetype={selectedFile.type}
        />
      {:else}
        <div class="drop-hint">
          <span class="drop-icon">📱</span>
          <span>Drag APK/AAB here or click to browse</span>
        </div>
      {/if}
    </div>

    <div class="install-controls">
      <Button variant="secondary" on:click={selectFile}>Browse Files</Button>

      {#if selectedFile?.type === 'aab'}
        <Button size="sm" variant="ghost" on:click={() => showKeystoreConfig = !showKeystoreConfig}>
          Keystore Config {showKeystoreConfig ? '▲' : '▼'}
        </Button>
      {/if}

      <Button
        variant="primary"
        disabled={!selectedFile || !$activeDeviceSerial || installing}
        on:click={handleInstall}
      >
        {installing ? 'Installing...' : 'Install'}
      </Button>
    </div>

    {#if showKeystoreConfig && selectedFile?.type === 'aab'}
      <div class="keystore-config">
        <Input bind:value={keystoreForm.path} placeholder="Keystore path" size="sm" />
        <Input bind:value={keystoreForm.alias} placeholder="Key alias" size="sm" />
        <Input bind:value={keystoreForm.store_password} placeholder="Store password" type="password" size="sm" />
        <Input bind:value={keystoreForm.key_password} placeholder="Key password" type="password" size="sm" />
      </div>
    {/if}

    {#if $installProgress}
      <div class="progress-section">
        <ProgressBar indeterminate={installing} />
        <span class="progress-text" class:error={$installProgress.stage === 'failed'} class:success={$installProgress.stage === 'complete'}>
          {$installProgress.message}
        </span>
      </div>
    {/if}
  </div>

  <div class="history-section">
    <InstallHistory />
  </div>
</div>

<style>
  .installer {
    display: flex;
    flex-direction: column;
    height: 100%;
    padding: 20px;
    gap: 20px;
  }
  .install-area {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .drop-zone {
    border: 2px dashed var(--border-color);
    border-radius: var(--radius-lg);
    padding: 32px;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: all 0.2s;
    min-height: 120px;
  }
  .drop-zone:hover, .drop-zone.dragging {
    border-color: var(--accent);
    background: rgba(0, 122, 204, 0.05);
  }
  .drop-hint {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    color: var(--text-secondary);
  }
  .drop-icon { font-size: 32px; }
  .install-controls {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .keystore-config {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 8px;
    padding: 12px;
    background: var(--bg-tertiary);
    border-radius: var(--radius-md);
  }
  .progress-section {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .progress-text { font-size: 12px; color: var(--text-secondary); }
  .progress-text.error { color: var(--error); }
  .progress-text.success { color: var(--success); }
  .history-section { flex: 1; min-height: 0; overflow: hidden; }
</style>
