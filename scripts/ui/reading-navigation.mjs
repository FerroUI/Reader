import assert from "node:assert/strict";
import http from "node:http";

const BASE = 520000, EARLIER = BASE, ALPHA = BASE + 1, BETA = BASE + 2, GAMMA = BASE + 3;
const FIRST = BASE + 10, TALL = BASE + 11, LAST = BASE + 12, FINAL = BASE + 14;
const route = h => h.page.evaluate("JSON.parse(readerApp.currentView())");
const go = (h, view) => h.page.evaluate(`readerApp.navigate(${JSON.stringify(JSON.stringify(view))})`);
const replace = (h, data) => h.page.evaluate(`readerApp.importData(${JSON.stringify(JSON.stringify(data))})`);
const article = (view, id) => view.elements.find(e => e.name === "ArticleEntry" && e.articleId === id);
const read = async (h, id) => (await h.state()).articles.find(a => a.id === id)?.read;

async function settled(h) {
  let previous, stable = 0;
  return h.until(async () => {
    const view = await h.snapshot(), offset = view.articleScroll?.offsetY;
    stable = Number.isFinite(offset) && Math.abs(offset - previous) < 0.2 ? stable + 1 : 0;
    previous = offset;
    return stable >= 2 && view;
  }, "Space navigation layout settles");
}

async function focusStream(h) {
  // Focus the application through non-interactive header text. Space on a
  // focused native Button correctly activates that button instead of Reader.
  const title = await h.control(e => /TextBlock$/.test(e.type)
    && e.text?.startsWith("Paging alpha") && e.x > 200 && e.y < 180, "stream title");
  await h.page.click(title.x + 5, title.y + title.height / 2);
}

async function prepareStream(h, data) {
  await replace(h, data);
  await go(h, {Feed: ALPHA});
  await focusStream(h);
  // Empty per-stream preferences intentionally restore the default Expanded
  // mode on import/navigation. Select the fixture's mode on its actual stream.
  const mode = data.settings.view_mode;
  await h.page.press(mode === "List" ? "2" : "1");
  await h.until(async () => {
    if ((await h.state()).settings.view_mode !== mode) return false;
    return (await h.snapshot()).elements.some(e => e.articleId === FIRST
      && e.name === (mode === "List" ? "ArticleRow" : "ArticleEntry"));
  }, `${mode} fixture is rendered in its selected stream`);
}

async function fixtureServer() {
  const server = http.createServer((_, response) => {
    response.writeHead(200, {"content-type": "text/html; charset=utf-8", "cache-control": "no-store"});
    response.end("<!doctype html><title>Reader keyboard link</title><p>Native link activation.</p>");
  });
  await new Promise((resolve, reject) => {
    server.once("error", reject);
    server.listen(0, "127.0.0.1", resolve);
  });
  return {origin: `http://127.0.0.1:${server.address().port}`, async close() {
    server.closeAllConnections();
    await new Promise(resolve => server.close(resolve));
  }};
}

function fixture(baseline, mode, origin) {
  const data = structuredClone(baseline), now = Math.floor(Date.now() / 1000);
  data.feeds = ["earlier", "alpha", "beta", "gamma"].map((name, index) => ({
    id: BASE + index, title: `Paging ${name}`, url: `${origin}/${name}.xml`, site_url: origin,
    folder: "", folders: [], description: "Keyboard paging fixture", fetched_at: now, error: null, sample: false,
  }));
  const long = `<p>Before the <a href="${origin}/reference">native reference</a> after the link.</p>`
    + `<pre>${Array.from({length: 8}, (_, index) => `line_${index}: ${"wide code sample ".repeat(20)}`).join("\n")}</pre>`
    + Array.from({length: 12}, (_, index) => `<p>Long paragraph ${index}. `
      + "Space preserves fifty pixels of reading context while advancing through a nonuniform article. ".repeat(5) + "</p>").join("");
  data.articles = [
    [BASE + 9, EARLIER, "Earlier unread item", "<p>Earlier unread subscription must never be wrapped to.</p>", false],
    [FIRST, ALPHA, "Short first article", "<p>A short first article fits in one viewport.</p>", false],
    [TALL, ALPHA, "Tall mixed article", long, false],
    [LAST, ALPHA, "Short final article", "<p>The last article advances to the next unread subscription.</p>", false],
    [BASE + 13, BETA, "Already read item", "<p>A read subscription is skipped.</p>", true],
    [FINAL, GAMMA, "Final unread item", "<p>The final unread subscription does not wrap.</p>", false],
  ].map(([id, feed_id, title, content, read], index) => ({
    id, feed_id, title, content, read, guid: `reading-page-${id}`, url: `${origin}/articles/${id}`,
    author: "Reader keyboard fixture", summary: title, published: now - index,
    starred: false, tags: [], note: "", read_at: read ? now : null,
  }));
  data.folders = []; data.next_id = BASE + 100; data.stream_settings = {};
  data.settings.view_mode = mode; data.settings.sort = "Newest"; data.settings.unread_only = false;
  data.settings.mark_read_on_scroll = false; data.settings.show_all_subscriptions = false;
  data.settings.density = "Comfortable"; data.settings.font_size = 14;
  return data;
}

async function focusedLinkSpace(h, url) {
  const paragraph = await h.control(e => e.name === "ArticleBodyText"
    && e.text?.startsWith("Before the native reference"), "native inline link paragraph");
  const bounds = paragraph.links?.find(link => link.url === url)?.bounds[0];
  assert(bounds, "the inline reference exposes native shaped-link bounds");
  const existing = new Set((await h.page.browserSend("Target.getTargets")).targetInfos.map(t => t.targetId));
  const before = await settled(h);
  try {
    await h.page.click(paragraph.x + 3, bounds.y + bounds.height / 2);
    await h.page.press("Tab");
    await h.page.press("Space");
    await h.until(async () => (await h.page.browserSend("Target.getTargets")).targetInfos
      .find(t => !existing.has(t.targetId) && t.type === "page" && t.url === url), "Space activates the focused native inline link");
    assert(Math.abs((await h.snapshot()).articleScroll.offsetY - before.articleScroll.offsetY) < 1,
      "native hyperlink activation must not also page the article stream");
    assert.equal(await read(h, LAST), false, "link activation does not select the next article");
  } finally {
    for (const target of (await h.page.browserSend("Target.getTargets")).targetInfos) {
      if (target.type === "page" && !existing.has(target.targetId))
        await h.page.browserSend("Target.closeTarget", {targetId: target.targetId});
    }
    await h.page.send("Page.bringToFront");
  }
  await focusStream(h);
}

async function streamPaging(h, mode, data, origin) {
  await prepareStream(h, data);
  const initial = await settled(h);
  await h.page.press("Space", ["shift"]);
  assert.equal(await read(h, FIRST), false, "initial Shift+Space must not open the last item");
  assert.equal(await read(h, LAST), false);
  assert.equal((await settled(h)).articleScroll.offsetY, initial.articleScroll.offsetY);
  await h.page.press("Space");
  await h.until(() => read(h, FIRST), "Space opens the first article");
  const first = await settled(h);
  assert(article(first, FIRST)?.height < first.articleScroll.viewportHeight, "the first item fits in the viewport");
  await h.page.press("Space");
  await h.until(() => read(h, TALL), "Space opens the next article when the short item is fully visible");
  let top = await settled(h);
  assert(article(top, TALL)?.height > top.articleScroll.viewportHeight * 2, "the next item spans several pages");
  if (mode === "Expanded") {
    await focusedLinkSpace(h, `${origin}/reference`);
    top = await settled(h);
  }
  await h.page.press("Space");
  const paged = await settled(h), distance = top.articleScroll.viewportHeight - 50;
  assert(Math.abs(paged.articleScroll.offsetY - top.articleScroll.offsetY - distance) < 2,
    "Space pages the primary article viewport with the original fifty-pixel overlap");
  assert.equal(await read(h, LAST), false, "paging inside a tall item must not skip to its neighbor");
  await h.page.press("Space", ["shift"]);
  const reversed = await settled(h);
  assert(Math.abs(reversed.articleScroll.offsetY - top.articleScroll.offsetY) < 2,
    "Shift+Space reverses the same page distance within the current item");
  await h.page.press("Space", ["shift"]);
  await h.until(async () => {
    const view = await h.snapshot(), first = article(view, FIRST);
    const list = view.elements.find(e => e.name === "ArticleList");
    return first && first.y >= list.y - 1 && first.y < list.y + 30;
  }, "Shift+Space opens the preceding article once the current top is visible");
  await h.page.press("Space");
  await settled(h);
  let pages = 0;
  while (!(await read(h, LAST)) && pages++ < 20) {
    assert.deepEqual(await route(h), {Feed: ALPHA}, "page navigation stays in the current stream until its final item");
    await h.page.press("Space");
    await settled(h);
  }
  assert(pages > 1 && pages < 20, "long content is read page by page before advancing");
  await h.page.press("Space");
  await h.until(async () => (await route(h)).Feed === GAMMA, "stream completion skips read subscriptions and opens the next unread one");
  assert.equal(await read(h, FINAL), false, "opening the next subscription does not mark its first item read");
  await h.page.press("Space");
  await h.until(() => read(h, FINAL), "Space opens the new subscription's first item");
  await settled(h);
  await h.page.press("Space");
  assert.deepEqual(await route(h), {Feed: GAMMA}, "the last subscription does not wrap to an earlier unread occurrence");
  assert.equal(await read(h, BASE + 9), false);
  await h.capture(`16-space-${mode.toLowerCase()}`);
}

export async function readingNavigationChecks(h, check) {
  const baseline = await h.state(), server = await fixtureServer();
  let passed = false;
  try {
    await h.resize(1363, 936);
    for (const mode of ["List", "Expanded"]) {
      await check(`${mode} Space navigation pages tall articles and advances unread subscriptions`,
        () => streamPaging(h, mode, fixture(baseline, mode, server.origin), server.origin));
    }
    await check("Space pages the overview without selecting an article", async () => {
      await replace(h, fixture(baseline, "Expanded", server.origin));
      await go(h, "Home");
      await h.resize(1363, 500);
      await h.clickText("A look at what's new in Reader");
      const before = await h.until(async () => {
        const scroll = (await h.snapshot()).elements.find(e => e.name === "ReaderPageScroll")?.scroll;
        return scroll?.extentHeight > scroll?.viewportHeight && scroll;
      }, "overview has a finite scrollable viewport");
      const articles = (await h.state()).articles;
      await h.page.press("Space");
      const expected = Math.min(before.viewportHeight - 50, before.extentHeight - before.viewportHeight);
      await h.until(async () => {
        const scroll = (await h.snapshot()).elements.find(e => e.name === "ReaderPageScroll")?.scroll;
        return scroll && Math.abs(scroll.offsetY - expected) < 1;
      }, "overview pages with the same fifty-pixel overlap");
      await h.page.press("Space", ["shift"]);
      await h.until(async () => (await h.snapshot()).elements.find(e => e.name === "ReaderPageScroll")?.scroll?.offsetY === 0,
        "Shift+Space returns to the overview top");
      assert.deepEqual((await h.state()).articles, articles, "overview paging never opens or marks an article read");
      await h.resize(1363, 936);
    });
    await check("Space remains native input in search and subscription editors", async () => {
      await prepareStream(h, fixture(baseline, "List", server.origin));
      const before = (await h.state()).articles;
      await h.edit("SearchReader", "paging");
      await h.page.press("Space");
      assert.equal((await h.control(e => e.name === "SearchReader", "search text")).text, "paging ");
      assert.deepEqual((await h.state()).articles, before);
      await h.clickName("SubscribeButton");
      await h.edit("DialogInput", "https://example.com/feed");
      await h.page.press("Space");
      assert.equal((await h.control(e => e.name === "DialogInput", "subscription text")).text, "https://example.com/feed ");
      assert.deepEqual((await h.state()).articles, before);
      await h.page.press("Escape");
      await h.until(async () => !(await h.snapshot()).elements.some(e => e.name === "ReaderDialog"), "subscription editor dismissed");
    });
    passed = true;
  } finally {
    await server.close();
    if (passed) { await replace(h, baseline); await h.clickNavigation("All items"); }
  }
}
