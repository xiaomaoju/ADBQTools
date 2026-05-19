<script lang="ts">
  import type { PackageInfo } from '../types';
  import { tt } from '../i18n';

  export let open: boolean = false;
  export let info: PackageInfo | null = null;
  export let error: string = '';
  export let loading: boolean = false;

  let copiedField = '';

  function handleBackdrop(e: MouseEvent) {
    if (e.target === e.currentTarget) open = false;
  }

  function formatSize(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
    return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`;
  }

  function sdkToAndroid(sdk: string): string {
    const map: Record<string, string> = {
      '21': '5.0', '22': '5.1', '23': '6.0', '24': '7.0', '25': '7.1',
      '26': '8.0', '27': '8.1', '28': '9', '29': '10', '30': '11',
      '31': '12', '32': '12L', '33': '13', '34': '14', '35': '15',
    };
    const ver = map[sdk];
    return ver ? `${sdk} (Android ${ver})` : sdk;
  }

  async function copyText(text: string, field: string) {
    try {
      await navigator.clipboard.writeText(text);
      copiedField = field;
      setTimeout(() => { copiedField = ''; }, 1500);
    } catch { /* ignore */ }
  }

  function shortPermission(perm: string): string {
    return perm.replace('android.permission.', '');
  }
</script>

{#if open}
  <div class="backdrop" on:click={handleBackdrop} role="presentation">
    <div class="dialog">
      <div class="dialog-header">
        <span class="dialog-title">{$tt('pkginfo.title')}</span>
        <button class="close-btn" on:click={() => open = false}>&#10005;</button>
      </div>
      <div class="dialog-body">
        {#if loading}
          <div class="loading">
            <span class="spinner"></span>
            <span>{$tt('installer.parsing')}</span>
          </div>
        {:else if error}
          <div class="error-box">
            <span class="error-icon">&#10007;</span>
            <div class="error-content">
              <span class="error-title">{$tt('pkginfo.parse_error')}</span>
              <span class="error-msg">{error}</span>
            </div>
          </div>
        {:else if info}
          <!-- Basic Info -->
          <div class="section">
            <div class="section-title">{$tt('pkginfo.basic')}</div>
            <div class="info-grid">
              <div class="info-row">
                <span class="info-label">{$tt('pkginfo.package_name')}</span>
                <span class="info-value copyable" on:click={() => copyText(info?.package_name ?? '', 'pkg')} role="button" tabindex="0" on:keydown={() => {}}>
                  {info.package_name}
                  <span class="copy-hint">{copiedField === 'pkg' ? $tt('pkginfo.copied') : $tt('pkginfo.copy')}</span>
                </span>
              </div>
              <div class="info-row">
                <span class="info-label">{$tt('pkginfo.version_name')}</span>
                <span class="info-value">{info.version_name || '-'}</span>
              </div>
              <div class="info-row">
                <span class="info-label">{$tt('pkginfo.version_code')}</span>
                <span class="info-value">{info.version_code || '-'}</span>
              </div>
              <div class="info-row">
                <span class="info-label">{$tt('pkginfo.min_sdk')}</span>
                <span class="info-value">{sdkToAndroid(info.min_sdk) || '-'}</span>
              </div>
              <div class="info-row">
                <span class="info-label">{$tt('pkginfo.target_sdk')}</span>
                <span class="info-value">{sdkToAndroid(info.target_sdk) || '-'}</span>
              </div>
              {#if info.compile_sdk}
                <div class="info-row">
                  <span class="info-label">{$tt('pkginfo.compile_sdk')}</span>
                  <span class="info-value">{sdkToAndroid(info.compile_sdk)}</span>
                </div>
              {/if}
              <div class="info-row">
                <span class="info-label">{$tt('pkginfo.debuggable')}</span>
                <span class="info-value" class:flag-warn={info.debuggable}>
                  {info.debuggable ? $tt('pkginfo.yes') : $tt('pkginfo.no')}
                </span>
              </div>
              <div class="info-row">
                <span class="info-label">{$tt('pkginfo.allow_backup')}</span>
                <span class="info-value">{info.allow_backup ? $tt('pkginfo.yes') : $tt('pkginfo.no')}</span>
              </div>
            </div>
          </div>

          <!-- Structure -->
          <div class="section">
            <div class="section-title">{$tt('pkginfo.structure')}</div>
            <div class="info-grid">
              <div class="info-row">
                <span class="info-label">{$tt('pkginfo.file_name')}</span>
                <span class="info-value mono">{info.file_name}</span>
              </div>
              <div class="info-row">
                <span class="info-label">{$tt('pkginfo.file_size')}</span>
                <span class="info-value">{formatSize(info.file_size)}</span>
              </div>
              <div class="info-row">
                <span class="info-label">{$tt('pkginfo.uncompressed_size')}</span>
                <span class="info-value">{formatSize(info.total_uncompressed_size)}</span>
              </div>
              <div class="info-row">
                <span class="info-label">{$tt('pkginfo.file_type')}</span>
                <span class="info-value"><span class="badge">{info.file_type}</span></span>
              </div>
              <div class="info-row">
                <span class="info-label">{$tt('pkginfo.dex_count')}</span>
                <span class="info-value">{info.dex_count}</span>
              </div>
              <div class="info-row">
                <span class="info-label">{$tt('pkginfo.has_assets')}</span>
                <span class="info-value">{info.has_assets ? $tt('pkginfo.yes') : $tt('pkginfo.no')}</span>
              </div>
              <div class="info-row">
                <span class="info-label">{$tt('pkginfo.has_resources')}</span>
                <span class="info-value">{info.has_resources ? $tt('pkginfo.yes') : $tt('pkginfo.no')}</span>
              </div>
              {#if info.architectures.length > 0}
                <div class="info-row">
                  <span class="info-label">{$tt('pkginfo.architectures')}</span>
                  <span class="info-value">
                    {#each info.architectures as arch}
                      <span class="badge arch">{arch}</span>
                    {/each}
                  </span>
                </div>
              {/if}
            </div>
            {#if Object.keys(info.native_libs).length > 0}
              <div class="native-libs">
                <span class="info-label">{$tt('pkginfo.native_libs')}</span>
                {#each Object.entries(info.native_libs) as [arch, libs]}
                  <div class="lib-group">
                    <span class="lib-arch">{arch}</span>
                    <div class="lib-list">
                      {#each libs as lib}
                        <span class="lib-item">{lib}</span>
                      {/each}
                    </div>
                  </div>
                {/each}
              </div>
            {/if}
          </div>

          <!-- Permissions -->
          <div class="section">
            <div class="section-title">{$tt('pkginfo.permissions')} <span class="count">({info.permissions.length})</span></div>
            {#if info.permissions.length > 0}
              <div class="perm-list">
                {#each info.permissions as perm}
                  <span class="perm-item" title={perm}>{shortPermission(perm)}</span>
                {/each}
              </div>
            {:else}
              <div class="empty-hint">{$tt('pkginfo.no_permissions')}</div>
            {/if}
          </div>

          <!-- Signing -->
          <div class="section">
            <div class="section-title">{$tt('pkginfo.signing')}</div>
            {#if info.signing_info}
              <div class="info-grid">
                {#if info.signing_info.owner}
                  <div class="info-row">
                    <span class="info-label">{$tt('pkginfo.owner')}</span>
                    <span class="info-value mono small">{info.signing_info.owner}</span>
                  </div>
                {/if}
                {#if info.signing_info.issuer}
                  <div class="info-row">
                    <span class="info-label">{$tt('pkginfo.issuer')}</span>
                    <span class="info-value mono small">{info.signing_info.issuer}</span>
                  </div>
                {/if}
                {#if info.signing_info.serial_number}
                  <div class="info-row">
                    <span class="info-label">{$tt('pkginfo.serial')}</span>
                    <span class="info-value mono small">{info.signing_info.serial_number}</span>
                  </div>
                {/if}
                {#if info.signing_info.valid_from}
                  <div class="info-row">
                    <span class="info-label">{$tt('pkginfo.valid_from')}</span>
                    <span class="info-value">{info.signing_info.valid_from}</span>
                  </div>
                {/if}
                {#if info.signing_info.valid_to}
                  <div class="info-row">
                    <span class="info-label">{$tt('pkginfo.valid_to')}</span>
                    <span class="info-value">{info.signing_info.valid_to}</span>
                  </div>
                {/if}
                {#if info.signing_info.algorithm}
                  <div class="info-row">
                    <span class="info-label">{$tt('pkginfo.algorithm')}</span>
                    <span class="info-value">{info.signing_info.algorithm}</span>
                  </div>
                {/if}
                {#if info.signing_info.sha256}
                  <div class="info-row fingerprint">
                    <span class="info-label">SHA-256</span>
                    <span class="info-value mono small copyable" on:click={() => copyText(info?.signing_info?.sha256 ?? '', 'sha256')} role="button" tabindex="0" on:keydown={() => {}}>
                      {info.signing_info.sha256}
                      <span class="copy-hint">{copiedField === 'sha256' ? $tt('pkginfo.copied') : $tt('pkginfo.copy')}</span>
                    </span>
                  </div>
                {/if}
                {#if info.signing_info.sha1}
                  <div class="info-row fingerprint">
                    <span class="info-label">SHA-1</span>
                    <span class="info-value mono small copyable" on:click={() => copyText(info?.signing_info?.sha1 ?? '', 'sha1')} role="button" tabindex="0" on:keydown={() => {}}>
                      {info.signing_info.sha1}
                      <span class="copy-hint">{copiedField === 'sha1' ? $tt('pkginfo.copied') : $tt('pkginfo.copy')}</span>
                    </span>
                  </div>
                {/if}
                {#if info.signing_info.md5}
                  <div class="info-row fingerprint">
                    <span class="info-label">MD5</span>
                    <span class="info-value mono small copyable" on:click={() => copyText(info?.signing_info?.md5 ?? '', 'md5')} role="button" tabindex="0" on:keydown={() => {}}>
                      {info.signing_info.md5}
                      <span class="copy-hint">{copiedField === 'md5' ? $tt('pkginfo.copied') : $tt('pkginfo.copy')}</span>
                    </span>
                  </div>
                {/if}
              </div>
            {:else}
              <div class="empty-hint">{$tt('pkginfo.no_signing')}</div>
            {/if}
          </div>
        {/if}
      </div>
    </div>
  </div>
{/if}

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.55);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }
  .dialog {
    background: var(--bg-secondary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-lg);
    width: 580px;
    max-width: 92vw;
    max-height: 85vh;
    display: flex;
    flex-direction: column;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.5);
  }
  .dialog-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 16px;
    border-bottom: 1px solid var(--border-color);
    flex-shrink: 0;
  }
  .dialog-title { font-weight: 600; color: var(--text-bright); font-size: 14px; }
  .close-btn {
    background: none;
    border: none;
    color: var(--text-secondary);
    cursor: pointer;
    font-size: 16px;
    padding: 2px 6px;
    border-radius: var(--radius-sm);
  }
  .close-btn:hover { background: var(--bg-hover); color: var(--text-primary); }
  .dialog-body {
    padding: 16px;
    overflow-y: auto;
    flex: 1;
  }

  /* Loading */
  .loading {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 10px;
    padding: 40px 0;
    color: var(--text-secondary);
    font-size: 13px;
  }
  .spinner {
    display: inline-block;
    width: 16px;
    height: 16px;
    border: 2px solid rgba(255,255,255,0.15);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 0.6s linear infinite;
  }
  @keyframes spin { to { transform: rotate(360deg); } }

  /* Error */
  .error-box {
    display: flex;
    align-items: flex-start;
    gap: 10px;
    padding: 14px;
    background: rgba(255, 80, 80, 0.08);
    border: 1px solid rgba(255, 80, 80, 0.2);
    border-radius: var(--radius-md);
  }
  .error-icon { color: var(--error); font-size: 16px; }
  .error-content { display: flex; flex-direction: column; gap: 4px; }
  .error-title { font-weight: 600; color: var(--error); font-size: 13px; }
  .error-msg { font-size: 12px; color: var(--text-secondary); word-break: break-all; }

  /* Section */
  .section {
    margin-bottom: 16px;
  }
  .section:last-child { margin-bottom: 0; }
  .section-title {
    font-size: 11px;
    font-weight: 600;
    color: var(--accent);
    text-transform: uppercase;
    letter-spacing: 0.5px;
    margin-bottom: 8px;
    padding-bottom: 4px;
    border-bottom: 1px solid var(--border-color);
  }
  .count {
    font-weight: 400;
    color: var(--text-secondary);
    text-transform: none;
    letter-spacing: 0;
  }

  /* Info Grid */
  .info-grid {
    display: flex;
    flex-direction: column;
    gap: 1px;
  }
  .info-row {
    display: flex;
    align-items: flex-start;
    padding: 5px 0;
    gap: 12px;
  }
  .info-row.fingerprint {
    flex-direction: column;
    gap: 3px;
  }
  .info-label {
    font-size: 12px;
    color: var(--text-secondary);
    min-width: 90px;
    flex-shrink: 0;
  }
  .info-value {
    font-size: 12px;
    color: var(--text-bright);
    word-break: break-all;
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 4px;
  }
  .info-value.mono {
    font-family: 'SF Mono', 'Fira Code', monospace;
  }
  .info-value.small {
    font-size: 11px;
  }
  .info-value.copyable {
    cursor: pointer;
    position: relative;
    padding: 2px 4px;
    margin: -2px -4px;
    border-radius: var(--radius-sm);
    transition: background 0.15s;
  }
  .info-value.copyable:hover {
    background: var(--bg-hover);
  }
  .copy-hint {
    font-size: 10px;
    color: var(--accent);
    opacity: 0;
    transition: opacity 0.15s;
    margin-left: 4px;
    white-space: nowrap;
  }
  .info-value.copyable:hover .copy-hint {
    opacity: 1;
  }
  .flag-warn {
    color: #e8a838;
  }

  /* Badge */
  .badge {
    font-size: 10px;
    font-weight: 600;
    padding: 1px 6px;
    border-radius: 3px;
    background: var(--accent);
    color: white;
    letter-spacing: 0.3px;
  }
  .badge.arch {
    background: var(--bg-hover);
    color: var(--text-bright);
    border: 1px solid var(--border-color);
  }

  /* Native libs */
  .native-libs {
    margin-top: 8px;
  }
  .lib-group {
    margin-top: 6px;
    padding-left: 8px;
    border-left: 2px solid var(--border-color);
  }
  .lib-arch {
    font-size: 11px;
    font-weight: 600;
    color: var(--text-primary);
    margin-bottom: 3px;
    display: block;
  }
  .lib-list {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
  }
  .lib-item {
    font-size: 10px;
    font-family: 'SF Mono', 'Fira Code', monospace;
    padding: 1px 5px;
    background: var(--bg-tertiary);
    border: 1px solid var(--border-color);
    border-radius: 3px;
    color: var(--text-primary);
  }

  /* Permissions */
  .perm-list {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
    max-height: 160px;
    overflow-y: auto;
  }
  .perm-item {
    font-size: 10px;
    font-family: 'SF Mono', 'Fira Code', monospace;
    padding: 2px 6px;
    background: var(--bg-tertiary);
    border: 1px solid var(--border-color);
    border-radius: 3px;
    color: var(--text-primary);
    cursor: default;
  }
  .perm-item:hover {
    border-color: var(--accent);
  }

  .empty-hint {
    font-size: 12px;
    color: var(--text-secondary);
    opacity: 0.6;
    padding: 8px 0;
  }
</style>
