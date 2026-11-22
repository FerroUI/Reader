import test from "node:test";
import assert from "node:assert/strict";
import { articleConnection, downloadArticle, MAX_DOCUMENT } from "../crates/reader-browser/wwwroot/article-download.js";

const page = "<html><article><p>A publisher article.</p></article></html>";
const message = { url: "https://publisher.example/story#section", proxy: "", connection: "Automatic" };
async function withFetch(mock, run) {
  const saved = globalThis.fetch; globalThis.fetch = mock;
  try { await run(); } finally { globalThis.fetch = saved; }
}

test("publisher article success keeps redirect identity and omits cookies", async () => {
  const calls = [];
  await withFetch(async (url, options) => {
    calls.push(url); assert.equal(options.credentials, "omit"); assert.equal(options.referrerPolicy, "no-referrer");
    const response = new Response(page, { headers: { "content-type": "text/html" } });
    Object.defineProperty(response, "url", { value: "https://publisher.example/moved/story" });
    return response;
  }, async () => {
    const result = await downloadArticle(message);
    assert.equal(result.html, page); assert.equal(result.base_url, "https://publisher.example/moved/story");
    assert.deepEqual(calls, ["https://publisher.example/story"]);
  });
});

test("public article fallback requests HTML without executing the publisher's JavaScript", async () => {
  const calls = [];
  await withFetch(async (url, options) => {
    calls.push(url);
    if (calls.length === 1) throw new TypeError("Failed to fetch");
    assert.equal(url, "https://r.jina.ai/https://publisher.example/story");
    assert.equal(options.headers["X-Respond-With"], "html");
    assert.equal(options.headers["X-Respond-Timing"], "html");
    return new Response(page, { headers: { "content-type": "text/html" } });
  }, async () => {
    const result = await downloadArticle(message);
    assert.equal(result.base_url, "https://publisher.example/story"); assert.equal(calls.length, 2);
  });
});

test("private addresses use only the selected trusted connection", async () => {
  for (const url of ["http://localhost/story", "http://192.168.1.1/story", "https://private.example/story?token=secret", "https://private.example/private/story"]) {
    assert.equal(articleConnection(url).blocked, true);
    let calls = 0;
    await withFetch(async () => { calls++; throw new TypeError("CORS"); }, async () => {
      await assert.rejects(downloadArticle({ ...message, url }), /not sent to a public service/);
      assert.equal(calls, 1);
    });
  }
  const custom = articleConnection("https://private.example/story?token=secret", "https://trusted.example/?url={url}", "Custom");
  assert.equal(custom.public, false); assert(custom.url.includes(encodeURIComponent("?token=secret")));
  assert.equal(articleConnection(message.url, "https://trusted.example/?url={url}", "Direct"), null);
});

test("HTTP refusals and missing articles never trigger a public fallback", async () => {
  for (const status of [401, 403, 404, 500]) {
    let calls = 0;
    await withFetch(async () => { calls++; return new Response("Denied", { status }); }, async () => {
      await assert.rejects(downloadArticle(message)); assert.equal(calls, 1);
    });
  }
});

test("article download rejects oversized streams and non-HTML files", async () => {
  for (const declared of [true, false]) {
    let cancelled = false;
    const stream = new ReadableStream({ start(controller) { controller.enqueue(new Uint8Array(MAX_DOCUMENT + 1)); }, cancel() { cancelled = true; } });
    await withFetch(async () => new Response(stream, { headers: declared ? { "content-length": String(MAX_DOCUMENT + 1) } : {} }), async () => {
      await assert.rejects(downloadArticle(message), /4 MB/); assert(cancelled);
    });
  }
  await withFetch(async () => new Response("PDF", { headers: { "content-type": "application/pdf" } }), async () => {
    await assert.rejects(downloadArticle(message), /not an HTML article/);
  });
});

test("legacy article character sets decode before extraction", async () => {
  const bytes = new Uint8Array([...new TextEncoder().encode("<html><p>caf"), 0xe9, ...new TextEncoder().encode("</p></html>")]);
  await withFetch(async () => new Response(bytes, { headers: { "content-type": "text/html;charset=windows-1252" } }), async () => {
    assert.match((await downloadArticle(message)).html, /café/);
  });
});

test("article public service rate limits suppress immediate repeated relay calls", async () => {
  let relays = 0;
  await withFetch(async url => {
    if (!url.startsWith("https://r.jina.ai/")) throw new TypeError("CORS");
    relays++; return new Response("Wait", { status: 429, headers: { "retry-after": "120" } });
  }, async () => {
    await assert.rejects(downloadArticle(message), /busy/);
    await assert.rejects(downloadArticle(message), /asked Reader to wait/);
    assert.equal(relays, 1);
  });
});
