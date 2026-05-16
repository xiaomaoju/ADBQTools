<script lang="ts">
  import { devices, activeDeviceSerial, onlineDevices } from '../stores/devices';
  import { disconnectDevice, restartAdb } from '../utils/tauri';
  import { tt } from '../i18n';
  import type { Device } from '../types';

  export let onWifiClick: () => void = () => {};
  let restarting = false;

  async function handleDisconnect(e: Event, serial: string) {
    e.stopPropagation();
    try {
      await disconnectDevice(serial);
    } catch (err) {
      console.error('Disconnect failed:', err);
    }
  }

  function selectDevice(serial: string) {
    $activeDeviceSerial = serial;
  }

  function transportIcon(d: Device): string {
    return d.transport === 'usb' ? '🔌' : '📶';
  }

  function statusColor(d: Device): string {
    switch (d.status) {
      case 'online': return 'var(--success)';
      case 'offline': return 'var(--text-secondary)';
      case 'unauthorized': return 'var(--warning)';
    }
  }
</script>

<div class="device-bar">
  <div class="bar-label">
    <span class="label-text">{$tt('devices.label')}</span>
    <span class="device-count">{$onlineDevices.length}</span>
  </div>

  <div class="device-list">
    {#each $devices as device (device.serial)}
      <button
        class="device-chip"
        class:active={$activeDeviceSerial === device.serial}
        on:click={() => selectDevice(device.serial)}
        title={device.serial}
      >
        <span class="status-dot" style="background: {statusColor(device)}"></span>
        <span class="chip-icon">{transportIcon(device)}</span>
        <span class="chip-label">{device.model || device.serial}</span>
        {#if device.transport === 'wifi'}
          <span
            class="chip-disconnect"
            role="button"
            tabindex="0"
            title={$tt('devices.disconnect')}
            on:click={(e) => handleDisconnect(e, device.serial)}
            on:keydown={(e) => { if (e.key === 'Enter') handleDisconnect(e, device.serial); }}
          >&times;</span>
        {/if}
      </button>
    {:else}
      <span class="empty-hint">{$tt('devices.none')}</span>
    {/each}
  </div>

  <button class="bar-btn wifi-btn" on:click={onWifiClick}>
    <span class="has-tooltip">
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <path d="M5 12.55a11 11 0 0 1 14.08 0"></path>
        <path d="M1.42 9a16 16 0 0 1 21.16 0"></path>
        <path d="M8.53 16.11a6 6 0 0 1 6.95 0"></path>
        <line x1="12" y1="20" x2="12.01" y2="20"></line>
      </svg>
      <span class="tooltip">{$tt('devices.wifi_connect')}</span>
    </span>
  </button>
  <button
    class="bar-btn restart-btn"
    class:restarting
    disabled={restarting}
    on:click={async () => {
      restarting = true;
      try { await restartAdb(); } catch (e) { console.error('restart adb:', e); }
      restarting = false;
    }}
  >
    <span class="has-tooltip">
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class:spin={restarting}>
        <polyline points="23 4 23 10 17 10"></polyline>
        <polyline points="1 20 1 14 7 14"></polyline>
        <path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10"></path>
        <path d="M20.49 15a9 9 0 0 1-14.85 3.36L1 14"></path>
      </svg>
      <span class="tooltip">{$tt('devices.restart_adb')}</span>
    </span>
  </button>
</div>

<style>
  .device-bar {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 12px;
    border-right: 1px solid var(--border-color);
    flex-shrink: 0;
  }
  .bar-label {
    display: flex;
    align-items: center;
    gap: 5px;
    flex-shrink: 0;
  }
  .label-text {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--text-secondary);
  }
  .device-count {
    background: var(--accent);
    color: white;
    border-radius: 8px;
    padding: 0 5px;
    font-size: 10px;
    min-width: 16px;
    text-align: center;
    line-height: 16px;
  }
  .device-list {
    display: flex;
    align-items: center;
    gap: 4px;
    overflow-x: auto;
    flex-shrink: 1;
    min-width: 0;
  }
  .device-list::-webkit-scrollbar { height: 0; }
  .device-chip {
    display: flex;
    align-items: center;
    gap: 5px;
    padding: 3px 10px;
    background: var(--bg-primary);
    border: 1px solid var(--border-color);
    border-radius: 14px;
    color: var(--text-primary);
    cursor: pointer;
    font-family: var(--font-ui);
    font-size: 12px;
    white-space: nowrap;
    transition: all 0.15s;
    flex-shrink: 0;
  }
  .device-chip:hover {
    border-color: var(--accent);
    background: var(--bg-hover);
  }
  .device-chip.active {
    border-color: var(--accent);
    background: rgba(0, 122, 204, 0.12);
    color: var(--accent);
  }
  .status-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    flex-shrink: 0;
  }
  .chip-icon {
    font-size: 11px;
    line-height: 1;
  }
  .chip-label {
    max-width: 120px;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .chip-disconnect {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 14px;
    height: 14px;
    border-radius: 50%;
    font-size: 12px;
    line-height: 1;
    color: var(--text-secondary);
    cursor: pointer;
    transition: all 0.12s;
    margin-left: 2px;
    flex-shrink: 0;
  }
  .chip-disconnect:hover {
    background: rgba(244, 71, 71, 0.2);
    color: var(--error);
  }
  .empty-hint {
    font-size: 11px;
    color: var(--text-secondary);
    opacity: 0.6;
    white-space: nowrap;
  }
  .bar-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    background: none;
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    color: var(--text-secondary);
    cursor: pointer;
    flex-shrink: 0;
    transition: all 0.15s;
  }
  .wifi-btn:hover {
    border-color: var(--accent);
    color: var(--accent);
    background: var(--bg-hover);
  }
  .restart-btn:hover {
    border-color: var(--warning);
    color: var(--warning);
    background: var(--bg-hover);
  }
  .restart-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  .spin {
    animation: spin 0.8s linear infinite;
  }
  @keyframes spin { to { transform: rotate(360deg); } }

  /* Tooltip system */
  .has-tooltip {
    position: relative;
    display: inline-flex;
    align-items: center;
    gap: 3px;
  }
  .tooltip {
    position: absolute;
    top: calc(100% + 8px);
    left: 50%;
    transform: translateX(-50%);
    background: #1a1a1a;
    color: #e0e0e0;
    font-family: var(--font-ui);
    font-size: 11px;
    font-weight: 400;
    padding: 4px 8px;
    border-radius: 4px;
    white-space: nowrap;
    pointer-events: none;
    opacity: 0;
    transition: opacity 0.15s ease;
    transition-delay: 0s;
    z-index: 200;
    border: 1px solid var(--border-color);
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.4);
    line-height: 1.3;
  }
  .tooltip::after {
    content: '';
    position: absolute;
    bottom: 100%;
    left: 50%;
    transform: translateX(-50%);
    border: 4px solid transparent;
    border-bottom-color: #1a1a1a;
  }
  .has-tooltip:hover .tooltip {
    opacity: 1;
    transition-delay: 0.5s;
  }
</style>
