import type { ReadingProgress } from "@bascan/shared";

const STORAGE_KEY = "bascan_progress";

function getAll(): Record<string, ReadingProgress> {
  try {
    return JSON.parse(localStorage.getItem(STORAGE_KEY) || "{}");
  } catch {
    return {};
  }
}

export function getProgress(volumeId: string): ReadingProgress | null {
  return getAll()[volumeId] ?? null;
}

export function saveProgress(volumeId: string, pageIndex: number): void {
  const all = getAll();
  all[volumeId] = { volumeId, pageIndex, timestamp: Date.now() };
  localStorage.setItem(STORAGE_KEY, JSON.stringify(all));
}
