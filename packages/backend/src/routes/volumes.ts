import { Hono } from "hono";
import { readdir } from "fs/promises";
import { SCANS_ROOT } from "../config";
import type { Volume, Page } from "@bascan/shared";

export const volumesRouter = new Hono();

/** Natural sort for filenames like 001.jpg, 006-007.jpg */
function naturalSort(a: string, b: string): number {
  return a.localeCompare(b, undefined, { numeric: true, sensitivity: "base" });
}

function isImageFile(name: string): boolean {
  return /\.(jpg|jpeg|png|webp)$/i.test(name);
}

function isSpread(filename: string): boolean {
  return /\d+-\d+\.\w+$/.test(filename);
}

// GET /api/volumes — list all volumes
volumesRouter.get("/", async (c) => {
  const entries = await readdir(SCANS_ROOT, { withFileTypes: true });
  const volumes: Volume[] = [];

  for (const entry of entries) {
    if (!entry.isDirectory()) continue;
    // Skip artbooks or non-volume folders
    if (!entry.name.startsWith("Berserk T")) continue;

    const folderPath = `${SCANS_ROOT}/${entry.name}`;
    const files = (await readdir(folderPath)).filter(isImageFile).sort(naturalSort);

    volumes.push({
      id: entry.name,
      title: entry.name,
      coverUrl: `/images/${encodeURIComponent(entry.name)}/${encodeURIComponent(files[0] || "")}`,
      pageCount: files.length,
    });
  }

  volumes.sort((a, b) => naturalSort(a.id, b.id));
  return c.json(volumes);
});

// GET /api/volumes/:id/pages — list pages in a volume
volumesRouter.get("/:id/pages", async (c) => {
  const volumeId = decodeURIComponent(c.req.param("id"));
  const folderPath = `${SCANS_ROOT}/${volumeId}`;

  try {
    const files = (await readdir(folderPath)).filter(isImageFile).sort(naturalSort);
    const pages: Page[] = files.map((filename) => ({
      filename,
      url: `/images/${encodeURIComponent(volumeId)}/${encodeURIComponent(filename)}`,
      isSpread: isSpread(filename),
    }));
    return c.json(pages);
  } catch {
    return c.text("Volume not found", 404);
  }
});
