import test from "node:test";
import assert from "node:assert/strict";
import { feedConnection, publicFeedEligible } from "../crates/reader-browser/wwwroot/feed-connection.js";
import { isHtmlDocument } from "../crates/reader-browser/wwwroot/feed-discovery.js";
import { retryAfterMilliseconds } from "../crates/reader-browser/wwwroot/feed-errors.js";
import { fetchFeed } from "../crates/reader-browser/wwwroot/feeds.js";

test("automatic public-service routing excludes local, IP-literal and recognizable private links", async () => {
  const privateUrls = ["http://localhost/feed", "http://localhost./feed", "http://router.local./feed", "http://router.home.arpa/feed",
    "http://server.localdomain/feed", "http://192.168.1.2/rss", "https://[::1]/rss", "https://2130706433/rss",
    "https://publisher.example/rss?authToken=private", "https://publisher.example/rss?api_key=private",
    "https://publisher.example/rss?X-Amz-Signature=private", "https://publisher.example/private-token/feed",
    "https://publisher.example/1234567890abcdef1234567890abcdef/rss"];
  const previous = globalThis.fetch;
  const calls = [];
  globalThis.fetch = async url => {calls.push(url);throw new TypeError("Failed to fetch");};
  try {
    for (const address of privateUrls) {
      assert.equal(publicFeedEligible(address), false, address);
      await assert.rejects(fetchFeed(address), error => error.code === "private");
    }
    assert.equal(calls.length, privateUrls.length, "private addresses must only be attempted directly");
  } finally {globalThis.fetch = previous;}
  assert(publicFeedEligible("https://publisher.example/?feed=rss2"));
  assert(publicFeedEligible("https://www.youtube.com/feeds/videos.xml?channel_id=UCpublic"));
});

test("legacy custom endpoints override Automatic while Direct forbids all proxy use", () => {
  const address = "https://publisher.example/rss", proxy = "https://custom.example/?url={url}";
  assert.equal(feedConnection(address, proxy, "Automatic").public, false);
  assert.equal(feedConnection(address, proxy, "Custom").source, "the custom feed proxy");
  assert.equal(feedConnection(address, proxy, "Direct"), null);
  assert.throws(() => feedConnection(address, "", "Custom"), /custom feed proxy URL/);
  assert.throws(() => feedConnection(address, "https://broken.example/", "Automatic"), /contain \{url\}/);
});

test("HTML discovery uses MIME and document prefixes without treating mislabeled feeds as pages", () => {
  for (const contents of ["<!-- publisher --><!doctype html><html></html>", '<?xml version="1.0"?><!-- comment --><html></html>', "<head><link rel='alternate'></head>"])
    assert(isHtmlDocument({contents}), contents);
  assert(isHtmlDocument({contents:"<link rel='alternate'>",contentType:"text/html; charset=UTF-8"}));
  for (const contents of ['<?xml version="1.0"?><rss version="2.0"></rss>', '<feed xmlns="http://www.w3.org/2005/Atom"></feed>', '{"version":"https://jsonfeed.org/version/1.1"}'])
    assert.equal(isHtmlDocument({contents,contentType:"text/html"}), false);
});

test("Retry-After accepts server seconds or an HTTP date and bounds invalid values", () => {
  const now = Date.parse("2026-11-22T12:00:00Z");
  assert.equal(retryAfterMilliseconds("120", now), 120000);
  assert.equal(retryAfterMilliseconds("Sun, 22 Nov 2026 12:01:00 GMT", now), 60000);
  assert.equal(retryAfterMilliseconds("invalid", now), 0);
  assert.equal(retryAfterMilliseconds("-1", now), 0);
});
