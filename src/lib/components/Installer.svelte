<script lang="ts">
  import AppPreview from './AppPreview.svelte';
  import InstallHistory from './InstallHistory.svelte';
  import Button from './ui/Button.svelte';
  import Input from './ui/Input.svelte';
  import ProgressBar from './ui/ProgressBar.svelte';
  import { activeDevice, activeDeviceSerial } from '../stores/devices';
  import { installProgress, addInstallRecord, savedKeystore } from '../stores/installer';
  import { installApk, installAab, onInstallProgress, listKeystoreAliases } from '../utils/tauri';
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

  let keystoreAliases: string[] = [];
  let loadingAliases = false;
  let aliasError = '';

  $: keystoreFilename = keystoreForm.path ? keystoreForm.path.split(/[/\\]/).pop() : '';

  // Auto-fetch aliases when keystore path and store password are both set
  $: if (keystoreForm.path && keystoreForm.store_password) {
    fetchAliases(keystoreForm.path, keystoreForm.store_password);
  } else {
    keystoreAliases = [];
    keystoreForm.alias = '';
    aliasError = '';
  }

  let fetchAliasesTimer: ReturnType<typeof setTimeout> | null = null;
  function fetchAliases(path: string, password: string) {
    // Debounce to avoid calling on every keystroke
    if (fetchAliasesTimer) clearTimeout(fetchAliasesTimer);
    fetchAliasesTimer = setTimeout(async () => {
      loadingAliases = true;
      aliasError = '';
      try {
        const aliases = await listKeystoreAliases(path, password);
        keystoreAliases = aliases;
        if (aliases.length === 1) {
          keystoreForm.alias = aliases[0];
        } else if (!aliases.includes(keystoreForm.alias)) {
          keystoreForm.alias = '';
        }
      } catch (e) {
        keystoreAliases = [];
        keystoreForm.alias = '';
        aliasError = String(e).replace(/^keytool error:\s*/i, '');
      } finally {
        loadingAliases = false;
      }
    }, 500);
  }

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

  async function selectKeystore() {
    const path = await open({
      filters: [{ name: 'Keystore', extensions: ['jks', 'keystore'] }],
    });
    if (path) {
      keystoreForm.path = path as string;
    }
  }

  function clearFile() {
    selectedFile = null;
    $installProgress = null;
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
    <!-- Drop Zone -->
    <div
      class="drop-zone"
      class:dragging={isDragging}
      class:has-file={!!selectedFile}
      role="button"
      tabindex="0"
      on:click={selectFile}
      on:keydown={(e) => { if (e.key === 'Enter' || e.key === ' ') selectFile(); }}
    >
      {#if selectedFile}
        <div class="file-selected">
          <div class="file-icon" class:aab={selectedFile.type === 'aab'}>
            {selectedFile.type === 'apk' ? '📦' : '📋'}
          </div>
          <div class="file-info">
            <span class="file-name">{selectedFile.name}</span>
            <span class="file-badge">{selectedFile.type.toUpperCase()}</span>
          </div>
          <button class="file-clear" on:click|stopPropagation={clearFile} title="Remove">
            &times;
          </button>
        </div>
      {:else}
        <div class="drop-hint">
          <div class="drop-icon-wrapper">
            <svg width="40" height="40" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
              <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
              <polyline points="17 8 12 3 7 8" />
              <line x1="12" y1="3" x2="12" y2="15" />
            </svg>
          </div>
          <span class="drop-title">Drag APK / AAB here</span>
          <span class="drop-sub">or click to browse files</span>
        </div>
      {/if}
    </div>

    <!-- Action Bar -->
    <div class="action-bar">
      <div class="action-left">
        <Button variant="secondary" on:click={selectFile}>Browse Files</Button>
        {#if selectedFile?.type === 'aab'}
          <button
            class="keystore-toggle"
            class:active={showKeystoreConfig}
            on:click={() => showKeystoreConfig = !showKeystoreConfig}
          >
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <rect x="3" y="11" width="18" height="11" rx="2" ry="2"></rect>
              <path d="M7 11V7a5 5 0 0 1 10 0v4"></path>
            </svg>
            Signing
            <span class="toggle-arrow">{showKeystoreConfig ? '▲' : '▼'}</span>
          </button>
        {/if}
      </div>
      <Button
        variant="primary"
        disabled={!selectedFile || !$activeDeviceSerial || installing}
        on:click={handleInstall}
      >
        {#if installing}
          <span class="spinner"></span> Installing...
        {:else}
          Install
        {/if}
      </Button>
    </div>

    <!-- Keystore Config Panel -->
    {#if showKeystoreConfig && selectedFile?.type === 'aab'}
      <div class="keystore-panel">
        <div class="keystore-header">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <rect x="3" y="11" width="18" height="11" rx="2" ry="2"></rect>
            <path d="M7 11V7a5 5 0 0 1 10 0v4"></path>
          </svg>
          <span>Signing Configuration</span>
        </div>
        <div class="keystore-grid">
          <div class="keystore-field full-width">
            <!-- svelte-ignore a11y_label_has_associated_control -->
            <label>Keystore File <span class="label-hint">(.jks / .keystore)</span></label>
            <div class="keystore-file-row">
              <div class="keystore-file-display" on:click={selectKeystore} on:keydown={(e) => { if (e.key === 'Enter' || e.key === ' ') selectKeystore(); }} role="button" tabindex="0">
                {#if keystoreFilename}
                  <span class="keystore-filename">{keystoreFilename}</span>
                {:else}
                  <span class="keystore-placeholder">Select keystore file...</span>
                {/if}
              </div>
              <button class="keystore-browse-btn" on:click={selectKeystore}>Browse</button>
            </div>
          </div>
          <div class="keystore-field">
            <label for="alias-select">Key Alias</label>
            {#if loadingAliases}
              <div class="alias-loading">
                <span class="spinner small"></span> Reading aliases...
              </div>
            {:else if aliasError}
              <div class="alias-error" title={aliasError}>
                {aliasError.length > 40 ? aliasError.slice(0, 40) + '...' : aliasError}
              </div>
            {:else if keystoreAliases.length > 0}
              <select id="alias-select" class="alias-select" bind:value={keystoreForm.alias}>
                {#if keystoreAliases.length > 1}
                  <option value="" disabled>Select an alias...</option>
                {/if}
                {#each keystoreAliases as alias}
                  <option value={alias}>{alias}</option>
                {/each}
              </select>
            {:else}
              <div class="alias-placeholder">Enter keystore file &amp; password first</div>
            {/if}
          </div>
          <div class="keystore-field">
            <!-- svelte-ignore a11y_label_has_associated_control -->
            <label>Store Password</label>
            <Input bind:value={keystoreForm.store_password} placeholder="Store password" type="password" size="sm" />
          </div>
          <div class="keystore-field">
            <!-- svelte-ignore a11y_label_has_associated_control -->
            <label>Key Password</label>
            <Input bind:value={keystoreForm.key_password} placeholder="Key password" type="password" size="sm" />
          </div>
        </div>
      </div>
    {/if}

    <!-- Progress -->
    {#if $installProgress}
      <div class="progress-section">
        <ProgressBar progress={$installProgress.stage === 'complete' ? 100 : 0} indeterminate={installing && $installProgress.stage !== 'complete'} />
        <div class="progress-info">
          <span class="progress-stage"
            class:error={$installProgress.stage === 'failed'}
            class:success={$installProgress.stage === 'complete'}
          >
            {#if $installProgress.stage === 'complete'}
              &#10003; Installed successfully
            {:else if $installProgress.stage === 'failed'}
              &#10007; {$installProgress.message}
            {:else}
              {$installProgress.message}
            {/if}
          </span>
        </div>
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
    gap: 16px;
    overflow-y: auto;
  }
  .install-area {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  /* Drop Zone */
  .drop-zone {
    border: 2px dashed var(--border-color);
    border-radius: var(--radius-lg);
    padding: 32px 24px;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: all 0.2s ease;
    min-height: 120px;
    background: transparent;
  }
  .drop-zone:hover {
    border-color: var(--accent);
    background: rgba(0, 122, 204, 0.04);
  }
  .drop-zone.dragging {
    border-color: var(--accent);
    background: rgba(0, 122, 204, 0.08);
    border-style: solid;
  }
  .drop-zone.has-file {
    border-style: solid;
    border-color: var(--border-color);
    padding: 16px 20px;
    min-height: auto;
    cursor: default;
  }

  /* Drop Hint */
  .drop-hint {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 10px;
    color: var(--text-secondary);
  }
  .drop-icon-wrapper {
    width: 56px;
    height: 56px;
    border-radius: 50%;
    background: var(--bg-tertiary);
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-secondary);
    margin-bottom: 4px;
  }
  .drop-zone:hover .drop-icon-wrapper,
  .drop-zone.dragging .drop-icon-wrapper {
    background: rgba(0, 122, 204, 0.12);
    color: var(--accent);
  }
  .drop-title {
    font-size: 14px;
    font-weight: 500;
    color: var(--text-primary);
  }
  .drop-sub {
    font-size: 12px;
    color: var(--text-secondary);
  }

  /* File Selected */
  .file-selected {
    display: flex;
    align-items: center;
    gap: 14px;
    width: 100%;
  }
  .file-icon {
    font-size: 28px;
    width: 44px;
    height: 44px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--bg-tertiary);
    border-radius: var(--radius-md);
    flex-shrink: 0;
  }
  .file-info {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 10px;
    min-width: 0;
  }
  .file-name {
    font-weight: 600;
    color: var(--text-bright);
    font-size: 13px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .file-badge {
    font-size: 10px;
    font-weight: 700;
    padding: 2px 6px;
    border-radius: 3px;
    background: var(--accent);
    color: white;
    flex-shrink: 0;
    letter-spacing: 0.5px;
  }
  .file-clear {
    background: none;
    border: none;
    color: var(--text-secondary);
    font-size: 20px;
    cursor: pointer;
    padding: 4px 8px;
    border-radius: var(--radius-sm);
    line-height: 1;
    flex-shrink: 0;
  }
  .file-clear:hover {
    background: var(--bg-hover);
    color: var(--text-bright);
  }

  /* Action Bar */
  .action-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
  .action-left {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .keystore-toggle {
    display: flex;
    align-items: center;
    gap: 6px;
    background: none;
    border: 1px solid var(--border-color);
    color: var(--text-secondary);
    font-size: 12px;
    padding: 6px 10px;
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition: all 0.15s;
  }
  .keystore-toggle:hover, .keystore-toggle.active {
    border-color: var(--accent);
    color: var(--accent);
  }
  .toggle-arrow {
    font-size: 9px;
    opacity: 0.6;
  }
  .spinner {
    display: inline-block;
    width: 12px;
    height: 12px;
    border: 2px solid rgba(255,255,255,0.3);
    border-top-color: white;
    border-radius: 50%;
    animation: spin 0.6s linear infinite;
    vertical-align: middle;
    margin-right: 4px;
  }
  @keyframes spin { to { transform: rotate(360deg); } }

  /* Keystore Panel */
  .keystore-panel {
    background: var(--bg-tertiary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-md);
    padding: 14px 16px;
    animation: slideDown 0.15s ease;
  }
  @keyframes slideDown {
    from { opacity: 0; transform: translateY(-6px); }
    to { opacity: 1; transform: translateY(0); }
  }
  .keystore-header {
    display: flex;
    align-items: center;
    gap: 8px;
    color: var(--text-secondary);
    font-size: 12px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    margin-bottom: 12px;
  }
  .keystore-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 10px;
  }
  .keystore-field {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .keystore-field.full-width {
    grid-column: 1 / -1;
  }
  .keystore-field label {
    font-size: 11px;
    color: var(--text-secondary);
    font-weight: 500;
  }
  .label-hint {
    opacity: 0.6;
    font-weight: 400;
  }
  .keystore-file-row {
    display: flex;
    gap: 6px;
  }
  .keystore-file-display {
    flex: 1;
    display: flex;
    align-items: center;
    padding: 0 10px;
    height: 30px;
    background: var(--bg-secondary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-size: 12px;
    overflow: hidden;
  }
  .keystore-file-display:hover {
    border-color: var(--accent);
  }
  .keystore-filename {
    color: var(--text-bright);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .keystore-placeholder {
    color: var(--text-secondary);
    opacity: 0.6;
  }
  .keystore-browse-btn {
    padding: 0 12px;
    height: 30px;
    background: var(--bg-hover);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    color: var(--text-primary);
    font-size: 12px;
    cursor: pointer;
    white-space: nowrap;
    transition: all 0.15s;
  }
  .keystore-browse-btn:hover {
    background: var(--border-color);
  }

  /* Alias Select */
  .alias-select {
    height: 30px;
    padding: 0 8px;
    background: var(--bg-secondary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    color: var(--text-bright);
    font-size: 12px;
    font-family: inherit;
    cursor: pointer;
    width: 100%;
  }
  .alias-select:focus {
    border-color: var(--accent);
    outline: none;
  }
  .alias-loading {
    display: flex;
    align-items: center;
    gap: 6px;
    height: 30px;
    padding: 0 10px;
    font-size: 12px;
    color: var(--text-secondary);
  }
  .spinner.small {
    width: 10px;
    height: 10px;
    border-width: 1.5px;
    border-color: rgba(255,255,255,0.2);
    border-top-color: var(--accent);
    margin-right: 0;
  }
  .alias-error {
    height: 30px;
    display: flex;
    align-items: center;
    padding: 0 10px;
    font-size: 11px;
    color: var(--error);
    background: rgba(255, 80, 80, 0.08);
    border: 1px solid rgba(255, 80, 80, 0.2);
    border-radius: var(--radius-sm);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .alias-placeholder {
    height: 30px;
    display: flex;
    align-items: center;
    padding: 0 10px;
    font-size: 12px;
    color: var(--text-secondary);
    opacity: 0.5;
    background: var(--bg-secondary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
  }

  /* Progress */
  .progress-section {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 10px 12px;
    background: var(--bg-tertiary);
    border-radius: var(--radius-md);
  }
  .progress-info {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
  .progress-stage {
    font-size: 12px;
    color: var(--text-secondary);
  }
  .progress-stage.error { color: var(--error); }
  .progress-stage.success { color: var(--success); }

  .history-section { flex: 1; min-height: 0; overflow: hidden; }
</style>
