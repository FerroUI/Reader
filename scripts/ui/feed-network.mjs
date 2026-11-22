import assert from "node:assert/strict";
import http from "node:http";

const replace = (h, data) => h.page.evaluate(`readerApp.importData(${JSON.stringify(JSON.stringify(data))})`);
const currentView = h => h.page.evaluate("JSON.parse(readerApp.currentView())");

async function listen(handler) {
  const server = http.createServer(handler);
  await new Promise((resolve, reject) => {
    server.once("error", reject);
    server.listen(0, "127.0.0.1", () => { server.off("error", reject); resolve(); });
  });
  return {server, origin: `http://127.0.0.1:${server.address().port}`};
}

async function close(server) {
  if (!server) return;
  server.closeAllConnections();
  await new Promise((resolve, reject) => server.close(error => error ? reject(error) : resolve()));
}

function send(response, {status = 200, type = "application/atom+xml", body}) {
  response.writeHead(status, {"content-type": type, "content-length": Buffer.byteLength(body), "cache-control": "no-store"});
  response.end(body);
}

/** A real cross-origin publisher deliberately omits ACAO. The isolated relay
 * returns only this fixture's four documents, so CI never needs a public proxy. */
async function fixture() {
  let revision = 1;
  const publisherRequests = [], relayRequests = [];
  const content = path => {
    if (path === "/publisher/index.html") return {type: "text/html", body:
      '<!-- Publisher preamble must not prevent discovery. --><!doctype html><html><head>' +
      '<base href="/publisher/"><link rel="alternate" type="application/atom+xml" href="feed.xml">' +
      '</head><body>Public feed discovery fixture</body></html>'};
    if (path === "/publisher/feed.xml") return {body:
      '<?xml version="1.0"?><feed xmlns="http://www.w3.org/2005/Atom">' +
      '<title>Cross-origin reader fixture</title><id>urn:reader:cors</id><updated>2026-11-22T10:00:00Z</updated>' +
      '<link href="./"/><entry><id>urn:reader:cors:one</id><title>Cross-origin first article</title>' +
      '<link href="articles/one"/><updated>2026-11-22T10:00:00Z</updated><content type="html">&lt;p&gt;Fetched through the selected relay.&lt;/p&gt;</content></entry>' +
      (revision > 1 ? '<entry><id>urn:reader:cors:two</id><title>Cross-origin refreshed article</title>' +
        '<link href="articles/two"/><updated>2026-11-22T11:00:00Z</updated><content>Refresh fixture.</content></entry>' : "") +
      '</feed>'};
    if (path === "/publisher/malformed.xml") return {type: "application/rss+xml", body:
      '<rss><channel><title>Malformed fixture</title></rss>'};
    return {status: 404, type: "text/plain", body: "The fixture feed was not found."};
  };
  const publisher = await listen((request, response) => {
    publisherRequests.push({path: request.url, origin: request.headers.origin || "", cookie: request.headers.cookie || "", authorization: request.headers.authorization || ""});
    send(response, content(new URL(request.url, "http://fixture.invalid").pathname));
  });
  let relay;
  try {
    const site = `${publisher.origin}/publisher/index.html`, feed = `${publisher.origin}/publisher/feed.xml`, missing = `${publisher.origin}/publisher/missing.xml`;
    const malformed = `${publisher.origin}/publisher/malformed.xml`;
    const allowed = new Set([site, feed, missing, malformed]);
    relay = await listen((request, response) => {
      response.setHeader("access-control-allow-origin", "*");
      const url = new URL(request.url, "http://fixture.invalid");
      const target = url.searchParams.get("url");
      if (url.pathname !== "/relay" || !allowed.has(target)) {
        send(response, {status: 400, type: "text/plain", body: "Unexpected relay target."});
        return;
      }
      relayRequests.push({target, origin: request.headers.origin || "", cookie: request.headers.cookie || "", authorization: request.headers.authorization || ""});
      send(response, content(new URL(target).pathname));
    });
    const proxy = `${relay.origin}/relay?url={url}`;
    return {site, feed, missing, malformed, proxy, publisherRequests, relayRequests,
      via: target => proxy.replace("{url}", encodeURIComponent(target)),
      refresh: () => { revision = 2; },
      close: () => Promise.all([close(relay.server), close(publisher.server)])};
  } catch (error) {
    await close(relay?.server); await close(publisher.server); throw error;
  }
}

export async function feedNetworkChecks(h, check) {
  const baseline = await h.state();
  const network = await fixture();
  const cookieName = "reader-network-fixture";
  let passed = false;
  try {
    await check("cross-origin discovery, relay subscription, refresh and failure reporting", async () => {
      const cookie = await h.page.send("Network.setCookie", {name: cookieName, value: "must-not-be-forwarded", url: network.site, path: "/", sameSite: "Lax"});
      assert(!cookie.error && cookie.success !== false, "the isolated profile must contain a cookie to exercise credential omission");
      const data = structuredClone(baseline);
      data.settings.feed_connection = "Automatic";
      data.settings.feed_proxy = network.proxy;
      data.settings.unread_only = false;
      data.settings.view_mode = "List";
      data.stream_settings = {};
      await replace(h, data);
      await h.clickNavigation("Home");
      const expected = {
        cors: [network.site, network.feed, network.missing, network.malformed],
        http: [{url: network.missing, status: 404}, {url: network.via(network.missing), status: 404}],
      };
      await h.expectNetworkErrors(expected, async logs => {
        await h.clickName("SubscribeButton");
        await h.edit("DialogInput", network.site, true);
        const subscribed = await h.until(async () => (await h.state()).feeds.find(feed => feed.url === network.feed),
          "a cross-origin website discovers and subscribes to its publisher feed");
        await h.until(async () => (await currentView(h)).Feed === subscribed.id, "the fetched subscription opens");
        const first = (await h.state()).articles.find(article => article.feed_id === subscribed.id);
        assert(first, "Rust must parse an article from the relayed Atom document");
        assert.equal(first.title, "Cross-origin first article");
        assert.equal(first.url, new URL("articles/one", network.feed).href, "relative article links use the publisher base");
        assert.equal((await h.state()).feeds.length, baseline.feeds.length + 1);
        assert(network.publisherRequests.some(request => request.path === "/publisher/index.html"), "the direct website request must reach the separate origin");
        assert(network.publisherRequests.some(request => request.path === "/publisher/feed.xml"), "the discovered feed must also try its publisher directly");
        assert(network.relayRequests.some(request => request.target === network.site), "website discovery must use the selected fallback");
        assert(network.relayRequests.some(request => request.target === network.feed), "feed fetching must use the selected fallback");
        assert(logs.some(log => log.kind === "cors-policy" && log.url === network.site), "Chromium must actually enforce CORS on the direct publisher response");
        await h.capture("12-cross-origin-feed");

        network.refresh();
        await h.clickLabel("Refresh");
        await h.until(async () => (await h.state()).articles.some(article => article.feed_id === subscribed.id && article.title === "Cross-origin refreshed article"),
          "the existing cross-origin subscription refreshes through its selected relay");
        const refreshed = await h.state();
        assert.equal(refreshed.feeds.filter(feed => feed.url === network.feed).length, 1);
        assert.equal(refreshed.feeds.length, baseline.feeds.length + 1, "refresh must not subscribe to the relay URL or duplicate the publisher");
        assert.equal(refreshed.feeds.find(feed => feed.url === network.feed).id, subscribed.id);
        assert.equal(refreshed.articles.filter(article => article.feed_id === subscribed.id).length, 2);
        assert.equal(refreshed.articles.find(article => article.title === first.title).id, first.id, "refresh must retain article identity");
        assert(network.relayRequests.filter(request => request.target === network.feed).length >= 2);
        await h.control(element => /Button$/.test(element.type) && element.automationName === "Refresh", "refresh completion");

        await h.clickName("SubscribeButton");
        await h.edit("DialogInput", network.missing, true);
        const failure = await h.control(element => /TextBlock$/.test(element.type) && /(?:HTTP\s*404|not found)/i.test(element.text || ""),
          "an unavailable feed reports its HTTP failure");
        assert.equal((await h.state()).feeds.length, baseline.feeds.length + 1, "a failed request must not create a subscription");
        assert.equal((await currentView(h)).Feed, subscribed.id, "a failed request must retain the current reading location");
        assert(network.relayRequests.some(request => request.target === network.missing));
        await h.capture("12-feed-error");

        await h.clickName("SubscribeButton");
        await h.edit("DialogInput", network.malformed, true);
        const parseFailure = await h.control(element => /TextBlock$/.test(element.type) && /^The feed is not valid XML:/.test(element.text || ""),
          "a string JsValue parse error becomes a readable native status message");
        assert.equal((await h.state()).feeds.length, baseline.feeds.length + 1, "a parse failure must not create a subscription");
        assert.equal((await currentView(h)).Feed, subscribed.id, "a parse failure must retain the current reading location");
        assert(network.relayRequests.some(request => request.target === network.malformed));
        assert([...network.publisherRequests, ...network.relayRequests].every(request => !request.cookie && !request.authorization),
          "public feed transport must not forward browser cookies or authorization");
        await h.capture("12-feed-parse-error");
        await h.record("12-feed-network", {publisherUrl: network.feed, subscriptionId: subscribed.id,
          articleCountAfterRefresh: 2, cookieSeeded: true, error: failure.text, parseError: parseFailure.text, expectedNetworkLogs: logs,
          publisherRequests: network.publisherRequests, relayRequests: network.relayRequests});
      });
    });
    passed = true;
  } finally {
    try {
      await h.page.send("Network.deleteCookies", {name: cookieName, url: network.site});
      if (passed) {
        await h.page.press("Escape");
        await replace(h, baseline);
        await h.clickNavigation("All items");
      }
    } finally {
      await network.close();
    }
  }
}
