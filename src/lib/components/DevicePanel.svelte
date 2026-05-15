<script lang="ts">
  import { devices, activeDeviceSerial, onlineDevices } from '../stores/devices';
  import { disconnectDevice } from '../utils/tauri';
  import type { Device } from '../types';

  export let onWifiClick: () => void = () => {};

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
    <span class="label-text">Devices</span>
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
            title="Disconnect"
            on:click={(e) => handleDisconnect(e, device.serial)}
            on:keydown={(e) => { if (e.key === 'Enter') handleDisconnect(e, device.serial); }}
          >&times;</span>
        {/if}
      </button>
    {:else}
      <span class="empty-hint">No devices</span>
    {/each}
  </div>

  <button class="wifi-btn" on:click={onWifiClick} title="WiFi Connect">
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
      <path d="M5 12.55a11 11 0 0 1 14.08 0"></path>
      <path d="M1.42 9a16 16 0 0 1 21.16 0"></path>
      <path d="M8.53 16.11a6 6 0 0 1 6.95 0"></path>
      <line x1="12" y1="20" x2="12.01" y2="20"></line>
    </svg>
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
  .wifi-btn {
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
</style>
