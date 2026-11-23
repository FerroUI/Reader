import assert from "node:assert/strict";
import { writeFile, rm } from "node:fs/promises";
import path from "node:path";
import { sleep } from "../../FerroUI/scripts/browser/harness.mjs";

export const BASE = 840000;
export const GROUP = "Engineering";
export const sourceName = index => ["Compiler notes", "Runtime notes", "Design notes"][index];
export const replace = (h, data) => h.page.evaluate(`readerApp.importData(${JSON.stringify(JSON.stringify(data))})`);
export const navigate = (h, view) => h.page.evaluate(`readerApp.navigate(${JSON.stringify(JSON.stringify(view))})`);
export const active = data => data.feeds.filter(feed => !feed.archived);
export const folder = (element, name = GROUP) => element.name === "SubscriptionFolder" && element.automationName === name;
export const center = element => [element.x + element.width / 2, element.y + element.height / 2];

export async function installFixture(h, baseline, site) {
  const data = structuredClone(baseline);
  const files = [0, 1, 2].map(index => `verification-folder-${index}.xml`);
  data.feeds = files.map((file, index) => ({
    id: BASE + index, title: sourceName(index), url: new URL(file, h.page.url).href,
    site_url: "https://folder-fixture.example/", fetched_at: 1,
    folder: index < 2 ? GROUP : "Reference",
    folders: index === 0 ? [GROUP] : index === 1 ? [GROUP, "Reference"] : ["Reference"],
  }));
  const makeArticle = (index, feed, title, saved = {}) => ({
    id: BASE + 100 + index, feed_id: BASE + feed, guid: `folder-article-${index}`,
    title, url: `https://folder-fixture.example/articles/${index}`,
    content: `<p>${title}. Local article content remains available with its original source.</p>`,
    summary: title, author: "Folder fixture", published: 1795341600 - index,
    read: false, starred: false, tags: [], note: "", read_at: null, ...saved,
  });
  data.articles = [
    makeArticle(0, 0, "Saved compiler article", {starred: true}),
    makeArticle(1, 0, "Ordinary compiler article"),
    makeArticle(2, 1, "Tagged runtime article", {tags: ["Keep"]}),
    makeArticle(3, 1, "Noted runtime article", {note: "Remember this runtime detail."}),
    makeArticle(4, 1, "Ordinary runtime article"),
    makeArticle(5, 2, "Unaffected design article"),
  ];
  data.folders = [GROUP, "Reference", "Empty folder"];
  data.stream_settings = {};
  data.next_id = BASE + 200;
  Object.assign(data.settings, {density: "Comfortable", view_mode: "Expanded", unread_only: false,
    sort: "Newest", font_size: 14, mark_read_on_scroll: false, show_all_subscriptions: true,
    feed_connection: "Direct", feed_proxy: "", send_to: []});
  const xml = (index, extra = false) => {
    const articles = data.articles.filter(article => article.feed_id === BASE + index);
    if (extra) articles.push(makeArticle(6, index, "New article fetched after unsubscribe"));
    return `<?xml version="1.0"?><rss version="2.0"><channel><title>${sourceName(index)}</title>`
      + `<link>https://folder-fixture.example/</link><description>Subscription lifecycle fixture</description>`
      + articles.map(article => `<item><guid>${article.guid}</guid><title>${article.title}</title>`
        + `<link>${article.url}</link><description><![CDATA[${article.content}]]></description>`
        + `<pubDate>${new Date(article.published * 1000).toUTCString()}</pubDate></item>`).join("")
      + "</channel></rss>";
  };
  for (const index of [0, 1, 2]) await writeFile(path.join(site, files[index]), xml(index));
  return {data, xml,
    update: (index, extra) => writeFile(path.join(site, files[index]), xml(index, extra)),
    cleanup: () => Promise.all(files.map(file => rm(path.join(site, file), {force: true}))),
  };
}

export async function context(h, name = GROUP) {
  const target = await h.control(element => folder(element, name), `folder ${name}`);
  const [x, y] = center(target);
  await h.page.send("Input.dispatchMouseEvent", {type: "mouseMoved", x, y, pointerType: "mouse"});
  await h.page.send("Input.dispatchMouseEvent", {type: "mousePressed", x, y, button: "right", buttons: 2, clickCount: 1, pointerType: "mouse"});
  await h.page.send("Input.dispatchMouseEvent", {type: "mouseReleased", x, y, button: "right", buttons: 0, clickCount: 1, pointerType: "mouse"});
  return h.control(element => element.name === "ReaderMenu", `folder ${name} menu`);
}

export async function keyboardContext(h) {
  const key = {key: "F10", code: "F10", windowsVirtualKeyCode: 121, nativeVirtualKeyCode: 121, modifiers: 8};
  await h.page.send("Input.dispatchKeyEvent", {type: "rawKeyDown", ...key});
  await h.page.send("Input.dispatchKeyEvent", {type: "keyUp", ...key});
  return h.control(element => element.name === "ReaderMenu", "keyboard folder menu");
}

export async function dismiss(h) {
  await h.page.press("Escape");
  await h.until(async () => !(await h.snapshot()).elements.some(element => element.name === "ReaderMenu" || element.name === "ReaderDialog"),
    "folder popup dismisses");
}

export async function undo(h) {
  await h.until(async () => !(await h.snapshot()).elements.some(element => element.name === "ReaderDialog"), "confirmation closes before undo");
  // Search submission and programmatic navigation retain the native editor's
  // focus. Use plain stream-heading text to target application undo without
  // opening an article (which would add a read-state command to the history).
  const route = JSON.parse(await h.page.evaluate("readerApp.currentView()"));
  const title = typeof route === "string"
    ? ({All: "All items", Starred: "Starred items"}[route] ?? route)
    : route.Folder ?? route.Tag ?? (route.Search !== undefined
      ? `Search results for “${route.Search}”`
      : (await h.state()).feeds.find(feed => feed.id === route.Feed)?.title);
  assert(title, "the undo fixture has a current stream title");
  const heading = await h.control(element => /TextBlock$/.test(element.type)
    && element.text?.startsWith(title) && element.x > 200 && element.y > 90 && element.y < 180,
    "current stream heading before application undo");
  await h.page.click(heading.x + Math.min(5, heading.width / 2), heading.y + heading.height / 2);
  await h.until(async () => !(await h.snapshot()).elements.some(element => /TextBox$/.test(element.type) && element.focused),
    "the search editor releases focus before application undo");
  await h.page.press("z", ["ctrl"]);
}

export async function reveal(h, name) {
  for (let attempt = 0; attempt < 8; attempt++) {
    const view = await h.snapshot(), scroll = view.elements.find(element => element.name === "DialogScroll");
    const item = view.elements.find(element => element.name === name && element.hit);
    if (item && (!scroll || item.y >= scroll.y - 1 && item.y + item.height <= scroll.y + scroll.height + 1)) return item;
    assert(scroll, `a scrollable confirmation contains ${name}`);
    await h.page.wheel(...center(scroll), 0, scroll.height * 0.65);
    await sleep(50);
  }
  throw new Error(`The confirmation action ${name} is not reachable`);
}
