import { writable } from 'svelte/store';
import type { InstallProgress, InstallRecord, KeystoreConfig } from '../types';
import { generateId } from '../utils/format';

export const installProgress = writable<InstallProgress | null>(null);
export const installHistory = writable<InstallRecord[]>([]);
export const savedKeystore = writable<KeystoreConfig | null>(null);

export function addInstallRecord(
  filename: string,
  deviceSerial: string,
  deviceModel: string,
  result: 'success' | 'failed',
  error?: string
) {
  installHistory.update(list => [{
    id: generateId(),
    filename,
    device_serial: deviceSerial,
    device_model: deviceModel,
    timestamp: Date.now(),
    result,
    error,
  }, ...list].slice(0, 100));
}
