import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type { Device, LogEntry, InstallProgress, KeystoreConfig } from '../types';

export async function getDevices(): Promise<Device[]> {
  return invoke('get_devices');
}

export async function connectWifi(addr: string): Promise<string> {
  return invoke('connect_wifi', { addr });
}

export async function pairDevice(addr: string, code: string): Promise<string> {
  return invoke('pair_device', { addr, code });
}

export async function disconnectDevice(serial: string): Promise<string> {
  return invoke('disconnect_device', { serial });
}

export async function startLogcat(serial: string): Promise<void> {
  return invoke('start_logcat', { serial });
}

export async function stopLogcat(serial: string): Promise<void> {
  return invoke('stop_logcat', { serial });
}

export async function pauseLogcat(serial: string): Promise<void> {
  return invoke('pause_logcat', { serial });
}

export async function resumeLogcat(serial: string): Promise<void> {
  return invoke('resume_logcat', { serial });
}

export async function clearLogcat(serial: string): Promise<void> {
  return invoke('clear_logcat', { serial });
}

export async function queryLogHistory(
  device: string,
  from?: string,
  to?: string,
  limit?: number
): Promise<LogEntry[]> {
  return invoke('query_log_history', { device, from, to, limit });
}

export async function restartAdb(): Promise<string> {
  return invoke('restart_adb');
}

export async function listKeystoreAliases(keystorePath: string, storePassword: string): Promise<string[]> {
  return invoke('list_keystore_aliases', { keystorePath, storePassword });
}

export async function installApk(serial: string, path: string): Promise<string> {
  return invoke('install_apk', { serial, path });
}

export async function installAab(
  serial: string,
  path: string,
  keystore?: KeystoreConfig
): Promise<string> {
  return invoke('install_aab', { serial, path, keystore });
}

export function onDevicesChanged(callback: (devices: Device[]) => void): Promise<UnlistenFn> {
  return listen<Device[]>('devices-changed', (event) => callback(event.payload));
}

export interface LogcatPayload {
  serial: string;
  entries: LogEntry[];
}

export function onLogcat(callback: (serial: string, entries: LogEntry[]) => void): Promise<UnlistenFn> {
  return listen<LogcatPayload>('logcat-data', (event) => callback(event.payload.serial, event.payload.entries));
}

export function onInstallProgress(callback: (progress: InstallProgress) => void): Promise<UnlistenFn> {
  return listen<InstallProgress>('install-progress', (event) => callback(event.payload));
}

export function onLogcatError(callback: (message: string) => void): Promise<UnlistenFn> {
  return listen<string>('logcat-error', (event) => callback(event.payload));
}
