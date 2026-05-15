<script lang="ts">
  import { devices, activeDeviceSerial, onlineDevices } from '../stores/devices';
  import type { Device } from '../types';
  import Button from './ui/Button.svelte';

  export let onWifiClick: () => void = () => {};

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

<div class="device-panel">
  <div class="panel-header">
    <span class="panel-title">Devices</span>
    <span class="device-count">{$onlineDevices.length}</span>
  </div>

  <div class="device-list">
    {#each $devices as device (device.serial)}
      <button
        class="device-item"
        class:active={$activeDeviceSerial === device.serial}
        on:click={() => selectDevice(device.serial)}
      >
        <span class="status-dot" style="background: {statusColor(device)}" />
        <div class="device-info">
          <span class="device-model">{device.model || device.serial}</span>
          <span class="device-serial">{transportIcon(device)} {device.serial}</span>
        </div>
      </button>
    {:else}
      <div class="empty-state">No devices connected</div>
    {/each}
  </div>

  <div class="panel-footer">
    <Button size="sm" on:click={onWifiClick}>WiFi Connect</Button>
  </div>
</div>

<style>
  .device-panel {
    display: flex;
    flex-direction: column;
    height: 100%;
  }
  .panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 12px;
    border-bottom: 1px solid var(--border-color);
  }
  .panel-title {
    font-weight: 600;
    font-size: 12px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--text-secondary);
  }
  .device-count {
    background: var(--accent);
    color: white;
    border-radius: 10px;
    padding: 0 6px;
    font-size: 11px;
    min-width: 18px;
    text-align: center;
  }
  .device-list {
    flex: 1;
    overflow-y: auto;
    padding: 4px 0;
  }
  .device-item {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 8px 12px;
    background: transparent;
    border: none;
    color: var(--text-primary);
    cursor: pointer;
    text-align: left;
    font-family: var(--font-ui);
    transition: background 0.1s;
  }
  .device-item:hover { background: var(--bg-hover); }
  .device-item.active { background: var(--bg-tertiary); border-left: 2px solid var(--accent); }
  .status-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    flex-shrink: 0;
  }
  .device-info { display: flex; flex-direction: column; min-width: 0; }
  .device-model {
    font-size: 13px;
    font-weight: 500;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .device-serial {
    font-size: 11px;
    color: var(--text-secondary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .empty-state {
    padding: 20px 12px;
    text-align: center;
    color: var(--text-secondary);
    font-size: 12px;
  }
  .panel-footer {
    padding: 8px 12px;
    border-top: 1px solid var(--border-color);
  }
</style>
