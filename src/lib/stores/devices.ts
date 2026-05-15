import { writable, derived } from 'svelte/store';
import type { Device } from '../types';

export const devices = writable<Device[]>([]);
export const activeDeviceSerial = writable<string | null>(null);

export const activeDevice = derived(
  [devices, activeDeviceSerial],
  ([$devices, $serial]) => $devices.find(d => d.serial === $serial) ?? null
);

export const onlineDevices = derived(
  devices,
  ($devices) => $devices.filter(d => d.status === 'online')
);
