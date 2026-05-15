import { writable, derived, get } from 'svelte/store';
import type { LogEntry, LogLevel, LogSource } from '../types';

const MAX_ENTRIES = 100_000;

export const logEntries = writable<Map<string, LogEntry[]>>(new Map());
export const filterLevel = writable<LogLevel | null>(null);
export const filterTags = writable<string[]>([]);
export const filterSearch = writable<string>('');
export const filterSearchRegex = writable<boolean>(false);
export const filterPid = writable<number | null>(null);
export const unityMode = writable<boolean>(false);
export const autoScroll = writable<boolean>(true);
export const isPaused = writable<boolean>(false);

const UNITY_TAGS = ['Unity', 'Il2Cpp', 'Mono', 'CRASH'];
const LOG_LEVEL_ORDER: LogLevel[] = ['verbose', 'debug', 'info', 'warn', 'error', 'fatal'];

export function addLogEntries(serial: string, entries: LogEntry[]) {
  logEntries.update(map => {
    const existing = map.get(serial) ?? [];
    const combined = existing.concat(entries);
    if (combined.length > MAX_ENTRIES) {
      map.set(serial, combined.slice(combined.length - MAX_ENTRIES));
    } else {
      map.set(serial, combined);
    }
    return new Map(map);
  });
}

export function clearLogEntries(serial: string) {
  logEntries.update(map => {
    map.set(serial, []);
    return new Map(map);
  });
}

export function getFilteredEntries(entries: LogEntry[]): LogEntry[] {
  const level = get(filterLevel);
  const tags = get(filterTags);
  const search = get(filterSearch);
  const isRegex = get(filterSearchRegex);
  const pid = get(filterPid);
  const isUnity = get(unityMode);

  let filtered = entries;

  if (isUnity) {
    filtered = filtered.filter(e => UNITY_TAGS.includes(e.tag));
  }

  if (level) {
    const minIndex = LOG_LEVEL_ORDER.indexOf(level);
    filtered = filtered.filter(e => LOG_LEVEL_ORDER.indexOf(e.level) >= minIndex);
  }

  if (tags.length > 0) {
    filtered = filtered.filter(e => tags.includes(e.tag));
  }

  if (pid !== null) {
    filtered = filtered.filter(e => e.pid === pid);
  }

  if (search) {
    if (isRegex) {
      try {
        const re = new RegExp(search, 'i');
        filtered = filtered.filter(e => re.test(e.message) || re.test(e.tag));
      } catch {
        // invalid regex, skip
      }
    } else {
      const lower = search.toLowerCase();
      filtered = filtered.filter(
        e => e.message.toLowerCase().includes(lower) || e.tag.toLowerCase().includes(lower)
      );
    }
  }

  return filtered;
}
