import assert from "node:assert/strict";
import { writeFile, rm } from "node:fs/promises";
import path from "node:path";
import { createHarness } from "./ui/harness.mjs";
import { mobileChecks } from "./ui/mobile.mjs";
import { refinementChecks } from "./ui/refinement.mjs";
import { feedNetworkChecks } from "./ui/feed-network.mjs";
import { interceptStarterFeeds, starterChecks } from "./ui/starter.mjs";
import { sleep } from "../FerroUI/scripts/browser/harness.mjs";

const site = path.resolve(process.argv[2] || "target/browser/reader-browser");
const output = path.resolve(process.argv[3] || "target/ui-smoke");
let starter;
const h = await createHarness(site, output, {
  beforeNavigate: async page => { starter = await interceptStarterFeeds(page); },
});
const checks = [];
async function check(name, run) {
  const start = performance.now();
  await run();
  checks.push({ name, milliseconds: Math.round(performance.now() - start) });
  console.log(`ok ${name}`);
}
try {
  await check("native first paint", async () => { await h.ready(); await h.capture("01-home"); });
  if (process.argv.includes("--capture-only")) process.exitCode = 0;
  else {
    await starterChecks(h, check, starter);
    await check("stream navigation and list shortcut", async () => {
      await h.clickNavigation("All items");
      await h.capture("02-expanded");
      await h.page.press("2");
      await h.until(async () => (await h.state()).settings.view_mode === "List", "list view");
      await h.capture("03-list");
    });
    await check("scan, open, star, tag, and keep unread", async () => {
      const before = await h.state();
      await h.page.press("n");
      assert.equal((await h.state()).articles.filter(a => a.read).length, before.articles.filter(a => a.read).length, "scan must not mark read");
      await h.page.press("Enter");
      await h.page.press("s");
      const starred = (await h.state()).articles.find(a => a.starred !== before.articles.find(b => b.id === a.id)?.starred);
      assert(starred, "star shortcut changes the current item");
      await h.page.press("t");
      await h.edit("DialogInput", "Verification, Rust", true);
      await h.until(async () => (await h.state()).articles.find(a => a.id === starred.id).tags.includes("Verification"), "tags committed");
      await h.page.press("m");
      await h.until(async () => !(await h.state()).articles.find(a => a.id === starred.id).read, "keep unread");
    });
    await check("search typing does not trigger reading shortcuts", async () => {
      const before = await h.state();
      await h.edit("SearchReader", "is:starred tag:Verification");
      await h.inputHints("search", "search");
      await h.page.press("Enter");
      await h.until(() => h.page.evaluate("JSON.parse(readerApp.currentView()).Search === 'is:starred tag:Verification'"), "search route");
      assert.equal((await h.state()).articles.filter(a => a.starred).length, before.articles.filter(a => a.starred).length);
      await h.capture("04-search");
    });
    await check("browser modifiers and composition keys do not execute Reader commands", async () => {
      await h.clickNavigation("All items");
      await h.page.press("n");
      const before = (await h.state()).articles;
      await h.page.press("s", ["alt"]);
      await h.page.press("m", ["shift"]);
      await h.page.press("z", ["ctrl", "shift"]);
      await h.page.evaluate(`(() => {
        const host = document.getElementById('out');
        host.dispatchEvent(new KeyboardEvent('keydown', { key: 's', code: 'KeyS', isComposing: true, bubbles: true }));
        host.dispatchEvent(new KeyboardEvent('keyup', { key: 's', code: 'KeyS', isComposing: true, bubbles: true }));
      })()`);
      assert.deepEqual((await h.state()).articles, before, "modifiers and IME must preserve item annotations");
    });
    await check("feed subscription through the browser network host", async () => {
      const xml = `<?xml version="1.0"?><rss version="2.0"><channel><title>Reader verification</title><link>https://example.com/</link><description>Local test fixture</description><item><guid>reader-verification-1</guid><title>Native feed integration</title><link>https://example.com/native-feed</link><pubDate>Sun, 22 Nov 2026 10:00:00 GMT</pubDate><description><![CDATA[<p>A <strong>real feed</strong> rendered by FerroUI.</p><pre>let reader = true;\n    indented();</pre>]]></description></item></channel></rss>`;
      await writeFile(path.join(site, "verification-feed.xml"), xml);
      const url = new URL("verification-feed.xml", h.page.url).href;
      await h.clickName("SubscribeButton");
      await h.edit("DialogInput", url);
      await h.inputHints("url", "go");
      await h.page.press("Enter");
      await h.until(async () => (await h.state()).feeds.some(f => f.url === url), "downloaded subscription");
      await h.until(async () => (await h.state()).articles.some(a => a.title === "Native feed integration"), "parsed feed item");
      await h.capture("05-feed");
    });
    await check("preferences menu and exact compact density", async () => {
      await h.clickLabel("Settings");
      await h.clickText("Reader settings");
      await h.clickText("Compact");
      await h.until(async () => (await h.state()).settings.density === "Compact", "compact preference");
      await h.capture("06-settings");
      await h.clickLabel("Close dialog");
      await h.capture("07-compact");
    });
    await check("atomic IndexedDB persistence survives reload", async () => {
      const expected = await h.state();
      await h.page.evaluate("readerApp.flushSave(); import('./host.js').then(m => m.flush())");
      await h.reload();
      await h.ready();
      const actual = await h.state();
      assert.deepEqual(actual.feeds, expected.feeds);
      assert.deepEqual(actual.articles.map(a => [a.id, a.starred, a.tags]), expected.articles.map(a => [a.id, a.starred, a.tags]));
      assert.equal(actual.settings.density, "Compact");
    });
    await check("competing storage writers cannot silently overwrite a newer generation", async () => {
      const result = await h.page.evaluate(`(async () => {
        const primary = await import('./reader-storage.js');
        const stale = await import('./reader-storage.js?independent-writer');
        await stale.prepareStorage();
        const current = readerApp.exportData();
        primary.save(current);
        await primary.flushStorage();
        let message = '';
        stale.setStorageReporter(error => { message = error.message; });
        stale.save('stale writer');
        await stale.flushStorage();
        const observer = await import('./reader-storage.js?independent-observer');
        await observer.prepareStorage();
        return { message, preserved: observer.load() === current };
      })()`);
      assert.match(result.message, /changed in another tab/);
      assert(result.preserved, "a stale writer must preserve the committed library");
    });
    await refinementChecks(h, check);
    await feedNetworkChecks(h, check);
    await mobileChecks(h, check);
    await check("expanded read tracking follows user scrolling and preserves Keep unread", async () => {
      const data = await h.state();
      for (const article of data.articles) { article.read = false; article.read_at = null; }
      data.settings.mark_read_on_scroll = true;
      data.stream_settings = {};
      await h.page.evaluate(`readerApp.importData(${JSON.stringify(JSON.stringify(data))})`);
      await h.clickNavigation("All items");
      await h.page.press("1");
      await h.page.press("n");
      await h.page.press("n");
      assert.equal((await h.state()).articles.filter(a => a.read).length, 0, "programmatic cursor scrolling must not mark read");
      await h.resize(1363, 820);
      assert.equal((await h.state()).articles.filter(a => a.read).length, 0, "viewport layout must not mark read");
      await h.page.wheel(900, 500, 0, 420);
      const read = await h.until(async () => (await h.state()).articles.find(a => a.read), "user scrolling marks the reading line");
      await h.page.press("m");
      await h.until(async () => !(await h.state()).articles.find(a => a.id === read.id).read, "Keep unread applies to the current entry");
      await h.page.wheel(900, 500, 0, 15);
      await sleep(250);
      assert.equal((await h.state()).articles.find(a => a.id === read.id).read, false, "small scroll must not undo explicit Keep unread");
      await h.resize(1363, 936);
    });
    await check("ten thousand items keep a bounded native control tree", async () => {
      const data = await h.state();
      const feed = data.feeds[0];
      data.articles = Array.from({ length: 10000 }, (_, index) => ({ id: 100000 + index, feed_id: feed.id, guid: `stress-${index}`, title: `Virtualized item ${index}`, url: `https://example.com/items/${index}`, content: "<p>A short item used to exercise retained controls.</p>", summary: "Virtualization fixture", author: "", published: 1795341600 - index, read: false, starred: false, tags: [], note: "", read_at: null }));
      data.next_id = 110001;
      data.settings.view_mode = "List";
      data.settings.unread_only = false;
      data.stream_settings = {};
      await h.page.evaluate(`readerApp.importData(${JSON.stringify(JSON.stringify(data))})`);
      await h.clickNavigation("All items");
      await h.page.press("2");
      const tree = await h.until(async () => {
        const view = await h.snapshot();
        return view.articleContainers > 0 && view;
      }, "virtualized item realization");
      assert(tree.visualCount < 5000, `${tree.visualCount} retained controls for 10000 items`);
      assert(tree.articleContainers > 0 && tree.articleContainers < 120, `${tree.articleContainers} realized items for 10000 records`);
      await h.page.wheel(900, 500, 0, 2200);
      await h.capture("09-virtualized");
    });
    await check("installed application and saved library reload without network", async () => {
      await h.page.evaluate("import('./host.js').then(m => m.flush())");
      const url = new URL(h.page.url);
      url.search = "";
      await h.navigate(url.href);
      await h.ready();
      const build = await h.page.evaluate("document.documentElement.dataset.readerBuild");
      assert.match(build, /^[a-f0-9]{16}$/);
      await h.until(() => h.page.evaluate("navigator.serviceWorker?.controller?.state === 'activated'"),
        "offline worker activates", 30000);
      assert(await h.page.evaluate("new URL(navigator.serviceWorker.controller.scriptURL).pathname.endsWith('/app-sw.js')"),
        "Reader's own worker must control the offline navigation");
      const offline = await h.page.send("Network.emulateNetworkConditions", { offline: true, latency: 0, downloadThroughput: 0, uploadThroughput: 0 });
      assert(!offline.error, JSON.stringify(offline));
      await h.until(() => h.page.evaluate("navigator.onLine === false"), "browser network is offline");
      await h.reload();
      await h.ready();
      assert.equal(await h.page.evaluate("document.documentElement.dataset.readerBuild"), build,
        "offline navigation must restore the installed application version");
      assert.equal((await h.state()).articles.length, 10000);
      await h.capture("10-offline");
      await h.page.send("Network.emulateNetworkConditions", { offline: false, latency: 0, downloadThroughput: -1, uploadThroughput: -1 });
    });
    h.healthy();
    await writeFile(path.join(output, "results.json"), JSON.stringify({ passed: checks.length, checks }, null, 2));
    console.log(`All ${checks.length} browser checks passed.`);
  }
} catch (error) {
  await h.page.screenshot(path.join(output, "failure.png")).catch(() => {});
  await writeFile(path.join(output, "failure.json"), JSON.stringify(await h.snapshot().catch(() => null), null, 2));
  console.error(error);
  console.error(h.page.log.slice(-20).join("\n"));
  process.exitCode = 1;
} finally {
  await rm(path.join(site, "verification-feed.xml"), { force: true });
  await h.page.close();
}
