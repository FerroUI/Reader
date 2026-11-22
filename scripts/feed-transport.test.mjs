import test from "node:test";
import assert from "node:assert/strict";
import { fetchFeed, refreshFeeds } from "../crates/reader-browser/wwwroot/feeds.js";
import { download } from "../crates/reader-browser/wwwroot/feed-download.js";
import { PUBLIC_FEED_ENDPOINT } from "../crates/reader-browser/wwwroot/feed-public.js";
import { attachRuntime, request } from "../crates/reader-browser/wwwroot/host.js";

const xml = '<rss><channel><title>Public fixture</title></channel></rss>';
const service = () => new Response(JSON.stringify({status:"ok",feed:{title:"Relayed feed",link:"https://publisher.example/"},items:[
  {guid:"opaque-id",title:"Public article",link:"https://publisher.example/story",pubDate:"2026-11-22 10:00:00",content:"<p>Complete article.</p>"}
]}), {headers:{"content-type":"application/json"}});
async function withFetch(mock, run) {
  const previous = globalThis.fetch;
  globalThis.fetch = mock;
  try { await run(); } finally { globalThis.fetch = previous; }
}

test("automatic connection uses the public feed service after a CORS failure and retains publisher identity", async () => {
  const calls = [], address = "https://publisher.example/rss?feed=rss2";
  await withFetch(async (url, options) => {
    calls.push(url);
    assert.equal(options.credentials, "omit"); assert.equal(options.referrerPolicy, "no-referrer");
    if (url === address) throw new TypeError("Failed to fetch");
    assert.equal(url, PUBLIC_FEED_ENDPOINT.replace("{url}", encodeURIComponent(address)));
    return service();
  }, async () => {
    const result = await fetchFeed(address);
    assert.equal(result.url, address);
    const feed = JSON.parse(result.contents);
    assert.equal(feed.feed_url, address); assert.equal(feed.items[0].id, "opaque-id");
    assert.equal(feed.items[0].content_html, "<p>Complete article.</p>");
    assert.equal(calls.length, 2);
  });
});

test("direct success avoids the public service and Direct policy ignores a saved custom endpoint", async () => {
  const calls = [];
  await withFetch(async url => { calls.push(url); return new Response(xml); }, async () => {
    await fetchFeed("https://direct.example/rss"); assert.deepEqual(calls, ["https://direct.example/rss"]);
  });
  await withFetch(async url => { calls.push(url); throw new TypeError("Failed to fetch"); }, async () => {
    await assert.rejects(fetchFeed("https://private.example/rss", "https://custom.example/?url={url}", "Direct"),
      error => error.code === "network" && /direct browser access/.test(error.message));
    assert.equal(calls.length, 2);
  });
});

test("readable HTTP and encoding failures are terminal and never disguised as CORS", async () => {
  for (const response of [new Response("Missing", {status:404}), new Response(xml, {headers:{"content-type":"text/xml; charset=unsupported-test-encoding"}})]) {
    let calls = 0;
    await withFetch(async () => { calls++; return response; }, async () => {
      await assert.rejects(fetchFeed("https://terminal.example/rss"), error =>
        (error.code === "http" && error.status === 404) || error.code === "encoding");
      assert.equal(calls, 1);
    });
  }
});

test("declared and streamed size rejection cancel the body instead of invoking another endpoint", async () => {
  for (const declared of [true, false]) {
    let cancelled = false;
    const body = new ReadableStream({start(controller) { controller.enqueue(new Uint8Array(4)); }, cancel() { cancelled = true; }});
    await withFetch(async () => new Response(body, {headers:declared ? {"content-length":"4"} : {}}), async () => {
      await assert.rejects(download("https://large.example/rss", {limit:3}), error => error.code === "size");
      assert(cancelled);
    });
  }
});

test("offline failure avoids the public relay and preserves an actionable cause", async () => {
  const previous = Object.getOwnPropertyDescriptor(globalThis, "navigator");
  Object.defineProperty(globalThis, "navigator", {value:{onLine:false}, configurable:true});
  let calls = 0;
  try {
    await withFetch(async () => { calls++; throw new TypeError("Failed to fetch"); }, async () => {
      await assert.rejects(fetchFeed("https://offline.example/rss"), error => error.code === "offline" && /offline/.test(error.message));
      assert.equal(calls, 1);
    });
  } finally { if (previous) Object.defineProperty(globalThis, "navigator", previous); else delete globalThis.navigator; }
});

test("public service errors are reported without creating a fake empty feed", async () => {
  await withFetch(async url => {
    if (url.startsWith("https://bad-feed.example/")) throw new TypeError("Failed to fetch");
    return new Response(JSON.stringify({status:"error",message:"Feed could not be loaded"}));
  }, async () => {
    await assert.rejects(fetchFeed("https://bad-feed.example/feed"), error => error.code === "service" && /rss2json.*could not/i.test(error.message));
  });
});

test("string exceptions from Rust feed parsing survive subscribe and refresh host boundaries", async () => {
  const errors = [];
  let finished;
  const complete = new Promise(resolve => { finished = resolve; });
  attachRuntime({libraryGeneration:()=>312,currentView:()=> '"Home"',receiveFeed(){throw "The feed is not valid XML: fixture";},
    feedError:(url,message)=>errors.push([url,message]),finishRefresh:finished,hostError:assert.fail,storageError:assert.fail});
  await withFetch(async () => new Response("<rss>invalid"), async () => {
    request(JSON.stringify({kind:"subscribe",url:"https://invalid.example/rss",proxy:"",connection:"Direct"}));
    await complete;
    await refreshFeeds([{id:1,url:"https://invalid.example/refresh"}], "", () => {throw "The feed is not valid XML: refresh";}, (url,message)=>errors.push([url,message]), "Direct");
  });
  assert.equal(errors[0][1], "The feed is not valid XML: fixture");
  assert.equal(errors[1][1], "The feed is not valid XML: refresh");
});

test("public relay rate limits respect Retry-After without automatic retry or provider hopping", async () => {
  let relayCalls = 0;
  await withFetch(async url => {
    if (url.startsWith("https://limited.example/")) throw new TypeError("Failed to fetch");
    relayCalls++;
    return new Response("Rate limited", {status:429,headers:{"retry-after":"120"}});
  }, async () => {
    await assert.rejects(fetchFeed("https://limited.example/one"), error => error.status === 429 && error.retryAfter === 120000);
    await assert.rejects(fetchFeed("https://limited.example/two"), error => error.status === 429 && error.retryAfter > 0);
    assert.equal(relayCalls, 1);
  });
});
