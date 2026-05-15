<script lang="ts">
  import './styles/global.css';
</script>

<main class="app-shell">
  <div class="sidebar">
    <p style="padding: 12px; color: var(--text-secondary);">Devices</p>
  </div>
  <div class="content">
    <div class="toolbar">
      <p style="padding: 8px;">AndroidQTools</p>
    </div>
    <div class="main-area">
      <p style="padding: 20px; color: var(--text-secondary);">Main content area</p>
    </div>
    <div class="statusbar">
      <span>Ready</span>
    </div>
  </div>
</main>

<style>
  .app-shell {
    display: flex;
    height: 100vh;
    width: 100vw;
  }
  .sidebar {
    width: var(--sidebar-width);
    min-width: var(--sidebar-width);
    background: var(--bg-secondary);
    border-right: 1px solid var(--border-color);
    display: flex;
    flex-direction: column;
  }
  .content {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
  }
  .toolbar {
    height: var(--toolbar-height);
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border-color);
    display: flex;
    align-items: center;
  }
  .main-area {
    flex: 1;
    overflow: hidden;
  }
  .statusbar {
    height: var(--statusbar-height);
    background: var(--accent);
    color: white;
    display: flex;
    align-items: center;
    padding: 0 12px;
    font-size: 12px;
  }
</style>
