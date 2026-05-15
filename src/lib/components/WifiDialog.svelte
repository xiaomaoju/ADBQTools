<script lang="ts">
  import Dialog from './ui/Dialog.svelte';
  import Input from './ui/Input.svelte';
  import Button from './ui/Button.svelte';
  import { connectWifi, pairDevice } from '../utils/tauri';

  export let open: boolean = false;

  let mode: 'connect' | 'pair' = 'connect';
  let addr: string = '';
  let pairCode: string = '';
  let status: string = '';
  let loading: boolean = false;

  async function handleConnect() {
    loading = true;
    status = '';
    try {
      const result = await connectWifi(addr);
      status = result;
      if (result.includes('connected')) {
        setTimeout(() => { open = false; }, 1000);
      }
    } catch (e) {
      status = `Failed: ${e}`;
    }
    loading = false;
  }

  async function handlePair() {
    loading = true;
    status = '';
    try {
      const result = await pairDevice(addr, pairCode);
      status = result;
    } catch (e) {
      status = `Failed: ${e}`;
    }
    loading = false;
  }
</script>

<Dialog bind:open title="WiFi Connection">
  <div class="wifi-form">
    <div class="mode-switch">
      <Button size="sm" active={mode === 'connect'} on:click={() => mode = 'connect'}>Connect</Button>
      <Button size="sm" active={mode === 'pair'} on:click={() => mode = 'pair'}>Pair</Button>
    </div>

    <label class="field">
      <span class="label">{mode === 'pair' ? 'Pair Address' : 'Device Address'}</span>
      <Input bind:value={addr} placeholder="192.168.1.100:5555" />
    </label>

    {#if mode === 'pair'}
      <label class="field">
        <span class="label">Pairing Code</span>
        <Input bind:value={pairCode} placeholder="123456" />
      </label>
    {/if}

    {#if status}
      <div class="status" class:error={status.includes('Failed')}>{status}</div>
    {/if}

    <Button
      variant="primary"
      disabled={loading || !addr}
      on:click={mode === 'pair' ? handlePair : handleConnect}
    >
      {loading ? 'Connecting...' : mode === 'pair' ? 'Pair' : 'Connect'}
    </Button>
  </div>
</Dialog>

<style>
  .wifi-form { display: flex; flex-direction: column; gap: 12px; }
  .mode-switch { display: flex; gap: 4px; }
  .field { display: flex; flex-direction: column; gap: 4px; }
  .label { font-size: 12px; color: var(--text-secondary); }
  .status {
    font-size: 12px;
    padding: 6px 10px;
    border-radius: var(--radius-sm);
    background: var(--bg-tertiary);
    color: var(--success);
  }
  .status.error { color: var(--error); }
</style>
