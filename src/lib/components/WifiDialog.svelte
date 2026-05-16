<script lang="ts">
  import Dialog from './ui/Dialog.svelte';
  import Input from './ui/Input.svelte';
  import Button from './ui/Button.svelte';
  import { connectWifi, pairDevice } from '../utils/tauri';
  import { savedWifiIps, addWifiIp, removeWifiIp } from '../stores/settings';
  import { tt } from '../i18n';

  export let open: boolean = false;

  let mode: 'connect' | 'pair' = 'connect';
  let addr: string = '';
  let pairCode: string = '';
  let status: string = '';
  let loading: boolean = false;
  let actionDone: 'connected' | 'paired' | null = null;

  // Reset state when dialog is reopened
  $: if (open) {
    loading = false;
    status = '';
    actionDone = null;
  }

  function extractIp(address: string): string {
    return address.split(':')[0];
  }

  function selectSavedIp(ip: string) {
    addr = ip;
  }

  async function handleConnect() {
    loading = true;
    status = '';
    actionDone = null;
    try {
      const result = await connectWifi(addr);
      status = result;
      if (result.includes('connected')) {
        actionDone = 'connected';
        const ip = extractIp(addr);
        if (ip) addWifiIp(ip);
        setTimeout(() => { open = false; }, 1500);
      }
    } catch (e) {
      status = `Failed: ${e}`;
    }
    loading = false;
  }

  async function handlePair() {
    loading = true;
    status = '';
    actionDone = null;
    try {
      const result = await pairDevice(addr, pairCode);
      status = result;
      if (result.includes('Successfully paired')) {
        actionDone = 'paired';
        const ip = extractIp(addr);
        if (ip) addWifiIp(ip);
      }
    } catch (e) {
      status = `Failed: ${e}`;
    }
    loading = false;
  }
</script>

<Dialog bind:open title={$tt('wifi.title')}>
  <div class="wifi-form">
    <div class="mode-switch">
      <Button size="sm" active={mode === 'connect'} on:click={() => { mode = 'connect'; actionDone = null; status = ''; }}>{$tt('wifi.connect')}</Button>
      <Button size="sm" active={mode === 'pair'} on:click={() => { mode = 'pair'; actionDone = null; status = ''; }}>{$tt('wifi.pair')}</Button>
    </div>

    <div class="mode-desc">
      {#if mode === 'connect'}
        <p>{$tt('wifi.connect_desc')}</p>
      {:else}
        <p>{$tt('wifi.pair_desc')}</p>
      {/if}
    </div>

    <!-- Saved WiFi IPs -->
    {#if $savedWifiIps.length > 0}
      <div class="saved-section">
        <span class="saved-label">{$tt('wifi.saved_ips')}</span>
        <div class="saved-list">
          {#each $savedWifiIps as ip}
            <button class="saved-chip" on:click={() => selectSavedIp(ip)} title={ip}>
              <span class="saved-ip">{ip}</span>
              <span
                class="saved-remove"
                role="button"
                tabindex="0"
                on:click|stopPropagation={() => removeWifiIp(ip)}
                on:keydown|stopPropagation={(e) => { if (e.key === 'Enter') removeWifiIp(ip); }}
              >&times;</span>
            </button>
          {/each}
        </div>
      </div>
    {/if}

    <label class="field">
      <span class="label">{mode === 'pair' ? $tt('wifi.pair_address') : $tt('wifi.device_address')}</span>
      <Input bind:value={addr} placeholder="192.168.1.100:5555" />
    </label>

    {#if mode === 'pair'}
      <label class="field">
        <span class="label">{$tt('wifi.pairing_code')}</span>
        <Input bind:value={pairCode} placeholder="123456" />
      </label>
    {/if}

    {#if status}
      <div class="status" class:error={status.includes('Failed')} class:success={actionDone !== null}>{status}</div>
    {/if}

    <Button
      variant={actionDone ? 'secondary' : 'primary'}
      disabled={loading || !addr || actionDone !== null}
      on:click={mode === 'pair' ? handlePair : handleConnect}
    >
      {#if actionDone === 'connected'}
        &#10003; {$tt('wifi.connected')}
      {:else if actionDone === 'paired'}
        &#10003; {$tt('wifi.paired')}
      {:else if loading}
        {mode === 'pair' ? $tt('wifi.pairing') : $tt('wifi.connecting')}
      {:else}
        {mode === 'pair' ? $tt('wifi.pair') : $tt('wifi.connect')}
      {/if}
    </Button>
  </div>
</Dialog>

<style>
  .wifi-form { display: flex; flex-direction: column; gap: 12px; }
  .mode-switch { display: flex; gap: 4px; }
  .mode-desc {
    font-size: 12px;
    line-height: 1.5;
    color: var(--text-secondary);
    background: var(--bg-tertiary);
    border-radius: var(--radius-sm);
    padding: 8px 10px;
    border-left: 3px solid var(--accent);
  }
  .mode-desc p { margin: 0; }

  /* Saved IPs */
  .saved-section {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .saved-label {
    font-size: 11px;
    color: var(--text-secondary);
    font-weight: 500;
  }
  .saved-list {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
  }
  .saved-chip {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 3px 8px;
    background: var(--bg-tertiary);
    border: 1px solid var(--border-color);
    border-radius: 12px;
    color: var(--text-primary);
    font-size: 12px;
    font-family: var(--font-mono);
    cursor: pointer;
    transition: all 0.15s;
  }
  .saved-chip:hover {
    border-color: var(--accent);
    background: rgba(0, 122, 204, 0.08);
  }
  .saved-remove {
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
  }
  .saved-remove:hover {
    background: rgba(244, 71, 71, 0.2);
    color: var(--error);
  }

  .field { display: flex; flex-direction: column; gap: 4px; }
  .label { font-size: 12px; color: var(--text-secondary); }
  .status {
    font-size: 12px;
    padding: 6px 10px;
    border-radius: var(--radius-sm);
    background: var(--bg-tertiary);
    color: var(--text-secondary);
  }
  .status.success { color: var(--success); }
  .status.error { color: var(--error); }
</style>
