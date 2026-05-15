<script lang="ts">
  import type { StackFrame } from '../types';

  export let frames: StackFrame[];
  let collapsed: boolean = true;
</script>

<div class="stack-group">
  <button class="stack-toggle" on:click={() => collapsed = !collapsed}>
    <span class="toggle-icon">{collapsed ? '▶' : '▼'}</span>
    <span class="stack-label">Stack Trace</span>
    <span class="frame-count">{frames.length} frames</span>
  </button>
  {#if !collapsed}
    <div class="stack-frames">
      {#each frames as frame, i}
        <div class="frame">
          <span class="frame-num">{i}</span>
          <span class="frame-at">at </span>
          {#if frame.module}
            <span class="frame-module">{frame.module}.</span>
          {/if}
          <span class="frame-class">{frame.class_name}</span>
          <span class="frame-dot">.</span>
          <span class="frame-method">{frame.method_name}</span>
          <span class="frame-parens">()</span>
          {#if frame.file}
            <span class="frame-file"> in {frame.file}:{frame.line ?? '?'}</span>
          {/if}
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .stack-group {
    padding: 2px 10px 2px 10px;
    background: rgba(244, 71, 71, 0.04);
    border-left: 2px solid var(--log-error);
    margin-left: 175px; /* align with tag column */
  }
  .stack-toggle {
    display: flex;
    align-items: center;
    gap: 6px;
    background: none;
    border: none;
    color: var(--log-error);
    font-family: var(--font-mono);
    font-size: 12px;
    cursor: pointer;
    padding: 2px 0;
    line-height: 18px;
  }
  .stack-toggle:hover { opacity: 0.8; }
  .toggle-icon { font-size: 9px; width: 10px; }
  .stack-label { font-weight: 600; }
  .frame-count {
    color: var(--text-secondary);
    font-size: 11px;
  }
  .stack-frames { padding-left: 16px; }
  .frame {
    font-family: var(--font-mono);
    font-size: 12px;
    color: var(--text-secondary);
    line-height: 20px;
    white-space: nowrap;
  }
  .frame-num {
    display: inline-block;
    width: 20px;
    color: var(--text-secondary);
    opacity: 0.4;
    text-align: right;
    margin-right: 8px;
    font-size: 10px;
  }
  .frame-at { color: var(--text-secondary); }
  .frame-module { color: #888; }
  .frame-class { color: #4fc1ff; font-weight: 600; }
  .frame-dot { color: var(--text-secondary); }
  .frame-method { color: #cca700; font-weight: 600; }
  .frame-parens { color: var(--text-secondary); }
  .frame-file { color: #4ec9b0; }
</style>
