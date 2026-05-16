<script lang="ts">
  import './styles/global.css';
  import { onMount, onDestroy } from 'svelte';
  import DevicePanel from './lib/components/DevicePanel.svelte';
  import WifiDialog from './lib/components/WifiDialog.svelte';
  import SettingsDialog from './lib/components/SettingsDialog.svelte';
  import TutorialDialog from './lib/components/TutorialDialog.svelte';
  import Toolbar from './lib/components/Toolbar.svelte';
  import LogcatViewer from './lib/components/LogcatViewer.svelte';
  import Installer from './lib/components/Installer.svelte';
  import StatusBar from './lib/components/StatusBar.svelte';
  import { devices, activeDeviceSerial } from './lib/stores/devices';
  import { onDevicesChanged, getDevices } from './lib/utils/tauri';
  import type { ViewMode } from './lib/types';
  import type { UnlistenFn } from '@tauri-apps/api/event';

  let currentView: ViewMode = 'logcat';
  let showWifiDialog = false;
  let showSettingsDialog = false;
  let showTutorialDialog = false;
  let unlisten: UnlistenFn | null = null;

  onMount(async () => {
    try {
      const initial = await getDevices();
      $devices = initial;
      if (initial.length > 0 && !$activeDeviceSerial) {
        $activeDeviceSerial = initial[0].serial;
      }
    } catch (e) {
      console.error('Failed to get initial devices:', e);
    }

    unlisten = await onDevicesChanged((newDevices) => {
      $devices = newDevices;
      if (newDevices.length > 0 && !$activeDeviceSerial) {
        $activeDeviceSerial = newDevices[0].serial;
      }
      if ($activeDeviceSerial && !newDevices.find(d => d.serial === $activeDeviceSerial)) {
        $activeDeviceSerial = newDevices[0]?.serial ?? null;
      }
    });
  });

  onDestroy(() => {
    unlisten?.();
  });
</script>

<main class="app-shell">
  <div class="topbar">
    <DevicePanel onWifiClick={() => showWifiDialog = true} />
    <Toolbar bind:currentView onSettingsClick={() => showSettingsDialog = true} />
  </div>
  <div class="main-area">
    {#if currentView === 'logcat'}
      <LogcatViewer />
    {:else}
      <Installer />
    {/if}
  </div>
  <div class="statusbar-area">
    <StatusBar />
  </div>
</main>

<WifiDialog bind:open={showWifiDialog} />
<SettingsDialog bind:open={showSettingsDialog} onTutorialClick={() => showTutorialDialog = true} />
<TutorialDialog bind:open={showTutorialDialog} />

<style>
  .app-shell {
    display: flex;
    flex-direction: column;
    height: 100vh;
    width: 100vw;
  }
  .topbar {
    display: flex;
    align-items: center;
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border-color);
    min-height: 0;
  }
  .main-area {
    flex: 1;
    overflow: hidden;
    position: relative;
    min-height: 0;
  }
  .statusbar-area {
    height: var(--statusbar-height);
    background: var(--accent);
    color: white;
    display: flex;
    align-items: center;
    padding: 0 12px;
    flex-shrink: 0;
  }
</style>
