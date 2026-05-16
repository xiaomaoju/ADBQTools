<script lang="ts">
  import Dialog from './ui/Dialog.svelte';
  import { tt } from '../i18n';

  export let open: boolean = false;

  interface Section {
    titleKey: string;
    contentKey: string;
    icon: string;
  }

  const sections: Section[] = [
    { titleKey: 'tutorial.device_title', contentKey: 'tutorial.device_content', icon: '📱' },
    { titleKey: 'tutorial.logcat_title', contentKey: 'tutorial.logcat_content', icon: '📋' },
    { titleKey: 'tutorial.installer_title', contentKey: 'tutorial.installer_content', icon: '📦' },
    { titleKey: 'tutorial.wifi_title', contentKey: 'tutorial.wifi_content', icon: '📶' },
  ];

  let expandedIndex: number | null = null;

  function toggle(i: number) {
    expandedIndex = expandedIndex === i ? null : i;
  }

  $: if (!open) expandedIndex = null;
</script>

<Dialog bind:open title={$tt('tutorial.title')} wide={true}>
  <div class="tutorial">
    <p class="overview">{$tt('tutorial.overview')}</p>

    <div class="accordion">
      {#each sections as section, i}
        <div class="acc-item" class:expanded={expandedIndex === i}>
          <button class="acc-header" on:click={() => toggle(i)}>
            <span class="acc-icon">{section.icon}</span>
            <span class="acc-title">{$tt(section.titleKey)}</span>
            <span class="acc-arrow">{expandedIndex === i ? '▾' : '▸'}</span>
          </button>
          {#if expandedIndex === i}
            <div class="acc-body">
              {#each $tt(section.contentKey).split('\n') as line}
                {#if line.trim()}
                  <p
                    class="content-line"
                    class:bullet={line.trim().startsWith('•')}
                    class:numbered={/^\d+\./.test(line.trim())}
                    class:sub-label={!line.trim().startsWith('•') && !/^\d+\./.test(line.trim()) && line.trim().endsWith('：')}
                  >{line}</p>
                {:else}
                  <div class="content-gap"></div>
                {/if}
              {/each}
            </div>
          {/if}
        </div>
      {/each}
    </div>
  </div>
</Dialog>

<style>
  .tutorial {
    display: flex;
    flex-direction: column;
    gap: 12px;
    max-height: 65vh;
    overflow-y: auto;
    padding-right: 4px;
  }
  .overview {
    margin: 0;
    font-size: 13px;
    line-height: 1.7;
    color: var(--text-primary);
    padding: 10px 12px;
    background: var(--bg-tertiary);
    border-radius: var(--radius-md);
    border-left: 3px solid var(--accent);
  }
  .accordion {
    display: flex;
    flex-direction: column;
    border: 1px solid var(--border-color);
    border-radius: var(--radius-md);
    overflow: hidden;
  }
  .acc-item + .acc-item {
    border-top: 1px solid var(--border-color);
  }
  .acc-header {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 10px 14px;
    background: var(--bg-secondary);
    border: none;
    cursor: pointer;
    text-align: left;
    color: var(--text-bright);
    font-size: 13px;
    font-weight: 600;
    font-family: inherit;
    transition: background 0.15s;
  }
  .acc-header:hover {
    background: var(--bg-hover);
  }
  .acc-icon {
    font-size: 15px;
    line-height: 1;
    flex-shrink: 0;
  }
  .acc-title {
    flex: 1;
  }
  .acc-arrow {
    font-size: 11px;
    color: var(--text-secondary);
    flex-shrink: 0;
  }
  .acc-body {
    padding: 10px 14px 12px 38px;
    background: var(--bg-tertiary);
  }
  .content-line {
    margin: 0;
    font-size: 12.5px;
    line-height: 1.7;
    color: var(--text-primary);
  }
  .content-line.bullet {
    padding-left: 2px;
  }
  .content-line.numbered {
    padding-left: 8px;
  }
  .content-line.sub-label {
    font-weight: 600;
    color: var(--text-bright);
    margin-top: 6px;
  }
  .content-gap {
    height: 6px;
  }

  .tutorial::-webkit-scrollbar { width: 6px; }
  .tutorial::-webkit-scrollbar-track { background: transparent; }
  .tutorial::-webkit-scrollbar-thumb { background: var(--border-color); border-radius: 3px; }
  .tutorial::-webkit-scrollbar-thumb:hover { background: var(--text-secondary); }
</style>
