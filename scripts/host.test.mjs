import test from "node:test";
import assert from "node:assert/strict";
import { httpUrl, proxyUrl, fetchFeed, refreshFeeds, decodeFeed } from "../crates/reader-browser/wwwroot/feeds.js";
import { parseRoute, routeFor, attachNavigation, navigationChanged } from "../crates/reader-browser/wwwroot/navigation.js";
import { viewportRectangle } from "../crates/reader-browser/wwwroot/viewport.js";
import { articleUrl } from "../crates/reader-browser/wwwroot/transfers.js";
import { attachInputGuard } from "../crates/reader-browser/wwwroot/input.js";
import { attachRuntime, request } from "../crates/reader-browser/wwwroot/host.js";

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
  for (const hash of ["#home/invalid", "#constructor", "#__proto__/test", "#feed/9007199254740992"]) assert.equal(parseRoute(hash), null);
});

function historyFixture() {
  const events = new EventTarget(), microtasks = [], traversals = [];
  const entries = [{ url: "#home", state: null }];
  let position = 0, view = '"Home"', overlay = false;
  const location = { get hash() { return entries[position].url; } };
  const history = {
    get state() { return entries[position].state; },
    pushState(state, _, url) { entries.splice(++position); entries.push({ url, state }); },
    replaceState(state, _, url) { entries[position] = { url, state }; },
    back() { traversals.push(() => { if (position > 0) { position--; events.dispatchEvent(new Event("popstate")); } }); },
  };
  const browser = { history, location, document: {}, queueMicrotask: task => microtasks.push(task),
    addEventListener: events.addEventListener.bind(events), removeEventListener: events.removeEventListener.bind(events) };
  const runtime = {
    currentView: () => view,
    navigate(value) { view = value; navigationChanged(); },
    hasOverlay: () => overlay,
    dismissOverlay() { const wasOpen = overlay; overlay = false; if (wasOpen) navigationChanged(); return wasOpen; },
  };
  const dispose = attachNavigation(runtime, browser);
  const drainMicrotasks = () => { while (microtasks.length) microtasks.shift()(); };
  const settle = () => {
    for (let n = 0; microtasks.length || traversals.length; n++) {
      assert(n < 30, "history synchronization did not settle");
      drainMicrotasks(); if (traversals.length) traversals.shift()();
    }
  };
  return { browser, runtime, dispose, settle, drainMicrotasks,
    open(value = true) { overlay = value; navigationChanged(); },
    routes: () => entries.slice(0, position + 1).map(entry => entry.url) };
}

test("browser Back closes the native overlay before returning to the previous stream", () => {
  const app = historyFixture();
  try {
    app.runtime.navigate('"All"'); app.settle();
    app.open(); app.settle();
    assert.deepEqual(app.routes(), ["#home", "#all", "#all"]);
    app.browser.history.back(); app.settle();
    assert.equal(app.runtime.hasOverlay(), false);
    assert.equal(app.runtime.currentView(), '"All"');
    app.browser.history.back(); app.settle();
    assert.equal(app.runtime.currentView(), '"Home"');
    assert.equal(app.browser.document.title, "Home – Reader");
  } finally { app.dispose(); }
});

test("native dialog close and route changes do not leave phantom history entries or lose a pending route", () => {
  const app = historyFixture();
  try {
    app.runtime.navigate('"All"'); app.settle();
    app.open(); app.settle();
    app.open(false); app.drainMicrotasks(); // history.back is pending
    app.runtime.navigate('"Starred"'); app.settle();
    assert.deepEqual(app.routes(), ["#home", "#all", "#starred"]);
    app.open(); app.settle();
    app.runtime.navigate('"Trends"'); app.settle();
    assert.equal(app.runtime.hasOverlay(), false);
    assert.deepEqual(app.routes(), ["#home", "#all", "#starred", "#trends"]);
  } finally { app.dispose(); }
});

test("keyboard-sized visual viewport follows panning without undoing pinch magnification", () => {
  const layout = { width: 390, height: 844 };
  assert.deepEqual(viewportRectangle(layout, { width: 390, height: 380, offsetLeft: 0, offsetTop: 30, scale: 1 }),
    { width: 390, height: 380, left: 0, top: 30 });
  assert.equal(viewportRectangle(layout, { width: 195, height: 422, scale: 2 }), null);
  assert.deepEqual(viewportRectangle(layout), { ...layout, left: 0, top: 0 });
  assert.deepEqual(viewportRectangle(layout, { width: NaN, height: 0, offsetTop: -10, scale: 1 }),
    { ...layout, left: 0, top: 0 });
});

test("article operations retain fragments while rejecting executable and credentialed links", () => {
  assert.equal(articleUrl("#part-2", "https://example.com/article"), "https://example.com/article#part-2");
  assert.equal(articleUrl("https://example.com/a?x=1#heading"), "https://example.com/a?x=1#heading");
  for (const url of ["javascript:alert(1)", "data:text/html,example", "https://user:secret@example.com/article"]) assert.throws(() => articleUrl(url));
});

test("IME candidate keys bypass app shortcuts without canceling browser composition", () => {
  const listeners = new Map();
  const element = {
    addEventListener(name, callback) { listeners.set(name, callback); },
    removeEventListener(name) { listeners.delete(name); },
  };
  const dispose = attachInputGuard(element);
  const key = (properties = {}) => {
    const event = { key: "Enter", keyCode: 13, isComposing: false, stopped: false, prevented: false,
      stopPropagation() { this.stopped = true; }, preventDefault() { this.prevented = true; }, ...properties };
    listeners.get("keydown")(event);
    return event;
  };
  assert.equal(key().stopped, false);
  listeners.get("compositionstart")();
  assert.equal(key().stopped, true);
  listeners.get("compositionend")();
  for (const properties of [{ isComposing: true }, { key: "Dead" }, { keyCode: 229 }]) {
    const event = key(properties); assert.equal(event.stopped, true); assert.equal(event.prevented, false);
  }
  assert.equal(key().stopped, false);
  dispose(); assert.equal(listeners.size, 0);
});

test("a completed subscription cannot take navigation away from a newer reading location", async () => {
  const previous = globalThis.fetch;
  let release, view = '"Home"', received;
  const completed = new Promise(resolve => {
    attachRuntime({ currentView: () => view, libraryGeneration: () => 0, receiveFeed(...args) { received = args; },
      feedError: assert.fail, storageError: assert.fail, hostError: assert.fail, finishRefresh: resolve });
  });
  globalThis.fetch = () => new Promise(resolve => { release = resolve; });
  try {
    request(JSON.stringify({ kind: "subscribe", url: "https://pending.example/feed", proxy: "" }));
    await new Promise(resolve => setImmediate(resolve));
    view = '"Starred"';
    release(new Response("<rss><channel><title>Pending</title></channel></rss>"));
    await completed;
    assert.equal(received[2], false);
    assert.equal(received[0], "https://pending.example/feed");
  } finally { globalThis.fetch = previous; }
});

test("an IndexedDB version change reports immediately and protects the loaded document from later writes", async () => {
  const previous = { indexedDB: globalThis.indexedDB, localStorage: globalThis.localStorage };
  let closed = false;
  const database = {
    close() { closed = true; },
    transaction() { return { objectStore() { return { get() {
      const read = { result: { contents: '{"saved":true}', generation: 4 } };
      queueMicrotask(() => read.onsuccess()); return read;
    } }; } }; },
  };
  globalThis.indexedDB = { open() {
    const opening = { result: database }; queueMicrotask(() => opening.onsuccess()); return opening;
  } };
  globalThis.localStorage = { getItem() { return null; } };
  try {
    const storage = await import("../crates/reader-browser/wwwroot/reader-storage.js?lifecycle-test");
    await storage.prepareStorage();
    const errors = []; storage.setStorageReporter(error => errors.push(error.message));
    database.onversionchange();
    assert.equal(closed, true); assert.match(errors[0], /another tab/);
    assert.throws(() => storage.save('{"changed":true}'), /reload Reader/);
    assert.equal(storage.load(), '{"saved":true}');
  } finally { Object.assign(globalThis, previous); }
});

test("backup replacement rejects old feed results and its busy state ignores the old generation", async () => {
  const previous = globalThis.fetch;
  let generation = 20;
  const releases = new Map(), received = [], finished = [];
  attachRuntime({ currentView: () => '"All"', libraryGeneration: () => generation,
    receiveFeed(...args) { received.push(args); }, finishRefresh() { finished.push(generation); },
    feedError: assert.fail, storageError: assert.fail, hostError: assert.fail });
  globalThis.fetch = url => new Promise(resolve => releases.set(url, resolve));
  const settle = () => new Promise(resolve => setImmediate(resolve));
  try {
    request(JSON.stringify({ kind: "refresh", feeds: [{ id: 17, url: "https://stale.example/feed" }], proxy: "" }));
    await settle();
    generation++;
    request(JSON.stringify({ kind: "subscribe", url: "https://current.example/feed", proxy: "" }));
    await settle();
    releases.get("https://current.example/feed")(new Response("<rss><channel><title>Current</title></channel></rss>"));
    await settle();
    assert.equal(received.length, 1); assert.deepEqual(finished, [21]);
    releases.get("https://stale.example/feed")(new Response("<rss><channel><title>Stale</title></channel></rss>"));
    await settle();
    assert.equal(received.length, 1, "same feed identity cannot revalidate a request from a replaced library");
    assert.deepEqual(finished, [21], "a stale request cannot finish the new library's busy state");
  } finally { globalThis.fetch = previous; }
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
