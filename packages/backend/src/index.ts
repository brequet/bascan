import { Hono } from "hono";
import { cors } from "hono/cors";
import { serveStatic } from "hono/bun";
import { volumesRouter } from "./routes/volumes";
import { SCANS_ROOT } from "./config";

const app = new Hono();

app.use("/*", cors({ origin: "http://localhost:5173" }));

// API routes
app.route("/api/volumes", volumesRouter);

// Serve scan images directly
app.get("/images/:volume/:file", async (c) => {
  const { volume, file } = c.req.param();
  const path = `${SCANS_ROOT}/${decodeURIComponent(volume)}/${decodeURIComponent(file)}`;

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
console.log(`Serving scans from: ${SCANS_ROOT}`);

export default {
  port: PORT,
  fetch: app.fetch,
};
