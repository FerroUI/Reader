import assert from "node:assert/strict";

const route = h => h.page.evaluate("JSON.parse(readerApp.currentView())");
const go = (h, view) => h.page.evaluate(`readerApp.navigate(${JSON.stringify(JSON.stringify(view))})`);
const replace = (h, data) => h.page.evaluate(`readerApp.importData(${JSON.stringify(JSON.stringify(data))})`);
const closed = h => h.until(async () => !(await h.snapshot()).elements.some(e => e.name === "ReaderDialog"), "editor dismissed");
const cursor = async h => (await h.snapshot()).elements.find(e => e.treeCursor && /Button$/.test(e.type));

export async function refinementChecks(h, check) {
  const baseline = await h.state();
  let passed = false;
  try {
    await check("home stories open their source and article tags navigate", async () => {
      const data = structuredClone(baseline);
      data.stream_settings = {};
      data.settings.unread_only = false;
      data.settings.view_mode = "Expanded";
      await replace(h, data);
      await h.clickNavigation("Home");
      const feed = data.feeds.find(feed => data.articles.some(a => a.feed_id === feed.id && !a.read));
      const article = data.articles.find(a => a.feed_id === feed.id && !a.read);
      await h.clickLabel(article.title);
      await h.until(async () => (await route(h)).Feed === feed.id, "home story opens the containing subscription");
      await h.until(async () => (await h.state()).articles.find(a => a.id === article.id).read, "home story selects its article");
      await h.page.press("t");
      await h.control(e => e.name === "ArticleTagsPopup", "inline tag editor");
      await h.edit("DialogInput", "Review tag", true);
      await closed(h);
      await h.clickLabel("Show items tagged Review tag");
      await h.until(async () => (await route(h)).Tag === "Review tag", "footer tag opens that tag stream");
      await h.capture("11-tags");
    });

    await check("empty unread recovery keeps its feed and search keeps its query", async () => {
      const data = structuredClone(baseline);
      data.stream_settings = {};
      data.settings.unread_only = false;
      data.settings.view_mode = "List";
      const feed = data.feeds[0];
      for (const article of data.articles) if (article.feed_id === feed.id) article.read = true;
      await replace(h, data);
      await go(h, { Feed: feed.id });
      await h.clickName("StreamFilter");
      await h.clickText("New items");
      const empty = await h.control(e => e.name === "EmptyStreamTitle", "empty unread message");
      assert.match(empty.text, /has no unread items/);
      await h.capture("11-empty-feed");
      await h.clickLabel("View all items");
      assert.deepEqual(await route(h), { Feed: feed.id });
      assert.equal((await h.state()).settings.unread_only, false);
      await h.until(async () => (await h.snapshot()).articleContainers > 0, "current feed items restored");
      await h.edit("SearchReader", "unmatchable-review-query", true);
      const search = await h.control(e => e.name === "EmptyStreamTitle", "empty search message");
      assert.match(search.text, /Your search - unmatchable-review-query - did not match any items/);
      assert(!(await h.snapshot()).elements.some(e => e.automationName === "View all items"), "search must not offer unrelated navigation");
      assert.deepEqual(await route(h), { Search: "unmatchable-review-query" });
      await h.capture("11-empty-search");
    });

    await check("desktop quick add anchors, cancels and survives a narrow resize", async () => {
      await h.clickNavigation("Home");
      const anchor = await h.control(e => e.name === "SubscribeButton", "Subscribe anchor");
      await h.clickName("SubscribeButton");
      const popup = await h.control(e => e.name === "ReaderDialog", "quick-add popup");
      assert(Math.abs(popup.x - anchor.x) < 1 && Math.abs(popup.y - anchor.y - anchor.height) < 1,
        "quick add must be attached to the actual Subscribe button");
      assert.equal(popup.width, 302);
      await h.edit("DialogInput", "https://example.com/unsaved-review.xml");
      await h.inputHints("url", "go");
      await h.capture("11-subscribe");
      await h.resize(390, 844);
      const input = await h.control(e => e.name === "DialogInput", "resized editor");
      assert.equal(input.text, "https://example.com/unsaved-review.xml");
      assert(input.focused, "changing popup to mobile dialog must retain native editing focus");
      await h.page.send("Input.insertText", { text: "#append" });
      await h.until(async () => (await h.control(e => e.name === "DialogInput", "rotated edit caret")).text === "https://example.com/unsaved-review.xml#append",
        "typing after a breakpoint must append at the preserved caret, not replace the draft");
      await h.clickLabel("Cancel");
      await closed(h);
      await h.resize(1363, 936);
      await h.clickName("SubscribeButton");
      await h.page.click(900, 400);
      await closed(h);
      assert.equal((await h.state()).feeds.length, baseline.feeds.length, "cancel and outside dismissal never subscribe");
    });

    await check("subscription cursor scans, collapses and opens without reading early", async () => {
      const data = structuredClone(baseline);
      // The network fixture is unfiled: use only the original grouped feeds so
      // the first visible node is a folder and its next node is a child feed.
      data.feeds = data.feeds.filter(feed => feed.folder || feed.folders?.length);
      data.articles = data.articles.filter(a => data.feeds.some(f => f.id === a.feed_id));
      data.settings.show_all_subscriptions = true;
      await replace(h, data);
      await h.clickNavigation("Home");
      const before = (await h.state()).articles;
      await h.page.press("p", ["shift"]);
      assert.equal(await cursor(h), undefined, "initial previous must not select the last subscription");
      await h.page.press("n", ["shift"]);
      const folder = await h.until(() => cursor(h), "first subscription cursor");
      await h.page.press("n", ["shift"]);
      const feed = await h.until(async () => {const current = await cursor(h);return current?.name !== folder.name && current;}, "next tree occurrence");
      assert.deepEqual(await route(h), "Home", "tree scan must not open a stream");
      assert.deepEqual((await h.state()).articles, before, "tree scan must not read articles");
      await h.page.press("x", ["shift"]);
      await h.until(async () => (await cursor(h))?.automationName === folder.automationName, "collapse from feed promotes cursor to its folder");
      await h.page.press("x", ["shift"]);
      await h.page.press("n", ["shift"]);
      await h.until(async () => (await cursor(h))?.automationName === feed.automationName, "expand restores child traversal");
      await h.page.press("o", ["shift"]);
      const target = data.feeds.find(f => feed.automationName === f.title || feed.automationName.startsWith(`${f.title} (`));
      assert(target, "cursor identifies a fixture subscription");
      await h.until(async () => (await route(h)).Feed === target.id, "open the exact selected subscription");
      assert.deepEqual((await h.state()).articles, before, "opening a stream does not open an article");
      await h.capture("11-tree-cursor");
    });

    await check("keyboard help keeps four groups and a reachable phone close action", async () => {
      await h.clickLabel("Settings");
      await h.clickText("Keyboard shortcuts");
      await h.control(e => e.name === "KeyboardHelp", "shortcut overlay");
      const view = await h.snapshot();
      for (const text of ["Navigation", "Application", "Jumping", "Acting on items"])
        assert(view.elements.some(e => e.text === text), `missing shortcut group: ${text}`);
      await h.capture("11-shortcuts");
      await h.resize(390, 844);
      const close = await h.control(e => /Button$/.test(e.type) && e.automationName === "Close keyboard shortcuts", "phone shortcut close action");
      assert(close.x >= 0 && close.y >= 0 && close.x + close.width <= 390 && close.y + close.height <= 844);
      assert(close.width >= 44 && close.height >= 44);
      await h.capture("11-mobile-shortcuts");
      await h.clickLabel("Close keyboard shortcuts");
      await h.resize(1363, 936);
    });

    await check("dated mark-all confirmation preserves cutoff and opt-out", async () => {
      const data = structuredClone(baseline);
      const now = Math.floor(Date.now() / 1000);
      const template = data.articles[0];
      data.articles = Array.from({length: 60}, (_, index) => ({...template,
        id: 70000 + index, guid: `confirmation-${index}`, title: `Confirmation item ${index}`,
        feed_id: data.feeds[0].id, published: now - (index < 30 ? 172800 : 3600),
        read: false, read_at: null, starred: false, tags: [], note: ""}));
      data.next_id = 70100;
      data.settings.confirm_mark_all = true;
      data.settings.view_mode = "List";
      data.settings.unread_only = false;
      data.stream_settings = {};
      await replace(h, data);
      await h.clickNavigation("All items");
      await h.page.press("2");
      await h.clickLabel("Mark all read options");
      await h.clickText("Items older than a day");
      await h.control(e => e.name === "ReaderDialog", "dated mark-all confirmation");
      assert((await h.snapshot()).elements.some(e => e.text?.includes("published before")), "age condition must remain visible");
      await h.capture("11-mark-confirmation");
      await h.click(e => /CheckBox$/.test(e.type) && e.automationName === "Don't ask me again", "persistent confirmation opt-out");
      await h.clickLabel("Mark all as read");
      await closed(h);
      const marked = await h.state();
      assert.equal(marked.articles.filter(a => a.read).length, 30, "confirmation must apply only the chosen age range");
      assert.equal(marked.settings.confirm_mark_all, false);
      await h.page.press("a", ["shift"]);
      await h.until(async () => (await h.state()).articles.every(a => a.read), "opt-out applies to keyboard mark-all");
      assert(!(await h.snapshot()).elements.some(e => e.name === "ReaderDialog"));
    });
    passed = true;
  } finally {
    if (passed) {
      await h.page.press("Escape");
      await h.resize(1363, 936);
      await replace(h, baseline);
      await h.clickNavigation("All items");
    }
  }
}
