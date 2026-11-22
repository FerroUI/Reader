import assert from "node:assert/strict";
import { readingFixture } from "./fixtures.mjs";

const defaults = [
  ["Hacker News", "https://news.ycombinator.com/rss"],
  ["Ars Technica", "https://feeds.arstechnica.com/arstechnica/index"],
  ["The Verge", "https://www.theverge.com/rss/index.xml"],
  ["The Rust Blog", "https://blog.rust-lang.org/feed.xml"],
  ["The GitHub Blog", "https://github.blog/feed/"],
  ["BBC News", "https://feeds.bbci.co.uk/news/rss.xml"],
];
const urls = defaults.map(([, url]) => url).sort();
const replace = (h, data) => h.page.evaluate(`readerApp.importData(${JSON.stringify(JSON.stringify(data))})`);
const flush = h => h.page.evaluate("readerApp.flushSave(); import('./host.js').then(m => m.flush())");

function document(title, index) {
  return '<?xml version="1.0"?><rss version="2.0"><channel>' +
    `<title>${title}</title><link>https://starter-fixture.example/${index}/</link>` +
    '<description>Isolated first-launch feed response</description>' +
    [0, 1].map(item => `<item><guid>initial-feed-${index}-${item}</guid>` +
      `<title>Initial fetched article ${index + 1}.${item + 1}</title>` +
      `<link>https://starter-fixture.example/${index}/${item}</link>` +
      '<pubDate>Sun, 22 Nov 2026 10:00:00 GMT</pubDate>' +
      '<description>Downloaded and parsed by the production feed pipeline.</description></item>').join("") +
    '</channel></rss>';
}

/** Intercept only the configured public defaults in this isolated Chrome target.
 * Real app startup and Rust parsing run unchanged; no public publisher is needed.
 * Holding the first requests makes the empty loading state observable. */
export async function interceptStarterFeeds(page) {
  const bodies = new Map(defaults.map(([title, url], index) => [url, document(title, index)]));
  const pending = [], requests = [], errors = [];
  let released = false;
  async function respond(event) {
    const body = bodies.get(event.request.url);
    assert(body, `unexpected intercepted URL: ${event.request.url}`);
    const result = await page.send("Fetch.fulfillRequest", {
      requestId: event.requestId, responseCode: 200,
      responseHeaders: [{name: "Content-Type", value: "application/rss+xml; charset=utf-8"},
        {name: "Access-Control-Allow-Origin", value: "*"}, {name: "Cache-Control", value: "no-store"}],
      body: Buffer.from(body).toString("base64"),
    });
    assert(!result.error, JSON.stringify(result));
  }
  page.on("Fetch.requestPaused", (event, session) => {
    if (session) return;
    requests.push(event.request.url);
    if (!released) pending.push(event);
    else void respond(event).catch(error => errors.push(error));
  });
  const enabled = await page.send("Fetch.enable", {
    patterns: urls.map(urlPattern => ({urlPattern, requestStage: "Request"})),
  });
  assert(!enabled.error, JSON.stringify(enabled));
  return {
    requests,
    healthy() { if (errors.length) throw errors[0]; },
    async release() {
      released = true;
      await Promise.all(pending.splice(0).map(respond));
    },
  };
}

export async function starterChecks(h, check, fixture) {
  let initial, preserved;
  await check("fresh library loads real default feeds and preserves edits on reopening", async () => {
    const loading = await h.state();
    assert.deepEqual(loading.feeds.map(feed => feed.url).sort(), urls);
    assert.equal(loading.articles.length, 0, "fresh startup must contain no fabricated articles");
    assert(loading.feeds.every(feed => !feed.sample && feed.fetched_at === 0));
    // 01-home already captures this held first paint; do not spend another
    // screenshot delay against the production download timeout.
    await h.record("13-starter-loading", {feeds: loading.feeds, articleCount: 0});
    await fixture.release();
    initial = await h.until(async () => {
      fixture.healthy();
      const state = await h.state();
      return state.feeds.every(feed => feed.fetched_at > 0) && state.articles.length === 12 && state;
    }, "initial default subscriptions automatically download and parse their feeds");
    assert.deepEqual([...fixture.requests].sort(), urls, "each default is fetched exactly once");
    assert(initial.feeds.every(feed => !feed.sample));
    assert(initial.articles.every(article => article.guid.startsWith("initial-feed-") && !article.content.includes("Illustrative sample")));
    await h.capture("13-starter-ready");

    // Preserve a removed default, a renamed subscription and user annotations.
    preserved = structuredClone(initial);
    const removed = preserved.feeds.pop();
    preserved.articles = preserved.articles.filter(article => article.feed_id !== removed.id);
    preserved.feeds[0].title = "My technology feed";
    preserved.articles[0].starred = true;
    preserved.articles[0].note = "Keep this note after reopening.";
    preserved.articles[0].tags = ["Saved"];
    preserved.settings.density = "Cozy";
    await replace(h, preserved);
    await flush(h);
    await h.reload();
    await h.ready();
    const reopened = await h.state();
    assert.deepEqual(reopened.feeds, preserved.feeds, "removed defaults must not be reinserted or existing IDs reset");
    assert.deepEqual(reopened.articles, preserved.articles);
    assert.equal(reopened.settings.density, "Cozy");
    assert.equal(fixture.requests.length, 6, "already fetched defaults must not restart their first-load batch");
  });

  await check("an intentionally empty saved library does not restore default subscriptions", async () => {
    const empty = {...preserved, feeds: [], articles: [], folders: [], stream_settings: {}};
    await replace(h, empty);
    await flush(h);
    await h.reload();
    await h.ready();
    const reopened = await h.state();
    assert.deepEqual(reopened.feeds, []);
    assert.deepEqual(reopened.articles, []);
    assert.equal(reopened.settings.density, "Cozy", "an empty document remains a saved user library");
    assert.equal(fixture.requests.length, 6);
    fixture.healthy();
    await h.capture("13-starter-empty");
    await h.record("13-starter-verification", {
      defaultUrls: urls, requestedUrls: fixture.requests,
      initialSubscriptionIds: initial.feeds.map(feed => feed.id), initialArticleCount: initial.articles.length,
      reopenedSubscriptionIds: preserved.feeds.map(feed => feed.id),
      removedDefaultPreserved: true, annotationsPreserved: true, emptyLibraryPreserved: true,
    });
  });

  await replace(h, readingFixture(initial));
  await h.clickNavigation("Home");
}
