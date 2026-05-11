import { Hono } from "hono";
import { cors } from "hono/cors";
import { seriesRouter } from "./routes/series";
import { LIBRARY_ROOT } from "./config";

const app = new Hono();

app.use("/*", cors({ origin: "http://localhost:5173" }));

app.route("/api/series", seriesRouter);

// Serve scan images: /images/:series/:volume/:file
app.get("/images/:series/:volume/:file", async (c) => {
  const { series, volume, file } = c.req.param();
  const path = `${LIBRARY_ROOT}/${decodeURIComponent(series)}/${decodeURIComponent(volume)}/${decodeURIComponent(file)}`;

  const bunFile = Bun.file(path);
  if (!(await bunFile.exists())) {
    return c.text("Not found", 404);
  }

  return new Response(bunFile, {
    headers: { "Content-Type": "image/jpeg", "Cache-Control": "public, max-age=31536000" },
  });
});

const PORT = 3001;
console.log(`Bascan backend running on http://localhost:${PORT}`);
console.log(`Library: ${LIBRARY_ROOT}`);

export default {
  port: PORT,
  fetch: app.fetch,
};
