import test from "node:test";
import assert from "node:assert/strict";
import { httpUrl, proxyUrl, fetchFeed, refreshFeeds, decodeFeed } from "../crates/reader-browser/wwwroot/feeds.js";
import { parseRoute, routeFor } from "../crates/reader-browser/wwwroot/navigation.js";

test("untrusted feed and proxy addresses cannot execute code or carry credentials", () => {
  for (const value of ["javascript:alert(1)", "data:text/xml,test", "file:///etc/passwd", "https://user:password@example.com/feed"]) {
    assert.throws(() => httpUrl(value));
  }
  assert.equal(httpUrl("../feed.xml#item", "https://example.com/blog/"), "https://example.com/feed.xml");
  assert.throws(() => proxyUrl("https://example.com/proxy", "https://feed.example/rss"));
  assert.equal(proxyUrl("https://proxy.example/?url={url}", "https://feed.example/rss?a=1&b=2"), "https://proxy.example/?url=https%3A%2F%2Ffeed.example%2Frss%3Fa%3D1%26b%3D2");
});

test("routes preserve folder, tag and search punctuation across browser history", () => {
  for (const view of ["Home", "All", "Starred", { Feed: 123 }, { Folder: "C# / .NET" }, { Search: 'tag:"read later" & rust' }, { Tag: "ąęłź" }]) {
    assert.deepEqual(parseRoute(routeFor(view)), view);
  }
  assert.equal(parseRoute("#folder/%ZZ"), null);
  assert.equal(parseRoute("#feed/not-a-number"), null);
});

test("legacy feed encodings are decoded before Rust XML parsing", () => {
  const prefix = '<?xml version="1.0" encoding="windows-1252"?><title>caf';
  const bytes = new Uint8Array([...new TextEncoder().encode(prefix), 0xe9, ...new TextEncoder().encode('</title>')]);
  assert.match(decodeFeed({ bytes, url: "https://example.com/rss" }).contents, /café/);
});

test("refresh bounds transport concurrency, deduplicates in-flight downloads and reports individual failures", async () => {
  const previous = globalThis.fetch;
  let active = 0, peak = 0, calls = 0;
  globalThis.fetch = async url => {
    calls++; active++; peak = Math.max(active, peak);
    await new Promise(resolve => setTimeout(resolve, 5)); active--;
    if (url.endsWith("/bad")) return new Response("No", { status: 404 });
    return new Response("<rss><channel><title>Fixture</title></channel></rss>");
  };
  try {
    await Promise.all([fetchFeed("https://feed.example/one"), fetchFeed("https://feed.example/one")]);
    assert.equal(calls, 1);
    const feeds = Array.from({ length: 9 }, (_, i) => ({ url: `https://feed.example/${i}` }));
    feeds.push({ url: "https://feed.example/bad" });
    const received = [], errors = [];
    await refreshFeeds(feeds, "", (url, xml) => received.push([url, xml]), (url, error) => errors.push([url, error]));
    assert.equal(received.length, 9); assert.equal(errors.length, 1); assert(peak <= 4);
    assert.match(errors[0][1], /HTTP 404/);
  } finally { globalThis.fetch = previous; }
});

test("direct cross-origin failure uses only the explicitly configured proxy", async () => {
  const previous = globalThis.fetch, calls = [];
  globalThis.fetch = async url => {
    calls.push(url);
    if (url.startsWith("https://feed.example/")) throw new TypeError("Failed to fetch");
    return new Response('{"version":"https://jsonfeed.org/version/1.1","title":"Feed","items":[]}');
  };
  try {
    const feed = await fetchFeed("https://feed.example/rss", "https://proxy.example/?url={url}");
    assert.match(feed.contents, /jsonfeed/); assert.equal(calls.length, 2);
    assert(calls[1].startsWith("https://proxy.example/"));
  } finally { globalThis.fetch = previous; }
});

test("redirected refresh preserves subscription identity and the document base URL", async () => {
  const previous = globalThis.fetch;
  globalThis.fetch = async () => {
    const response = new Response('<feed xmlns="http://www.w3.org/2005/Atom"><title>Moved</title></feed>');
    Object.defineProperty(response, "url", { value: "https://new.example/blog/feed.xml" });
    return response;
  };
  try {
    let result;
    await refreshFeeds([{ id: 17, url: "https://old.example/feed" }], "", (...args) => { result = args; }, assert.fail);
    assert.equal(result[0], "https://old.example/feed");
    assert.equal(result[2], 17);
    assert.equal(result[3], "https://new.example/blog/feed.xml");
  } finally { globalThis.fetch = previous; }
});
