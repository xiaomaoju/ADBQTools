<script lang="ts">
  import Dialog from './ui/Dialog.svelte';
  import { language } from '../stores/settings';
  import { tt } from '../i18n';

  export let open: boolean = false;
  export let onTutorialClick: () => void = () => {};
</script>

<Dialog bind:open title={$tt('settings.title')}>
  <div class="settings-form">
    <!-- Language Setting -->
    <div class="setting-row">
      <div class="setting-info">
        <span class="setting-label">{$tt('settings.language')}</span>
      </div>
      <div class="lang-toggle">
        <span class="lang-option" class:active={$language === 'zh'}>中文</span>
        <button
          class="toggle-switch"
          class:en={$language === 'en'}
          on:click={() => $language = $language === 'zh' ? 'en' : 'zh'}
          role="switch"
          aria-checked={$language === 'en'}
          aria-label="Toggle language"
        >
          <span class="toggle-thumb"></span>
        </button>
        <span class="lang-option" class:active={$language === 'en'}>EN</span>
      </div>
    </div>

    <div class="setting-divider"></div>

    <!-- Tutorial -->
    <button class="setting-row clickable" on:click={() => { open = false; onTutorialClick(); }} aria-label="Open tutorial">
      <div class="setting-info">
        <span class="setting-label">{$tt('settings.tutorial')}</span>
      </div>
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <polyline points="9 18 15 12 9 6"></polyline>
      </svg>
    </button>
  </div>
</Dialog>

<style>
  .settings-form {
    display: flex;
    flex-direction: column;
    gap: 0;
  }
  .setting-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 4px;
    background: none;
    border: none;
    color: var(--text-primary);
    font-family: inherit;
    width: 100%;
    text-align: left;
  }
  .setting-row.clickable {
    cursor: pointer;
    border-radius: var(--radius-sm);
    transition: background 0.15s;
  }
  .setting-row.clickable:hover {
    background: var(--bg-hover);
  }
  .setting-row.clickable svg {
    color: var(--text-secondary);
  }
  .setting-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .setting-label {
    font-size: 14px;
    font-weight: 500;
  }
  .setting-divider {
    height: 1px;
    background: var(--border-color);
  }

  /* iOS-style toggle */
  .lang-toggle {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .lang-option {
    font-size: 13px;
    color: var(--text-secondary);
    transition: color 0.2s;
    user-select: none;
  }
  .lang-option.active {
    color: var(--text-bright);
    font-weight: 600;
  }
  .toggle-switch {
    position: relative;
    width: 44px;
    height: 24px;
    background: var(--accent);
    border: none;
    border-radius: 12px;
    cursor: pointer;
    transition: background 0.25s;
    padding: 0;
    flex-shrink: 0;
  }
  .toggle-switch.en {
    background: var(--accent);
  }
  .toggle-thumb {
    position: absolute;
    top: 2px;
    left: 2px;
    width: 20px;
    height: 20px;
    background: white;
    border-radius: 50%;
    transition: transform 0.25s cubic-bezier(0.4, 0, 0.2, 1);
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.3);
  }
  .toggle-switch.en .toggle-thumb {
    transform: translateX(20px);
  }
</style>
