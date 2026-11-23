import assert from "node:assert/strict";
import { createTouch } from "./touch.mjs";
import { interceptSendToPopup } from "./send-to-popup.mjs";

const FEED = 620000, ITEM = FEED + 1;
const template = "https://notebook.invalid/save?url=${url}&title=${title}&source=${source}";
const replace = (h, data) => h.page.evaluate(`readerApp.importData(${JSON.stringify(JSON.stringify(data))})`);
const go = h => h.page.evaluate(`readerApp.navigate(${JSON.stringify(JSON.stringify({Feed: FEED}))})`);
const targets = async h => (await h.page.browserSend("Target.getTargets")).targetInfos.filter(t => t.type === "page").map(t => t.targetId).sort();
const menuItem = (name, e) => /MenuItem$/.test(e.type) && e.automationName === name;
const menu = (h, name) => h.control(e => menuItem(name, e), `Send to destination: ${name}`);
const clickMenu = (h, name) => h.click(e => menuItem(name, e), name);
const destinations = async h => (await h.state()).settings.send_to;
const checkbox = h => h.control(e => e.name === "ArticleReadState", "article read-state checkbox");
const read = async h => (await h.state()).articles.find(a => a.id === ITEM).read;

async function focusStream(h) {
  // Programmatic navigation after reload does not focus the native canvas.
  // Plain stream text establishes the browser input target without opening an item.
  await h.click(e => /TextBlock$/.test(e.type) && e.text?.startsWith("Renamed & Unicode source")
    && e.x > 200 && e.y < 180, "Send to stream title");
}

async function scanItem(h) {
  await focusStream(h);
  await h.page.press("n");
  await h.until(async () => {
    const elements = (await h.snapshot()).elements;
    return elements.some(e => e.name === "ArticleList" && e.focused)
      && elements.some(e => e.name === "ArticleHeadlineEdge" && e.border?.brush === "#4D90F0")
      && !elements.some(e => e.name === "ArticleEntry");
  }, "N focuses the native article list and scans the collapsed item");
}

function fixture(baseline, {mode = "List", automatic = false, configured = false, isRead = false} = {}) {
  const data = structuredClone(baseline), now = Math.floor(Date.now() / 1000);
  data.feeds = [{id: FEED, title: "Renamed & Unicode source", url: "https://news.invalid/feed.xml",
    site_url: "https://news.invalid/", folder: "", folders: [], description: "Send to fixture",
    fetched_at: now, error: null, sample: false}];
  data.articles = [{id: ITEM, feed_id: FEED, guid: "send-to-item", title: "Rust & 日本語 / ${source}",
    url: "https://news.invalid/article?q=1&next=two#section", content: "<p>A short article for native Reader actions.</p>",
    summary: "Send to fixture", author: "", published: now, read: isRead, starred: false,
    tags: [], note: "", read_at: isRead ? now : null}];
  data.folders = []; data.next_id = ITEM + 1; data.stream_settings = {};
  Object.assign(data.settings, {view_mode: mode, sort: "Newest", unread_only: false,
    mark_read_on_scroll: automatic, density: "Comfortable", font_size: 14,
    send_to: configured ? [{id: 1, name: "Reading notebook", url_template: template, enabled: true}] : []});
  return data;
}

async function dismiss(h) {
  await h.page.press("Escape");
  await h.until(async () => !(await h.page.evaluate("readerApp.hasOverlay()")), "Send to overlay closes");
}

export async function sendToChecks(h, check) {
  const baseline = await h.state(), existing = await targets(h);
  let touch, passed = false;
  try {
    await h.resize(1363, 936);
    await replace(h, fixture(baseline));
    await go(h);
    await focusStream(h);
    await h.page.press("2");
    await h.until(async () => {
      const elements = (await h.snapshot()).elements;
      return elements.some(e => e.name === "ArticleRow") && !elements.some(e => e.name === "ArticleEntry");
    }, "Send to fixture starts in native collapsed List view");
    await check("Send to configuration validates templates without opening or reading items", async () => {
      await scanItem(h);
      await h.page.press("t", ["shift"]);
      await h.clickLabel("Create a custom link");
      await h.edit("SendToName", "Reading notebook");
      await h.edit("SendToTemplate", "javascript:alert('${url}')");
      await h.clickLabel("Save destination");
      await h.control(e => e.name === "SendToError", "unsafe destination error");
      assert.deepEqual(await destinations(h), []);
      await h.edit("SendToTemplate", "https://notebook.invalid/?u=${unsupported}");
      await h.clickLabel("Save destination");
      await h.control(e => e.name === "SendToError" && e.text.includes("Unsupported placeholder"), "unknown placeholder error");
      await h.edit("SendToTemplate", template);
      await h.inputHints("url", "done");
      await h.page.press("Enter");
      await h.until(async () => (await destinations(h))[0]?.name === "Reading notebook", "custom destination saved");
      assert.equal((await destinations(h))[0].url_template, template);
      assert.equal(await read(h), false);
      assert.deepEqual(await targets(h), existing, "configuration must not launch the destination or OS Share");
      await h.capture("17-send-to-settings");
    });

    await check("native Send to opens the exact encoded destination without contacting a service", async () => {
      await dismiss(h);
      const before = await h.state(), article = before.articles[0], source = before.feeds[0].title;
      const expected = new URL(template.replace(/\$\{(url|title|source)\}/g,
        (_, token) => encodeURIComponent({url: article.url, title: article.title, source}[token]))).href;
      const interception = await interceptSendToPopup(h, expected);
      try {
        await h.page.press("t", ["shift"]);
        await clickMenu(h, "Reading notebook");
        await interception.opened();
        assert.deepEqual((await h.state()).articles, before.articles, "sending a scanned item does not mark it read or select it");
        assert(expected.includes("%24%7Bsource%7D"), "article title placeholder text must only be encoded once");
        await h.record("17-send-to-activation", {expected, source, title: article.title});
      } finally { await interception.close(); }
      await h.until(() => h.page.evaluate("!readerApp.hasOverlay()"), "destination activation closes its menu");
      await h.page.press("t", ["shift"]);
      await clickMenu(h, "Customize Send to");
    });

    await check("Send to supports edit, cancel, disable, reload and both native menu anchors", async () => {
      await h.clickLabel("Edit Reading notebook");
      await h.edit("SendToName", "Unsaved draft");
      await h.clickLabel("Cancel");
      assert.equal((await destinations(h))[0].name, "Reading notebook");
      await h.click(e => e.automationName === "Enable Reading notebook", "disable destination");
      await h.until(async () => !(await destinations(h))[0].enabled, "destination disabled");
      await dismiss(h);
      await h.page.press("Enter");
      await h.until(async () => (await h.snapshot()).elements.some(e => e.name === "ArticleEntry"), "item expands");
      assert(!(await h.snapshot()).elements.some(e => e.name === "ArticleSendTo"), "no Send to footer is shown without enabled services");
      await h.page.press("t", ["shift"]);
      await h.click(e => e.automationName === "Enable Reading notebook", "enable destination");
      await h.until(async () => (await destinations(h))[0].enabled, "destination enabled");
      await h.clickLabel("Edit Reading notebook");
      await h.edit("SendToName", "Notebook 日本語");
      await h.resize(390, 844);
      assert.equal((await h.control(e => e.name === "SendToName", "resized destination name")).text, "Notebook 日本語");
      assert.equal((await h.control(e => e.name === "SendToTemplate", "resized URL template")).text, template);
      await h.clickLabel("Save destination");
      await dismiss(h);
      await h.resize(1363, 936);
      const articles = (await h.state()).articles;
      await h.clickName("ArticleSendTo");
      await menu(h, "Notebook 日本語");
      await h.capture("17-send-to-menu");
      await dismiss(h);
      assert.deepEqual((await h.state()).articles, articles, "opening or dismissing Send to must not modify the article");
      await h.page.evaluate("readerApp.flushSave(); import('./host.js').then(m => m.flush())");
      await h.reload(); await h.ready(); await go(h);
      assert.deepEqual(await destinations(h), [{id: 1, name: "Notebook 日本語", url_template: template, enabled: true}]);
      await scanItem(h);
      await h.page.press("t", ["shift"]);
      await menu(h, "Notebook 日本語");
      assert(!(await h.snapshot()).elements.some(e => e.name === "ArticleEntry"), "Shift+T preserves a collapsed scanned item");
      await clickMenu(h, "Customize Send to");
      await h.clickLabel("Remove Notebook 日本語");
      await h.until(async () => !(await destinations(h)).length, "destination removed");
      await dismiss(h);
      assert.deepEqual(await targets(h), existing, "menus must not submit an article until a destination is activated");
    });

    await check("touch Send to remains within the viewport and Back dismisses its native menu", async () => {
      await h.resize(390, 844);
      await replace(h, fixture(baseline, {mode: "Expanded", configured: true})); await go(h);
      touch = await createTouch(h.page);
      const button = await h.control(e => e.name === "ArticleSendTo", "touch Send to");
      assert(button.height >= 44, "the touch Send to target must be at least 44px high");
      await touch.tapControl(button);
      const item = await menu(h, "Reading notebook"), view = await h.snapshot();
      assert(item.height >= 44, "touch destinations need 44px targets");
      assert(item.x >= 0 && item.y >= 0 && item.x + item.width <= view.width + 1 && item.y + item.height <= view.height + 1,
        "the native popup must stay inside the phone viewport");
      await h.until(() => h.page.evaluate("history.state?.reader?.overlay === true"), "menu participates in browser Back");
      const history = await h.page.send("Page.getNavigationHistory", {});
      await h.page.send("Page.navigateToHistoryEntry", {entryId: history.entries[history.currentIndex - 1].id});
      await h.until(() => h.page.evaluate("!readerApp.hasOverlay()"), "Back closes touch Send to");
      assert.equal(await read(h), false);
      assert.deepEqual(await targets(h), existing);
      await h.capture("17-send-to-phone");
    });

    await check("original Keep unread and Mark as read checkboxes retain their distinct state", async () => {
      await replace(h, fixture(baseline, {mode: "Expanded", automatic: true})); await go(h);
      let control = await checkbox(h);
      assert.equal(control.automationName, "Keep unread"); assert.equal(control.checked, false);
      assert(control.height >= 44);
      await touch.tapControl(control);
      await h.until(async () => (await checkbox(h)).checked === true, "Keep unread changes even when the item was already unread");
      assert.equal(await read(h), false);
      await touch.tapControl(await checkbox(h));
      await h.until(() => read(h), "clearing Keep unread marks the item read");
      control = await checkbox(h);
      assert.equal(control.automationName, "Keep unread"); assert.equal(control.checked, false);
      await replace(h, fixture(baseline, {mode: "Expanded", isRead: true})); await go(h);
      control = await checkbox(h);
      assert.equal(control.automationName, "Mark as read"); assert.equal(control.checked, true);
      await touch.tapControl(control);
      await h.until(async () => !(await read(h)), "manual read checkbox marks unread");
      control = await checkbox(h);
      assert.equal(control.automationName, "Mark as read"); assert.equal(control.checked, false);
      await touch.tapControl(control);
      await h.until(() => read(h), "manual read checkbox marks read");
      await h.capture("17-reader-read-state");
    });
    passed = true;
  } finally {
    await touch?.disable();
    if (passed) {
      await h.resize(1363, 936);
      await replace(h, baseline);
    }
  }
}
