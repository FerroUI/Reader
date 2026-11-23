import assert from "node:assert/strict";
import { sleep } from "../../FerroUI/scripts/browser/harness.mjs";

const BASE = 590000;
const replace = (h, data) => h.page.evaluate(`readerApp.importData(${JSON.stringify(JSON.stringify(data))})`);
const item = (view, name, id) => view.elements.find(e => e.name === name && (id == null || e.articleId === id));
const paragraph = (view, prefix) => view.elements.find(e => e.name === "ArticleBodyText" && e.text?.startsWith(prefix));
const identity = e => ({...e.richText, links: e.links.map(link => link.id)});
const near = (actual, expected, label, tolerance = 0.05) => assert(Math.abs(actual - expected) <= tolerance,
  `${label}: expected ${expected}, received ${actual}`);
const within = (e, parent) => e.x >= parent.x - 1 && e.y >= parent.y - 1
  && e.x + e.width <= parent.x + parent.width + 1 && e.y + e.height <= parent.y + parent.height + 1;
const child = (view, name, parent) => view.elements.find(e => e.name === name && within(e, parent));

// The generic single-character helper omits physical metadata for '='. Reader
// resolves this shortcut from Equal, while Shift+Equal produces the '+' key.
async function pressEqual(h, shift = false) {
  const key = shift ? "+" : "=";
  const base = {key, code: "Equal", windowsVirtualKeyCode: 187, nativeVirtualKeyCode: 187,
    modifiers: shift ? 8 : 0};
  await h.page.send("Input.dispatchKeyEvent", {type: "keyDown", ...base, text: key});
  await h.page.send("Input.dispatchKeyEvent", {type: "keyUp", ...base});
  await sleep(80);
}

function fixture(baseline, count = 2) {
  const data = structuredClone(baseline), now = Math.floor(Date.now() / 1000);
  data.feeds = [{id: BASE - 1, title: "Magnification fixture", url: "https://scale.example/feed.xml",
    site_url: "https://scale.example/", fetched_at: now}];
  data.folders = []; data.stream_settings = {}; data.next_id = BASE + count;
  data.settings = {...data.settings, density: "Comfortable", view_mode: "Expanded", unread_only: false,
    sort: "Newest", font_size: 16, mark_read_on_scroll: false, send_to: []};
  data.articles = Array.from({length: count}, (_, index) => ({
    id: BASE + index, feed_id: BASE - 1, guid: `scale-${index}`, title: `Magnification article ${index}`,
    url: `https://scale.example/articles/${index}`, author: "Reader fixture", published: now - index,
    read: false, starred: false, tags: [], note: "", read_at: null, summary: `Article ${index} snippet`,
    content: index === 0
      ? '<p>Magnification selection survives native font changes and its <a href="https://scale.example/reference">inline link</a>.</p>'
        + '<table><tr><th>Magnification table heading</th><td>Magnification table cell</td></tr></table>'
      : `<p>Inactive paragraph ${index} keeps the saved base font.</p>`,
  }));
  return data;
}

async function focusStream(h) {
  await h.click(e => /TextBlock$/.test(e.type) && e.text?.startsWith("All items (")
    && e.y > 40 && e.y < 180, "magnification fixture stream title");
}

async function resetScale(h) {
  await focusStream(h);
  for (let i = 0; i < 6; i++) await h.page.press("-");
}

async function prepare(h, data, mode = "Expanded") {
  await replace(h, data);
  await h.page.evaluate('readerApp.navigate("\\"All\\"")');
  await h.edit("SearchReader", "");
  await focusStream(h);
  await h.page.press(mode === "List" ? "2" : "1");
  await resetScale(h);
  return h.until(async () => {
    const view = await h.snapshot();
    return item(view, mode === "List" ? "ArticleRow" : "ArticleEntry", BASE) && view;
  }, `${mode} magnification fixture is realized`);
}

async function settled(h) {
  let previous, stable = 0;
  return h.until(async () => {
    const view = await h.snapshot(), offset = view.articleScroll?.offsetY;
    stable = Number.isFinite(offset) && Math.abs(offset - previous) < 0.2 ? stable + 1 : 0;
    previous = offset;
    return stable >= 3 && view;
  }, "magnification reflow and native scroll anchoring settle");
}

async function selectText(h, block) {
  const y = block.y + Math.min(12, block.height / 2);
  await h.page.mouseDown(block.x + 3, y);
  for (const distance of [35, 75, 135]) await h.page.send("Input.dispatchMouseEvent", {
    type: "mouseMoved", pointerType: "mouse", x: block.x + distance, y, buttons: 1,
  });
  await h.page.mouseUp(block.x + 135, y);
  return h.until(async () => {
    const selection = paragraph(await h.snapshot(), "Magnification selection")?.textSelection;
    return selection?.start !== selection?.end && selection?.text?.length > 8 && selection;
  }, "a native text range is selected before magnification");
}

export async function readingScaleChecks(h, check) {
  const baseline = await h.state(), initial = await h.snapshot();
  const route = await h.page.evaluate("readerApp.currentView()");
  let sidebarHidden = false, focusMode = false;
  const evidence = [];
  try {
    await h.resize(1363, 936);
    await check("current-article magnification preserves the base preference and native text selection", async () => {
      const data = fixture(baseline);
      await prepare(h, data);
      await pressEqual(h);
      let view = await settled(h);
      near(paragraph(view, "Magnification selection").fontSize, 16,
        "changing magnification without a current item leaves all bodies at base size");
      await h.page.press("n");
      await h.until(async () => {
        const view = await h.snapshot();
        return Math.abs(paragraph(view, "Magnification selection")?.fontSize - 17.6) < 0.05;
      }, "the current scan row inherits the session's 110% magnification");
      await h.page.press("-");
      view = await settled(h);
      const selection = await selectText(h, paragraph(view, "Magnification selection"));
      view = await h.snapshot();
      const originalBlock = paragraph(view, "Magnification selection");
      const original = identity(originalBlock);
      const saved = await h.state();
      for (const percent of [110, 120, 130, 140, 150, 150]) {
        await pressEqual(h, percent === 120);
        view = await h.until(async () => {
          const view = await h.snapshot(), block = paragraph(view, "Magnification selection");
          return block && Math.abs(block.fontSize - 16 * percent / 100) < 0.05 && view;
        }, `current article reaches ${percent}% without recreating its native document`);
        const block = paragraph(view, "Magnification selection");
        assert.deepEqual(identity(block), original, "native control, paragraph and hyperlinks remain mounted");
        assert.deepEqual(block.textSelection, selection, "the exact native selection survives font reflow");
        near(paragraph(view, "Inactive paragraph 1").fontSize, 16, "other expanded articles retain the saved base size");
        for (const prefix of ["Magnification table heading", "Magnification table cell"])
          near(paragraph(view, prefix).fontSize, 16 * percent / 100, "table text follows current-entry magnification");
        evidence.push({percent, fontSize: block.fontSize, height: block.height, identity: identity(block), selection: block.textSelection});
      }
      assert(paragraph(view, "Magnification selection").height > originalBlock.height,
        "the native paragraph's shaped line geometry grows with magnification");
      assert(paragraph(view, "Magnification selection").links[0].bounds[0].height > originalBlock.links[0].bounds[0].height * 1.3,
        "native hyperlink hit bounds follow the enlarged glyph layout");
      assert.deepEqual((await h.state()).settings, saved.settings, "magnification does not rewrite persistent settings");
      assert.deepEqual((await h.state()).articles, saved.articles, "magnification does not mark another article read");
      await h.capture("17-reading-magnification");
      await h.page.press("n");
      view = await settled(h);
      near(paragraph(view, "Magnification selection").fontSize, 16, "the previous current article returns to its base size");
      near(paragraph(view, "Inactive paragraph 1").fontSize, 24, "the 150% session level follows the new current article");
      assert.deepEqual(identity(paragraph(view, "Magnification selection")), original);
      assert.deepEqual(paragraph(view, "Magnification selection").textSelection, selection);
      await h.clickName("SearchReader");
      await pressEqual(h); await h.page.press("-");
      view = await h.snapshot();
      near(paragraph(view, "Inactive paragraph 1").fontSize, 24, "typing punctuation in Search does not run magnification commands");
      assert.equal(item(view, "SearchReader").text, "=-");
      await h.edit("SearchReader", "");
      await focusStream(h);
      for (let i = 0; i < 6; i++) await h.page.press("-");
      view = await settled(h);
      near(paragraph(view, "Inactive paragraph 1").fontSize, 16, "magnification stops at 100%");
    });

    await check("list magnification retains density, source/date layout and current-only expansion", async () => {
      const data = fixture(baseline);
      await prepare(h, data, "List");
      await h.page.press("n");
      for (let i = 0; i < 5; i++) await pressEqual(h);
      let view = await settled(h), row = item(view, "ArticleRow", BASE);
      near(row.height, 37, "magnification leaves the original fixed headline height intact");
      for (const name of ["ArticleHeadlineText", "ArticleHeadlineSource", "ArticleHeadlineDate"])
        near(child(view, name, row).fontSize, 19.5, `${name} inherits the current 150% font`);
      near(child(view, "ArticleHeadlineText", item(view, "ArticleRow", BASE + 1)).fontSize, 13,
        "an inactive headline retains its ordinary font");
      await h.page.press("o");
      view = await h.until(async () => {
        const next = await h.snapshot();
        return paragraph(next, "Magnification selection") && next;
      }, "the magnified headline opens inline");
      near(paragraph(view, "Magnification selection").fontSize, 24, "the opened body inherits 150% magnification");
      near(item(view, "ArticleRow", BASE).height, 36, "expanded list headline reserves its original one-pixel separator");
      await h.page.press("n");
      view = await settled(h);
      near(paragraph(view, "Magnification selection").fontSize, 16, "the formerly current expanded body returns to base size");
      near(child(view, "ArticleHeadlineText", item(view, "ArticleRow", BASE + 1)).fontSize, 19.5,
        "scanning transfers magnification to the next collapsed headline");
      assert.equal((await h.state()).articles.find(a => a.id === BASE + 1).read, false,
        "scanning a magnified headline does not read it");
      await h.capture("17-list-magnification");
    });

    await check("hiding the sidebar widens article bodies while titles retain their original cap", async () => {
      const data = fixture(baseline);
      data.articles[0].title = "A long article title keeps its original readable heading width. ".repeat(5);
      data.articles[0].content = `<p>Wide reading body. ${"Native paragraphs use the space released by hiding subscriptions. ".repeat(24)}</p>`;
      let view = await prepare(h, data);
      view = await settled(h);
      const normal = paragraph(view, "Wide reading body"), original = identity(normal);
      const normalBody = child(view, "ArticleBody", item(view, "ArticleEntry", BASE));
      assert(normalBody, "the wide reading fixture has a native article body");
      assert.equal(normalBody.maxWidth, 650, "the sidebar-visible native body retains the exact 650px cap");
      assert(normalBody.width > 0 && normalBody.width <= 650 && within(normal, normalBody),
        "naturally measured text stays within the configured body cap");
      await h.page.press("u"); sidebarHidden = true;
      view = await h.until(async () => {
        const view = await h.snapshot();
        const body = child(view, "ArticleBody", item(view, "ArticleEntry", BASE));
        return body?.maxWidth === null && body.width > 1000
          && paragraph(view, "Wide reading body")?.width > 1000 && view;
      }, "hidden subscriptions release the full article body width");
      assert.deepEqual(identity(paragraph(view, "Wide reading body")), original);
      const title = view.elements.find(e => /TextBlock$/.test(e.type) && e.text === data.articles[0].title);
      assert(title && title.width <= 650, "the wider-body rule does not stretch the article title");
      assert(paragraph(view, "Wide reading body").height < normal.height, "text reflows into the newly available width");
      await h.capture("17-wide-reading-body");
      await h.page.press("u"); sidebarHidden = false;
      await h.until(async () => {
        const view = await h.snapshot(), body = child(view, "ArticleBody", item(view, "ArticleEntry", BASE));
        const text = paragraph(view, "Wide reading body");
        return body?.maxWidth === 650 && Math.abs(body.width - normalBody.width) < 1
          && text && Math.abs(text.width - normal.width) < 1;
      }, "restoring subscriptions restores the exact cap and the original measured text width");
      await h.page.press("f"); focusMode = true;
      await h.until(async () => {
        const view = await h.snapshot(), body = child(view, "ArticleBody", item(view, "ArticleEntry", BASE));
        return body?.maxWidth === null && body.width > 1000
          && paragraph(view, "Wide reading body")?.width > 1000;
      }, "focus mode uses the same wider-body rule");
      await h.page.press("f"); focusMode = false;
    });

    await check("magnification and sidebar reflow retain a paragraph inside a far nonuniform article", async () => {
      const data = fixture(baseline, 320), index = 151;
      data.articles[index].content = Array.from({length: 28}, (_, block) => `<p>Scale anchor ${block}. `
        + "The visible paragraph remains anchored while native text reflows. ".repeat(8) + "</p>").join("");
      await prepare(h, data);
      await h.page.evaluate(`readerApp.scrollToArticle(${BASE + index})`);
      const block = await h.control(e => e.name === "ArticleBodyText" && e.text?.startsWith("Scale anchor 0."), "far article body");
      await h.page.click(block.x + 4, block.y + 12);
      await h.page.wheel(block.x + 100, 500, 0, 420);
      const before = await settled(h), list = item(before, "ArticleList");
      const anchor = before.elements.filter(e => e.name === "ArticleBodyText" && e.text?.startsWith("Scale anchor ")
        && e.y < list.y + list.height && e.y + e.height > list.y)
        .sort((a, b) => Math.abs(a.y - list.y) - Math.abs(b.y - list.y))[0];
      assert(anchor, "a concrete paragraph is visible inside the far article");
      const relativeY = anchor.y - list.y, original = identity(anchor), saved = await h.state();
      const anchored = async label => {
        const view = await settled(h), block = paragraph(view, anchor.text);
        assert(block, `${label} retains the same paragraph`);
        near(block.y - item(view, "ArticleList").y, relativeY, label, 3);
        assert.deepEqual(identity(block), original, `${label} retains its native document`);
        assert(view.articleContainers > 0 && view.articleContainers < 80, "reflow keeps realization bounded across 320 articles");
        assert(Number.isFinite(view.articleScroll.offsetY) && Number.isFinite(view.articleScroll.extentHeight));
        assert.deepEqual((await h.state()).articles, saved.articles, "layout reflow does not alter read state");
        return view;
      };
      for (let i = 0; i < 3; i++) await pressEqual(h);
      await anchored("magnification preserves the reading paragraph's viewport position");
      await h.page.press("u"); sidebarHidden = true;
      await anchored("sidebar hiding preserves the reading paragraph's viewport position");
      await h.page.press("u"); sidebarHidden = false;
      await anchored("sidebar restoration preserves the reading paragraph's viewport position");
      await h.capture("17-magnification-scroll-anchor");
    });
    await h.record("17-reading-magnification-evidence", evidence);
  } catch (error) {
    await h.capture("17-reading-scale-failure").catch(() => {});
    throw error;
  } finally {
    await focusStream(h).catch(() => {});
    if (focusMode) await h.page.press("f");
    if (sidebarHidden) await h.page.press("u");
    await resetScale(h);
    await replace(h, baseline);
    await h.page.evaluate(`readerApp.navigate(${JSON.stringify(route)})`);
    await h.resize(initial.width, initial.height);
    await h.page.evaluate("readerApp.flushSave(); import('./host.js').then(module => module.flush())");
  }
}
