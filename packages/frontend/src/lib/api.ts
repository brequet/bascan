import type { Series, Volume, Page } from "@bascan/shared";

export async function fetchSeries(): Promise<Series[]> {
  const res = await fetch(`/api/series`);
  return res.json();
}

export async function fetchVolumes(seriesId: string): Promise<Volume[]> {
  const res = await fetch(`/api/series/${encodeURIComponent(seriesId)}/volumes`);
  return res.json();
}

export async function fetchPages(seriesId: string, volumeId: string): Promise<Page[]> {
  const res = await fetch(`/api/series/${encodeURIComponent(seriesId)}/volumes/${encodeURIComponent(volumeId)}/pages`);
  return res.json();
}

export function imageUrl(path: string): string {
  return path;
}
