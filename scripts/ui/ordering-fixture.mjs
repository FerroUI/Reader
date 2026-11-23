import assert from "node:assert/strict";
import { sleep } from "../../FerroUI/scripts/browser/harness.mjs";

export const BASE = 860000, ALPHA = "Alpha group", BETA = "Beta group", GAMMA = "Gamma group";
export const TITLES = ["Zulu shared", "Alpha notes", "Beta notes", "Zebra unfiled", "Alpha unfiled"];
export const center = e => [e.x + e.width / 2, e.y + e.height / 2];
export const isRow = e => e.name === "SubscriptionLink" || e.name === "SubscriptionFolder";
export const label = e => e.name === "SubscriptionFolder" ? e.automationName
  : TITLES.find(title => e.automationName === title || e.automationName?.startsWith(title + " ("));
export const rows = async h => (await h.snapshot()).elements.filter(isRow).sort((a, b) => a.y - b.y);
export const replace = (h, data) => h.page.evaluate(`readerApp.importData(${JSON.stringify(JSON.stringify(data))})`);
export const navigate = (h, view) => h.page.evaluate(`readerApp.navigate(${JSON.stringify(JSON.stringify(view))})`);

export function fixture(baseline) {
  const data = structuredClone(baseline), now = Math.floor(Date.now() / 1000);
  data.feeds = TITLES.map((title, index) => ({id: BASE + index, title,
    url: `https://ordering.invalid/${index}.xml`, site_url: "https://ordering.invalid/",
    folder: index < 3 ? ALPHA : "", folders: index < 2 ? [ALPHA, BETA] : index === 2 ? [ALPHA] : [],
    description: "", fetched_at: now, error: null, sample: false}));
  data.articles = data.feeds.map((feed, index) => ({id: BASE + 100 + index,
    feed_id: feed.id, guid: "order-article-" + index, title: "Ordering article " + index,
    url: `https://ordering.invalid/articles/${index}`, content: "<p>Ordering preserves the article and its reading state.</p>",
    summary: "Ordering fixture", author: "", published: now - index,
    read: false, starred: false, tags: [], note: "", read_at: null}));
  data.folders = [ALPHA, BETA, GAMMA]; data.stream_settings = {}; data.next_id = BASE + 200;
  data.subscription_order = {sort: "Manual", root: [{Feed: BASE + 3}, {Folder: BETA},
    {Feed: BASE + 4}, {Folder: ALPHA}, {Folder: GAMMA}],
    folders: {[ALPHA]: [BASE, BASE + 1, BASE + 2], [BETA]: [BASE, BASE + 1]}};
  Object.assign(data.settings, {view_mode: "Expanded", density: "Comfortable", unread_only: false,
    font_size: 14, sort: "Newest", mark_read_on_scroll: false, show_all_subscriptions: true,
    feed_connection: "Direct", feed_proxy: "", send_to: []});
  return data;
}

export const folder = (h, name) => h.control(e => e.name === "SubscriptionFolder" && e.automationName === name, name);
export async function occurrence(h, name, parent) {
  return h.until(async () => {
    const tree = await rows(h), start = tree.findIndex(e => e.name === "SubscriptionFolder" && label(e) === parent);
    assert(start >= 0, parent + " is visible");
    const end = tree.findIndex((e, i) => i > start && e.name === "SubscriptionFolder");
    return tree.slice(start + 1, end < 0 ? undefined : end).find(e => e.name === "SubscriptionLink" && label(e) === name && e.hit);
  }, `${name} in ${parent}`);
}

export async function context(h, element) {
  const [x, y] = center(element);
  await h.page.send("Input.dispatchMouseEvent", {type: "mouseMoved", x, y, pointerType: "mouse"});
  await h.page.send("Input.dispatchMouseEvent", {type: "mousePressed", x, y, button: "right", buttons: 2, clickCount: 1, pointerType: "mouse"});
  await h.page.send("Input.dispatchMouseEvent", {type: "mouseReleased", x, y, button: "right", buttons: 0, clickCount: 1, pointerType: "mouse"});
  return h.control(e => e.name === "ReaderMenu", "subscription context menu");
}

export async function drag(h, source, target, beforeRelease) {
  const [x, y] = center(source);
  await h.page.send("Input.dispatchMouseEvent", {type: "mouseMoved", x, y, pointerType: "mouse"});
  await h.page.send("Input.dispatchMouseEvent", {type: "mousePressed", x, y, button: "left", buttons: 1, clickCount: 1, pointerType: "mouse"});
  for (let step = 1; step <= 8; step++) {
    await h.page.send("Input.dispatchMouseEvent", {type: "mouseMoved", x: x + (target[0] - x) * step / 8,
      y: y + (target[1] - y) * step / 8, button: "left", buttons: 1, pointerType: "mouse"});
    await sleep(25);
  }
  try { await beforeRelease?.(); }
  finally {
    await h.page.send("Input.dispatchMouseEvent", {type: "mouseReleased", x: target[0], y: target[1],
      button: "left", buttons: 0, clickCount: 1, pointerType: "mouse"});
  }
}

export async function expectRows(h, expected) {
  return h.until(async () => { assert.deepEqual((await rows(h)).map(label), expected); return true; }, "the native subscription tree follows its ordering");
}
