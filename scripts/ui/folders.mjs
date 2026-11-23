import assert from "node:assert/strict";
import { sleep } from "../../FerroUI/scripts/browser/harness.mjs";
import { createTouch } from "./touch.mjs";
import { clickItem, itemState, viewportBounds, inside } from "./menu-fixture.mjs";
import { BASE, GROUP, installFixture, replace, navigate, active, folder, center, context,
  keyboardContext, dismiss, undo, reveal } from "./folder-fixture.mjs";

export async function folderChecks(h, check, site) {
  const baseline = await h.state(), viewport = await h.snapshot();
  const route = await h.page.evaluate("readerApp.currentView()");
  const fixture = await installFixture(h, baseline, site);
  let touch;
  const prepare = async () => {
    await replace(h, fixture.data);
    await navigate(h, {Feed: BASE + 2});
    await h.control(element => folder(element), "fixture folder");
  };
  const confirm = async label => { await context(h); await clickItem(h, label); await h.control(e => e.name === "FolderConfirmation", "folder confirmation"); };
  try {
    await h.resize(1363, 936);
    await check("folder menus refresh their captured scope without changing the reading location", async () => {
      await prepare();
      const menu = await context(h), view = await h.snapshot();
      viewportBounds(menu, view);
      for (const label of ["Refresh folder", "Rename folder...", "Delete folder...", "Unsubscribe from all..."])
        await itemState(h, label, {enabled: true});
      assert.deepEqual(JSON.parse(await h.page.evaluate("readerApp.currentView()")), {Feed: BASE + 2});
      await h.capture("22-folder-menu");
      await clickItem(h, "Refresh folder");
      await h.until(async () => (await h.state()).feeds.filter(feed => feed.id < BASE + 2).every(feed => feed.fetched_at > 1),
        "only the captured folder finishes its refresh");
      assert.equal((await h.state()).feeds.find(feed => feed.id === BASE + 2).fetched_at, 1);
      assert.deepEqual(JSON.parse(await h.page.evaluate("readerApp.currentView()")), {Feed: BASE + 2});
      await h.click(element => folder(element), "open folder from its navigation row");
      await keyboardContext(h);
      await itemState(h, "Refresh folder", {enabled: true, selected: true, focused: true});
      await h.page.press("ArrowDown");
      await itemState(h, "Rename folder...", {enabled: true, selected: true, focused: true});
      await dismiss(h);
      await h.clickLabel("Folder settings");
      await itemState(h, "Delete folder...", {enabled: true});
      await dismiss(h);
    });

    await check("folder rename retains invalid drafts, trims valid names and preserves collapsed membership", async () => {
      await prepare();
      await h.clickLabel("Search scope"); await clickItem(h, GROUP);
      await h.clickLabel(`Toggle ${GROUP}`);
      await h.click(element => folder(element), "collapsed folder navigation");
      await h.clickLabel("Folder settings");
      await clickItem(h, "Rename folder...");
      for (const invalid of ["   ", "Reference"]) {
        await h.edit("DialogInput", invalid, true);
        await h.control(element => element.name === "DialogInput" && element.focused, "invalid folder draft remains editable");
        assert((await h.state()).folders.includes(GROUP));
        assert((await h.snapshot()).elements.some(element => element.foreground === "#B3261E" && element.text),
          "folder validation explains the rejected draft inline");
      }
      await h.capture("22-folder-rename-validation");
      await h.edit("DialogInput", "  Development  ", true);
      await h.until(async () => JSON.parse(await h.page.evaluate("readerApp.currentView()")).Folder === "Development", "rename retains the active folder route");
      const renamed = await h.state();
      assert(!renamed.folders.includes(GROUP));
      assert(renamed.folders.includes("Development"));
      assert.deepEqual(renamed.feeds.find(feed => feed.id === BASE + 1).folders, ["Development", "Reference"]);
      assert(!(await h.snapshot()).elements.some(element => element.name === "SubscriptionLink" && element.automationName?.startsWith("Compiler notes")),
        "the renamed folder retains its collapsed state");
      await h.clickLabel("Toggle Development");
      await h.control(element => element.name === "SubscriptionLink" && element.automationName?.startsWith("Compiler notes"), "renamed folder expands its original subscriptions");
      await h.capture("22-folder-renamed");
      await h.clickLabel("Search scope"); await itemState(h, "Development", {checked: true}); await dismiss(h);
      await h.edit("SearchReader", "compiler", true);
      assert.equal(JSON.parse(await h.page.evaluate("readerApp.currentView()")).Search, 'compiler folder:"Development"',
        "search uses the renamed folder shown in its scope choice");
      await navigate(h, {Folder: "Development"});
      await undo(h);
      await h.until(async () => JSON.parse(await h.page.evaluate("readerApp.currentView()")).Folder === GROUP,
        "undoing an active-folder rename restores its original reading route");
    });

    await check("deleting a folder preserves subscriptions and articles and supports a single undo", async () => {
      await prepare();
      await h.clickLabel("Search scope"); await clickItem(h, GROUP);
      const before = await h.state();
      await confirm("Delete folder...");
      await h.capture("22-folder-delete");
      await h.clickText("Cancel");
      assert.deepEqual((await h.state()).feeds, before.feeds);
      await confirm("Delete folder...");
      await h.clickName("ConfirmFolderAction");
      await h.until(async () => !(await h.state()).folders.includes(GROUP), "delete removes only the folder label");
      const deleted = await h.state();
      assert.deepEqual(deleted.articles, before.articles);
      assert.deepEqual(active(deleted).map(feed => feed.id), active(before).map(feed => feed.id));
      assert(deleted.feeds.every(feed => !(feed.folders || []).includes(GROUP) && feed.folder !== GROUP));
      await h.clickLabel("Search scope"); await itemState(h, "All items", {checked: true}); await dismiss(h);
      await h.edit("SearchReader", "design", true);
      assert.equal(JSON.parse(await h.page.evaluate("readerApp.currentView()")).Search, "design",
        "deleting a scoped folder removes the hidden folder filter too");
      await undo(h);
      await h.until(async () => (await h.state()).folders.includes(GROUP), "one undo restores the deleted folder");
      assert.deepEqual((await h.state()).feeds, before.feeds);
    });

    await check("bulk unsubscribe rejects a confirmation whose folder membership has changed", async () => {
      await prepare();
      await confirm("Unsubscribe from all...");
      const url = new URL("new-folder-subscription.xml", h.page.url).href;
      const opml = `<opml version="2.0"><head><title>Concurrent membership</title></head><body>`
        + `<outline text="${GROUP}"><outline type="rss" text="New subscription" xmlUrl="${url}"/></outline></body></opml>`;
      await h.page.evaluate(`readerApp.importOpml(${JSON.stringify(opml)})`);
      await h.until(async () => (await h.snapshot()).elements.some(element => element.name === "ConfirmFolderAction" && !element.enabled),
        "a stale bulk action becomes disabled");
      await h.control(element => element.name === "FolderConfirmationError" && element.text.includes("has changed"), "the stale target is explained");
      assert.equal(active(await h.state()).length, 4);
      await h.capture("22-folder-stale-confirmation");
      await dismiss(h);
    });

    await check("bulk unsubscribe retains annotated articles and ignores late refresh results before undo", async () => {
      await prepare();
      await h.clickLabel("Search scope"); await clickItem(h, "Compiler notes");
      const before = await h.state();
      const epoch = await h.page.evaluate(`readerApp.subscriptionEpoch(${BASE})`);
      await confirm("Unsubscribe from all...");
      await h.clickName("ConfirmFolderAction");
      await h.until(async () => active(await h.state()).length === 1, "both folder subscriptions are removed together");
      const removed = await h.state();
      assert.deepEqual(removed.articles.map(article => article.id).sort(), [0, 2, 3, 5].map(index => BASE + 100 + index));
      assert.deepEqual(removed.feeds.filter(feed => feed.archived).map(feed => feed.id).sort(), [BASE, BASE + 1]);
      const feed = before.feeds[0];
      await h.page.evaluate(`readerApp.receiveFeed(${JSON.stringify(feed.url)},${JSON.stringify(fixture.xml(0, true))},false,${feed.id},${JSON.stringify(feed.url)},${epoch})`);
      await h.page.evaluate(`readerApp.feedError(${JSON.stringify(feed.url)},"Late failure must be ignored",${feed.id},${epoch})`);
      assert.deepEqual((await h.state()).feeds, removed.feeds, "stale success and failure cannot reactivate or mutate an archived source");
      assert.deepEqual((await h.state()).articles, removed.articles);
      await h.clickLabel("Search scope"); await itemState(h, "All items", {checked: true}); await dismiss(h);
      await h.edit("SearchReader", "design", true);
      assert.equal(JSON.parse(await h.page.evaluate("readerApp.currentView()")).Search, "design",
        "an archived source cannot remain as an undisclosed search filter");
      await navigate(h, "Starred");
      await h.control(element => element.text === "Saved compiler article", "the retained starred article is readable");
      assert((await h.snapshot()).elements.some(element => element.text === "Compiler notes"), "archived articles retain their source title");
      await h.capture("22-retained-starred");
      await undo(h);
      await h.until(async () => active(await h.state()).length === 3, "one undo restores the whole folder unsubscribe");
      assert.deepEqual((await h.state()).articles, before.articles);
      assert.notEqual(await h.page.evaluate(`readerApp.subscriptionEpoch(${BASE})`), epoch,
        "reactivation starts a new source lifetime even when its ID is retained");
      await h.page.evaluate(`readerApp.receiveFeed(${JSON.stringify(feed.url)},${JSON.stringify(fixture.xml(0, true))},false,${feed.id},${JSON.stringify(feed.url)},${epoch})`);
      assert.deepEqual((await h.state()).articles, before.articles,
        "a refresh from before unsubscribe remains stale after Undo reactivates the same source ID");
    });

    await check("resubscription and undo preserve new downloads, saved annotations and reloadable source identity", async () => {
      await prepare();
      await confirm("Unsubscribe from all...");
      await h.clickName("ConfirmFolderAction");
      await h.until(async () => active(await h.state()).length === 1, "archive is ready for resubscription");
      await fixture.update(0, true);
      await h.clickName("SubscribeButton");
      await h.edit("DialogInput", fixture.data.feeds[0].url, true);
      await h.until(async () => (await h.state()).articles.some(article => article.guid === "folder-article-6"), "resubscription fetches a new item");
      const resubscribed = await h.state();
      assert.equal(resubscribed.feeds.filter(feed => feed.url === fixture.data.feeds[0].url).length, 1);
      assert.equal(resubscribed.feeds.find(feed => feed.url === fixture.data.feeds[0].url).id, BASE);
      assert(resubscribed.articles.find(article => article.guid === "folder-article-0").starred);
      await undo(h);
      await h.until(async () => active(await h.state()).length === 3, "undo reconciles reactivated and still-archived sources");
      const restored = await h.state();
      assert.equal(restored.articles.length, fixture.data.articles.length + 1);
      assert.equal(new Set(restored.articles.map(article => `${article.feed_id}:${article.guid}`)).size, restored.articles.length);
      assert(restored.articles.some(article => article.guid === "folder-article-6"));
      await h.page.evaluate("readerApp.flushSave(); import('./host.js').then(module => module.flush())");
      await h.reload(); await h.ready();
      assert.deepEqual((await h.state()).feeds, restored.feeds);
      assert.deepEqual((await h.state()).articles, restored.articles);
      await h.record("22-subscription-lifecycle", {activeSources: active(restored).length, articles: restored.articles.length,
        sameSourceIdentity: true, savedAnnotationsRetained: true, newDownloadsRetained: true, reloadVerified: true});
    });

    await check("phone folder holds cancel cleanly and keep bulk actions reachable in a short viewport", async () => {
      await h.resize(320, 740);
      await replace(h, fixture.data); await navigate(h, "All");
      touch = await createTouch(h.page);
      await h.clickLabel("Show subscriptions");
      let target = await h.control(element => folder(element), "phone folder");
      const startingView = await h.page.evaluate("readerApp.currentView()");
      await touch.start(center(target)); await sleep(650); await touch.cancel();
      await sleep(100);
      assert(!(await h.snapshot()).elements.some(element => element.name === "ReaderMenu"));
      await touch.start(center(target)); await sleep(650);
      await touch.move([center(target)[0] + 20, center(target)[1]]); await touch.move(center(target)); await touch.end();
      await sleep(100);
      assert(!(await h.snapshot()).elements.some(element => element.name === "ReaderMenu"));
      assert.equal(await h.page.evaluate("readerApp.currentView()"), startingView);
      target = await h.control(element => folder(element), "phone folder after cancelled gesture");
      await touch.start(center(target)); await sleep(650); await touch.end();
      const menu = await h.control(element => element.name === "ReaderMenu", "held folder actions");
      viewportBounds(menu, await h.snapshot());
      assert((await itemState(h, "Unsubscribe from all...")).height >= 44);
      assert.equal(await h.page.evaluate("readerApp.currentView()"), startingView, "a hold never opens the folder");
      await h.capture("22-folder-phone-menu");
      await clickItem(h, "Unsubscribe from all...");
      await h.resize(320, 360);
      const action = await reveal(h, "ConfirmFolderAction"), view = await h.snapshot();
      assert(action.height >= 44);
      assert(inside(action, {x:0, y:0, width:view.width, height:view.height}));
      await h.capture("22-folder-phone-confirmation");
      await dismiss(h);
      assert.equal(active(await h.state()).length, 3, "cancel leaves every subscription in place");
    });
  } catch (error) {
    await h.capture("22-folder-failure").catch(() => {});
    throw error;
  } finally {
    await touch?.cancel(); await touch?.disable();
    await h.page.press("Escape").catch(() => {});
    await replace(h, baseline);
    await h.page.evaluate(`readerApp.navigate(${JSON.stringify(route)})`);
    await h.resize(viewport.width, viewport.height);
    await h.page.evaluate("readerApp.flushSave(); import('./host.js').then(module => module.flush())");
    await fixture.cleanup();
  }
}
