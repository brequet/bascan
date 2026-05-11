import { Hono } from "hono";
import { readdir } from "fs/promises";
import { LIBRARY_ROOT } from "../config";
import type { Series, Volume, Page } from "@bascan/shared";

export const seriesRouter = new Hono();

function naturalSort(a: string, b: string): number {
  return a.localeCompare(b, undefined, { numeric: true, sensitivity: "base" });
}

function isImageFile(name: string): boolean {
  return /\.(jpg|jpeg|png|webp)$/i.test(name);
}

function isSpread(filename: string): boolean {
  return /\d+-\d+\.\w+$/.test(filename);
}

// GET /api/series
seriesRouter.get("/", async (c) => {
  const entries = await readdir(LIBRARY_ROOT, { withFileTypes: true });
  const series: Series[] = [];

  for (const entry of entries) {
    if (!entry.isDirectory()) continue;

    const seriesPath = `${LIBRARY_ROOT}/${entry.name}`;
    const volEntries = await readdir(seriesPath, { withFileTypes: true });
    const volDirs = volEntries.filter((e) => e.isDirectory()).map((e) => e.name).sort(naturalSort);

    let coverUrl = "";
    if (volDirs.length > 0) {
      const firstVolPath = `${seriesPath}/${volDirs[0]}`;
      const files = (await readdir(firstVolPath)).filter(isImageFile).sort(naturalSort);
      if (files.length > 0) {
        coverUrl = `/images/${encodeURIComponent(entry.name)}/${encodeURIComponent(volDirs[0])}/${encodeURIComponent(files[0])}`;
      }
    }

    series.push({
      id: entry.name,
      title: entry.name,
      coverUrl,
      volumeCount: volDirs.length,
    });
  }

  series.sort((a, b) => naturalSort(a.id, b.id));
  return c.json(series);
});

// GET /api/series/:seriesId/volumes
seriesRouter.get("/:seriesId/volumes", async (c) => {
  const seriesId = decodeURIComponent(c.req.param("seriesId"));
  const seriesPath = `${LIBRARY_ROOT}/${seriesId}`;

  const entries = await readdir(seriesPath, { withFileTypes: true });
  const volumes: Volume[] = [];

  for (const entry of entries) {
    if (!entry.isDirectory()) continue;

    const folderPath = `${seriesPath}/${entry.name}`;
    const files = (await readdir(folderPath)).filter(isImageFile).sort(naturalSort);

    volumes.push({
      id: entry.name,
      title: entry.name,
      coverUrl: `/images/${encodeURIComponent(seriesId)}/${encodeURIComponent(entry.name)}/${encodeURIComponent(files[0] || "")}`,
      pageCount: files.length,
    });
  }

  volumes.sort((a, b) => naturalSort(a.id, b.id));
  return c.json(volumes);
});

// GET /api/series/:seriesId/volumes/:volumeId/pages
seriesRouter.get("/:seriesId/volumes/:volumeId/pages", async (c) => {
  const seriesId = decodeURIComponent(c.req.param("seriesId"));
  const volumeId = decodeURIComponent(c.req.param("volumeId"));
  const folderPath = `${LIBRARY_ROOT}/${seriesId}/${volumeId}`;

  try {
    const files = (await readdir(folderPath)).filter(isImageFile).sort(naturalSort);
    const pages: Page[] = files.map((filename) => ({
      filename,
      url: `/images/${encodeURIComponent(seriesId)}/${encodeURIComponent(volumeId)}/${encodeURIComponent(filename)}`,
      isSpread: isSpread(filename),
    }));
    return c.json(pages);
  } catch {
    return c.text("Not found", 404);
  }
});
