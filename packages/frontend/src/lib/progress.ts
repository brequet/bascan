import type { ReadingProgress } from "@bascan/shared";

const STORAGE_KEY = "bascan_progress";

function getAll(): Record<string, ReadingProgress> {
  try {
    return JSON.parse(localStorage.getItem(STORAGE_KEY) || "{}");
  } catch {
    return {};
  }
}

function progressKey(seriesId: string, volumeId: string): string {
  return `${seriesId}/${volumeId}`;
}

export function getProgress(seriesId: string, volumeId: string): ReadingProgress | null {
  const all = getAll();
  // Try new key format first, then fall back to old format (volumeId only)
  return all[progressKey(seriesId, volumeId)] ?? all[volumeId] ?? null;
}

export function saveProgress(seriesId: string, volumeId: string, pageIndex: number): void {
  const all = getAll();
  const key = progressKey(seriesId, volumeId);
  all[key] = { volumeId, pageIndex, timestamp: Date.now() };
  localStorage.setItem(STORAGE_KEY, JSON.stringify(all));
}
