<script lang="ts">
  import { onlineDevices } from '../stores/devices';
</script>

<div class="statusbar">
  <span>{$onlineDevices.length} device{$onlineDevices.length !== 1 ? 's' : ''} connected</span>
  <span class="separator">|</span>
  <span>ADB Ready</span>
</div>

<style>
  .statusbar {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 12px;
  }
  .separator { color: rgba(255, 255, 255, 0.3); }
</style>
