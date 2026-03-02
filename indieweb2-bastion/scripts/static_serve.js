// Deno static server for PWA/ReScript JS
// Usage: deno run --allow-read --allow-net scripts/static_serve.js <dir> <port>
const [dir, portArg] = Deno.args;
if (!dir || !portArg) {
  console.error("Usage: static_serve.js <dir> <port>");
  Deno.exit(2);
}
const port = Number(portArg);
const handler = async (req) => {
  const url = new URL(req.url);
  let path = url.pathname === "/" ? "/index.html" : url.pathname;
  try {
    const file = await Deno.readFile(`${dir}${path}`);
    const ext = path.split(".").pop();
    const contentType = {
      html: "text/html",
      js: "application/javascript",
      css: "text/css",
      json: "application/json",
      png: "image/png",
      jpg: "image/jpeg",
      svg: "image/svg+xml",
    }[ext] || "application/octet-stream";
    return new Response(file, { headers: { "content-type": contentType } });
  } catch {
    return new Response("Not found", { status: 404 });
  }
};
console.log(`Serving ${dir} on http://localhost:${port}`);
Deno.serve({ port }, handler);
