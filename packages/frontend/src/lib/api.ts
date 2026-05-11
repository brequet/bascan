export async function fetchVolumes() {
  const res = await fetch(`/api/volumes`);
  return res.json();
}

export async function fetchPages(volumeId: string) {
  const res = await fetch(`/api/volumes/${encodeURIComponent(volumeId)}/pages`);
  return res.json();
}

export function imageUrl(path: string): string {
  return path;
}
