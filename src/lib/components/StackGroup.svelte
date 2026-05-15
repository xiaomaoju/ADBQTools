<script lang="ts">
  import type { StackFrame } from '../types';

  export let frames: StackFrame[];
  let collapsed: boolean = true;
</script>

<div class="stack-group">
  <button class="stack-toggle" on:click={() => collapsed = !collapsed}>
    {collapsed ? '▶' : '▼'} Stack Trace ({frames.length} frames)
  </button>
  {#if !collapsed}
    <div class="stack-frames">
      {#each frames as frame}
        <div class="frame">
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
  .stack-group { padding: 2px 12px 2px 24px; }
  .stack-toggle {
    background: none;
    border: none;
    color: var(--log-error);
    font-family: var(--font-mono);
    font-size: 12px;
    cursor: pointer;
    padding: 2px 0;
  }
  .stack-toggle:hover { text-decoration: underline; }
  .stack-frames { padding-left: 16px; }
  .frame {
    font-family: var(--font-mono);
    font-size: 12px;
    color: var(--text-secondary);
    line-height: 1.6;
  }
  .frame-at { color: var(--text-secondary); }
  .frame-module { color: var(--log-verbose); }
  .frame-class { color: var(--log-info); font-weight: 600; }
  .frame-dot { color: var(--text-secondary); }
  .frame-method { color: var(--log-warn); font-weight: 600; }
  .frame-parens { color: var(--text-secondary); }
  .frame-file { color: var(--log-debug); }
</style>
