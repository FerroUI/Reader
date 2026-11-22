const VERSION = "__BUILD_VERSION__";
const ASSETS = "__PRECACHE_FILES__";
const PREFIX = "ferroui-reader-";
const CACHE = PREFIX + VERSION;
const root = new URL("./", self.location.href);

self.addEventListener("install", event => {
  event.waitUntil((async () => {
    const cache = await caches.open(CACHE);
    // Activate only a complete version; never serve a half-cached WASM application.
    try { await cache.addAll(ASSETS.map(path => new URL(path, root).href)); }
    catch (error) { await caches.delete(CACHE); throw error; }
  })());
});
self.addEventListener("activate", event => {
  event.waitUntil((async () => {
    for (const key of await caches.keys()) if (key.startsWith(PREFIX) && key !== CACHE) await caches.delete(key);
    await self.clients.claim();
  })());
});
self.addEventListener("fetch", event => {
  const request = event.request, url = new URL(request.url);
  if (request.method !== "GET" || url.origin !== root.origin || !url.pathname.startsWith(root.pathname)) return;
  if (url.pathname.endsWith("app-sw.js") || url.searchParams.has("nooffline")) return;
  event.respondWith((async () => {
    const cache = await caches.open(CACHE);
    if (request.mode === "navigate") {
      // An active worker serves its matching index and asset snapshot as one version.
      const index = await cache.match(new URL("index.html", root).href);
      if (index) return index;
    }
    const cached = await cache.match(request, { ignoreSearch: true });
    return cached || fetch(request);
  })());
});
