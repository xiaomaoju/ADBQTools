import { writable } from 'svelte/store';

export interface ColumnWidths {
  timestamp: number;
  pidtid: number;
  package_name: number;
  tag: number;
  level: number;
}

const DEFAULT_WIDTHS: ColumnWidths = {
  timestamp: 175,
  pidtid: 90,
  package_name: 200,
  tag: 160,
  level: 30,
};

const STORAGE_KEY = 'logcat-column-widths';

function loadWidths(): ColumnWidths {
  try {
    const saved = localStorage.getItem(STORAGE_KEY);
    if (saved) return { ...DEFAULT_WIDTHS, ...JSON.parse(saved) };
  } catch (_) {}
  return { ...DEFAULT_WIDTHS };
}

function createColumnWidths() {
  const { subscribe, set, update } = writable<ColumnWidths>(loadWidths());

  subscribe((v) => {
    try { localStorage.setItem(STORAGE_KEY, JSON.stringify(v)); } catch (_) {}
  });

  return {
    subscribe,
    set,
    update,
    resize(col: keyof ColumnWidths, width: number) {
      update(w => ({ ...w, [col]: Math.max(30, width) }));
    },
    reset() {
      set({ ...DEFAULT_WIDTHS });
    },
  };
}

export const columnWidths = createColumnWidths();
