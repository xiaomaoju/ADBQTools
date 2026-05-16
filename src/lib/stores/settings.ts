import { writable } from 'svelte/store';
import type { Lang } from '../i18n';

// Persist language to localStorage
const storedLang = (typeof localStorage !== 'undefined' && localStorage.getItem('aqt-language')) as Lang | null;
export const language = writable<Lang>(storedLang || 'en');
language.subscribe(val => {
  if (typeof localStorage !== 'undefined') {
    localStorage.setItem('aqt-language', val);
  }
});

// Saved WiFi IPs (just the IP, no port)
const storedIps = typeof localStorage !== 'undefined' ? JSON.parse(localStorage.getItem('aqt-wifi-ips') || '[]') as string[] : [];
export const savedWifiIps = writable<string[]>(storedIps);
savedWifiIps.subscribe(val => {
  if (typeof localStorage !== 'undefined') {
    localStorage.setItem('aqt-wifi-ips', JSON.stringify(val));
  }
});

export function addWifiIp(ip: string) {
  savedWifiIps.update(list => {
    if (list.includes(ip)) return list;
    return [ip, ...list].slice(0, 20);
  });
}

export function removeWifiIp(ip: string) {
  savedWifiIps.update(list => list.filter(i => i !== ip));
}
