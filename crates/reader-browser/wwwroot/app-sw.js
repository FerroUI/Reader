const VERSION = "__BUILD_VERSION__";
const ASSETS = "__PRECACHE_FILES__";
const PREFIX = "ferroui-reader-";
const CACHE = PREFIX + VERSION;
const root = new URL("./", self.location.href);
const DOWNLOADS = 2;

async function verifiedAsset(asset, signal) {
  const canonical = new URL(asset.path, root);
  const download = new URL(canonical);
  download.searchParams.set("__reader_build", VERSION);
  const response = await fetch(download.href, { cache: "reload", credentials: "same-origin", signal });
  if (!response.ok || response.status === 206) throw new Error(`Offline asset ${asset.path}: HTTP ${response.status}.`);
  // The trusted build manifest bounds each allocation, including an unexpected oversized response.
  const bytes = new Uint8Array(asset.size);
  const reader = response.body?.getReader();
  let length = 0, complete = false;
  try {
    if (reader) {
      for (;;) {
        const { value, done } = await reader.read();
        if (done) break;
        if (value.byteLength > bytes.byteLength - length) throw new Error(`Offline asset ${asset.path}: size mismatch.`);
        bytes.set(value, length); length += value.byteLength;
      }
    }
    if (length !== asset.size) throw new Error(`Offline asset ${asset.path}: size mismatch.`);
    complete = true;
  } finally {
    if (reader) {
      if (!complete) await reader.cancel().catch(() => {});
      reader.releaseLock();
    }
  }
  const digest = await crypto.subtle.digest("SHA-256", bytes);
  const hash = Array.from(new Uint8Array(digest), byte => byte.toString(16).padStart(2, "0")).join("");
  if (hash !== asset.sha256) throw new Error(`Offline asset ${asset.path}: SHA-256 mismatch.`);
  const headers = new Headers(response.headers);
  // Fetch exposes decoded bytes; do not retain wire encoding or compressed Content-Length.
  headers.delete("content-encoding"); headers.delete("content-length");
  return { canonical: canonical.href, response: new Response(bytes, { status: response.status, statusText: response.statusText, headers }) };
}

self.addEventListener("install", event => {
  event.waitUntil((async () => {
    const existed = await caches.has(CACHE);
    const cache = await caches.open(CACHE);
    const cancellation = new AbortController();
    let next = 0, failure;
    const download = async () => {
      while (!cancellation.signal.aborted && next < ASSETS.length) {
        const asset = ASSETS[next++];
        try {
          const verified = await verifiedAsset(asset, cancellation.signal);
          if (!cancellation.signal.aborted) await cache.put(verified.canonical, verified.response);
        } catch (error) {
          if (!cancellation.signal.aborted) { failure = error; cancellation.abort(error); }
        }
      }
    };
    // Wait for cancelled downloads and pending cache writes before deleting a failed generation.
    await Promise.all(Array.from({ length: Math.min(DOWNLOADS, ASSETS.length) }, download));
    if (cancellation.signal.aborted) {
      if (!existed) await caches.delete(CACHE);
      throw failure;
    }
  })());
});
self.addEventListener("activate", event => {
  event.waitUntil((async () => {
    for (const key of await caches.keys()) if (key.startsWith(PREFIX) && key !== CACHE) await caches.delete(key);
    await self.clients.claim();
  })());
});
self.addEventListener("message", event => {
  if (event.data?.type !== "reader:activate" || !event.source?.id || !event.ports?.[0]) return;
  const port = event.ports[0];
  event.waitUntil((async () => {
    try {
      const windows = await self.clients.matchAll({ type: "window", includeUncontrolled: true });
      const belongs = client => {
        const url = new URL(client.url);
        return url.origin === root.origin && url.pathname.startsWith(root.pathname);
      };
      if (!windows.some(client => client.id === event.source.id && belongs(client))) {
        port.postMessage({ status: "rejected" }); return;
      }
      if (windows.some(client => client.id !== event.source.id && belongs(client))) {
        port.postMessage({ status: "blocked" }); return;
      }
      // The requesting tab has explicitly saved its library and will reload on controllerchange.
      // Other open Reader windows must close first; they are never reloaded by this worker.
      port.postMessage({ status: "accepted", version: VERSION });
      await self.skipWaiting();
    } catch {
      port.postMessage({ status: "rejected" });
    }
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
