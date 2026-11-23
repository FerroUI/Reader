import test from "node:test";
import assert from "node:assert/strict";
import { createHash, webcrypto } from "node:crypto";
import { mkdtemp, mkdir, readFile, rm, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { runInNewContext } from "node:vm";
import { prepareSite } from "./prepare-site.mjs";

const origin = "https://example.com/Reader/";
const hash = bytes => createHash("sha256").update(bytes).digest("hex");

async function fixture(t) {
  const directory = await mkdtemp(path.join(os.tmpdir(), "reader-offline-"));
  t.after(() => rm(directory, { recursive: true, force: true }));
  const files = {
    "index.html": '<!doctype html><html lang="en" data-reader-build="__BUILD_VERSION__"><head><link href="./app.css" rel="stylesheet"></head><body><script src="./main.js" type="module"></script></body></html>',
    "app.css": "body { color: #333; }",
    "main.js": "export const version = 1;",
    "reader-storage.js": "export const storage = true;",
    "reader-browser.js": "export default function runtime() {}",
    "reader_browser.wasm": Buffer.from([0, 97, 115, 109, 1, 0, 0, 0]),
    "ferroui.js": "export const framework = true;",
    "ferroui-loader.js": "export const loader = true;",
    "fonts/reader font.ttf": Buffer.from([1, 2, 3, 4]),
    "qa/ignored.json": "{}",
  };
  for (const [name, bytes] of Object.entries(files)) {
    await mkdir(path.dirname(path.join(directory, name)), { recursive: true });
    await writeFile(path.join(directory, name), bytes);
  }
  return directory;
}

// Execute the generated worker with real Web Crypto and Fetch Response streams.
// Only network and CacheStorage are replaced; tests dispatch its actual install/activate handlers.
async function worker(directory, build, tamper, windows = []) {
  const buckets = new Map([
    ["ferroui-reader-previous", new Map([[origin + "index.html", new Response("previous build")]])],
    ["another-application", new Map()],
  ]);
  const events = new Map(), requests = [], deleted = [];
  let active = 0, peak = 0, claims = 0, activations = 0;
  const caches = {
    has: async name => buckets.has(name),
    keys: async () => [...buckets.keys()],
    delete: async name => { deleted.push(name); return buckets.delete(name); },
    open: async name => {
      if (!buckets.has(name)) buckets.set(name, new Map());
      const entries = buckets.get(name);
      return {
        put: async (url, response) => { entries.set(url, new Response(await response.arrayBuffer(), { headers: response.headers })); },
        match: async (url, options = {}) => {
          const key = typeof url === "string" ? url : url.url;
          const entry = options.ignoreSearch
            ? [...entries].find(([stored]) => new URL(stored).pathname === new URL(key).pathname)?.[1]
            : entries.get(key);
          return entry?.clone();
        },
      };
    },
  };
  const fetch = async (address, options) => {
    const url = new URL(address);
    requests.push({ url: address, options });
    active++; peak = Math.max(peak, active);
    try {
      await new Promise(resolve => setTimeout(resolve, 2));
      if (options.signal.aborted) throw options.signal.reason;
      const filename = decodeURIComponent(url.pathname.slice(new URL(origin).pathname.length));
      let bytes = await readFile(path.join(directory, filename));
      if (filename === tamper) { bytes = Buffer.from(bytes); bytes[0] ^= 1; }
      return new Response(bytes, { headers: { "content-type": filename.endsWith(".wasm") ? "application/wasm" : "text/plain" } });
    } finally { active--; }
  };
  const self = {
    location: { href: origin + "app-sw.js" },
    clients: { claim: async () => { claims++; }, matchAll: async () => windows },
    addEventListener: (event, handler) => events.set(event, handler),
    skipWaiting: async () => { activations++; },
  };
  runInNewContext(await readFile(path.join(directory, "app-sw.js"), "utf8"), {
    self, caches, fetch, URL, Response, Headers, AbortController, crypto: webcrypto, Uint8Array,
  });
  const dispatch = event => {
    let completion;
    events.get(event)({ waitUntil: promise => { completion = promise; } });
    assert(completion, `${event} must retain its asynchronous work`);
    return completion;
  };
  const message = async (source, type = "reader:activate") => {
    let completion, response;
    events.get("message")({ data: { type }, source: source ? { id: source } : null,
      ports: [{ postMessage: value => { response = structuredClone(value); } }],
      waitUntil: promise => { completion = promise; },
    });
    await completion;
    return response;
  };
  return { buckets, requests, deleted, dispatch, message, active: () => active,
    peak: () => peak, claims: () => claims, activations: () => activations };
}

test("offline manifest hashes final stamped assets and preparation is idempotent", async t => {
  const directory = await fixture(t);
  const first = await prepareSite(directory);
  const index = await readFile(path.join(directory, "index.html"), "utf8");
  const script = await readFile(path.join(directory, "app-sw.js"), "utf8");
  assert(index.includes(`data-reader-build="${first.version}"`));
  assert(index.includes(`main.js?v=${first.version}`));
  assert(first.assets.includes("./fonts/reader%20font.ttf"));
  assert(!first.assets.some(name => /qa|build\.json|app-sw\.js/.test(name)));
  for (const asset of first.manifest) {
    const bytes = await readFile(path.join(directory, decodeURIComponent(asset.path)));
    assert.equal(asset.sha256, hash(bytes));
    assert.equal(asset.size, bytes.length);
  }
  const second = await prepareSite(directory);
  assert.deepEqual(second, first);
  assert.equal(await readFile(path.join(directory, "index.html"), "utf8"), index);
  assert.equal(await readFile(path.join(directory, "app-sw.js"), "utf8"), script);
  await writeFile(path.join(directory, "main.js"), "export const version = 2;");
  const changed = await prepareSite(directory);
  assert.notEqual(changed.version, first.version);
  assert.notEqual(changed.manifest.find(a => a.path === "./index.html").sha256, first.manifest.find(a => a.path === "./index.html").sha256);
});

test("verified installation uses canonical cache keys and preserves the old generation until activation", async t => {
  const directory = await fixture(t), build = await prepareSite(directory);
  const host = await worker(directory, build);
  await host.dispatch("install");
  const entries = host.buckets.get("ferroui-reader-" + build.version);
  assert.equal(entries.size, build.manifest.length);
  for (const asset of build.manifest) {
    const canonical = new URL(asset.path, origin).href;
    const response = entries.get(canonical);
    assert(response, `canonical cache key ${canonical}`);
    assert.equal(hash(Buffer.from(await response.clone().arrayBuffer())), asset.sha256);
  }
  for (const { url, options } of host.requests) {
    assert.equal(new URL(url).searchParams.get("__reader_build"), build.version);
    assert.equal(options.cache, "reload");
  }
  assert(host.peak() <= 2); assert.equal(host.active(), 0);
  assert(host.buckets.has("ferroui-reader-previous")); assert.equal(host.claims(), 0);
  assert.equal(host.activations(), 0, "installation never forces an update");
  await host.dispatch("activate");
  assert(!host.buckets.has("ferroui-reader-previous"));
  assert(host.buckets.has("another-application")); assert.equal(host.claims(), 1);
});

test("only an explicit same-scope client can request activation with no other Reader windows", async t => {
  const directory = await fixture(t), build = await prepareSite(directory);
  const windows = [{ id: "reader", url: origin + "#feed/1" }, { id: "other-app", url: "https://example.com/Other/" }];
  const host = await worker(directory, build, undefined, windows);
  await host.dispatch("install");
  assert.equal(host.activations(), 0);
  assert.equal(await host.message("reader", "unknown"), undefined);
  assert.equal(await host.message(null), undefined);
  assert.equal((await host.message("other-app")).status, "rejected");
  windows.push({ id: "another-reader", url: origin + "#home" });
  assert.equal((await host.message("reader")).status, "blocked");
  assert.equal(host.activations(), 0);
  assert(host.buckets.has("ferroui-reader-previous"));
  windows.pop();
  assert.deepEqual(await host.message("reader"), { status: "accepted", version: build.version });
  assert.equal(host.activations(), 1);
  assert.equal(host.claims(), 0, "the browser still owns the actual activation lifecycle");
});

test("same-length tampering rejects installation and removes only its failed generation", async t => {
  const directory = await fixture(t), build = await prepareSite(directory);
  const host = await worker(directory, build, "main.js");
  await assert.rejects(host.dispatch("install"), /main\.js: SHA-256 mismatch/);
  assert(!host.buckets.has("ferroui-reader-" + build.version));
  assert.equal(await host.buckets.get("ferroui-reader-previous").get(origin + "index.html").clone().text(), "previous build");
  assert(host.buckets.has("another-application"));
  assert.deepEqual(host.deleted, ["ferroui-reader-" + build.version]);
  assert.equal(host.active(), 0); assert(host.peak() <= 2); assert.equal(host.claims(), 0);
});
