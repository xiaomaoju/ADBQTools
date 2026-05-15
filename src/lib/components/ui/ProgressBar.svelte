<script lang="ts">
  export let progress: number = 0;
  export let indeterminate: boolean = false;
</script>

<div class="progress-track">
  {#if indeterminate}
    <div class="progress-bar indeterminate" />
  {:else}
    <div class="progress-bar" style="width: {Math.min(100, Math.max(0, progress))}%" />
  {/if}
</div>

<style>
  .progress-track {
    height: 4px;
    background: var(--bg-hover);
    border-radius: 2px;
    overflow: hidden;
  }
  .progress-bar {
    height: 100%;
    background: var(--accent);
    border-radius: 2px;
    transition: width 0.3s ease;
  }
  .indeterminate {
    width: 40%;
    animation: slide 1.5s ease-in-out infinite;
  }
  @keyframes slide {
    0% { transform: translateX(-100%); }
    100% { transform: translateX(350%); }
  }
</style>
