export type TransportType = 'usb' | 'wifi';
export type DeviceStatus = 'online' | 'offline' | 'unauthorized';
export type LogLevel = 'verbose' | 'debug' | 'info' | 'warn' | 'error' | 'fatal';
export type LogSource = 'system' | 'unity' | 'il2cpp' | 'mono';

export interface Device {
  serial: string;
  model: string;
  product: string;
  transport: TransportType;
  status: DeviceStatus;
}

export interface StackFrame {
  module: string;
  class_name: string;
  method_name: string;
  file: string | null;
  line: number | null;
}

export interface ScriptInfo {
  file: string;
  line: number;
}

export interface LogEntry {
  id: number;
  timestamp: string;
  pid: number;
  tid: number;
  level: LogLevel;
  tag: string;
  message: string;
  package_name: string;
  source: LogSource;
  stack_frames: StackFrame[] | null;
  unity_script_info: ScriptInfo | null;
}

export interface InstallProgress {
  stage: 'parsing' | 'building' | 'installing' | 'complete' | 'failed';
  message: string;
}

export interface KeystoreConfig {
  path: string;
  alias: string;
  store_password: string;
  key_password: string;
}

export interface InstallRecord {
  id: string;
  filename: string;
  device_serial: string;
  device_model: string;
  timestamp: number;
  result: 'success' | 'failed';
  error?: string;
}

export type ViewMode = 'logcat' | 'installer';

export interface FilterPreset {
  name: string;
  tags: string[];
  levels: LogLevel[];
}

export const UNITY_FILTER_PRESETS: FilterPreset[] = [
  { name: 'Unity All', tags: ['Unity', 'Il2Cpp', 'Mono', 'CRASH'], levels: [] },
  { name: 'Unity Errors', tags: ['Unity', 'Il2Cpp', 'Mono', 'CRASH'], levels: ['error', 'fatal'] },
  { name: 'Unity Scripting', tags: ['Unity'], levels: [] },
  { name: 'Unity Rendering', tags: ['Unity'], levels: [] },
  { name: 'Unity Network', tags: ['Unity'], levels: [] },
];
