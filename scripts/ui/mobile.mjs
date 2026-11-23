import assert from "node:assert/strict";
import { sleep } from "../../FerroUI/scripts/browser/harness.mjs";
import { createTouch } from "./touch.mjs";

const readCount = data => data.articles.filter(article => article.read).length;
const center = e => [e.x + e.width / 2, e.y + e.height / 2];

async function visibleRow(h) {
  const view = await h.snapshot();
  const row = view.elements.find(e => e.name === "ArticleRow" && e.y >= 200 && e.y + e.height < view.height - 80);
  assert(row, "an article row must be visible for touch input");
  return row;
}
async function headlineBackgroundPoint(h, row) {
  const view = await h.snapshot();
  const date = view.elements.find(e => e.name === "ArticleHeadlineDate"
    && e.x >= row.x && e.x + e.width <= row.x + row.width
    && e.y >= row.y && e.y + e.height <= row.y + row.height);
  assert(date, "the headline date identifies the non-button tap region");
  // Stay below its glyphs but within its column: the far-right 44px region
  // now belongs to the independent View original action.
  return [date.x + date.width / 2, row.y + row.height - 8];
}
async function settledScroll(h) {
  let last = -1;
  let stable = 0;
  await h.until(async () => {
    const offset = (await h.snapshot()).articleScroll?.offsetY;
    stable = Math.abs(offset - last) < 0.2 ? stable + 1 : 0;
    last = offset;
    await sleep(60);
    return stable >= 3;
  }, "touch inertia settles", 5000);
}
async function back(h, { nested = false } = {}) {
  const current = await h.page.send("Page.getNavigationHistory", {});
  assert(!current.error && current.currentIndex > 0, "browser history must contain the preceding entry");
  const result = await h.page.send("Page.navigateToHistoryEntry", { entryId: current.entries[current.currentIndex - 1].id });
  assert(!result.error, JSON.stringify(result));
  // A remaining underlying drawer replaces the consumed transient guard. Its
  // index stays the same; the caller asserts that only the top menu disappeared.
  if (!nested) await h.until(async () => (await h.page.send("Page.getNavigationHistory", {})).currentIndex === current.currentIndex - 1,
    "Back consumes exactly the overlay history entry");
}
function inside(e, view) {
  assert(e.x >= -1 && e.y >= -1 && e.x + e.width <= view.width + 1 && e.y + e.height <= view.height + 1,
    `${e.name || e.automationName} is outside ${view.width} × ${view.height}: ${JSON.stringify(e)}`);
}
const restoreViewport = h => h.page.evaluate(`(() => {
  for (const name of ['width','height','offsetTop','offsetLeft','scale']) delete visualViewport[name];
  visualViewport.dispatchEvent(new Event('resize'));
})()`);
const overlayReady = h => h.until(() => h.page.evaluate(
  "readerApp.hasOverlay() && history.state?.reader?.overlay === true && history.state.reader.view === readerApp.currentView()"),
  "native overlay is represented in browser history");

export async function mobileChecks(h, check) {
  const touch = await createTouch(h.page);
  let failed = false;
  try {
    await check("phone touch scrolling, cancellation and tap-to-expand", async () => {
      await h.resize(390, 844);
      await h.clickLabel("Show subscriptions");
      await h.clickNavigation("Home");
      await h.capture("08-mobile-home");
      await h.clickLabel("Show subscriptions");
      await h.clickNavigation("All items");
      await h.page.press("1");
      await h.capture("08-mobile");
      const data = await h.state();
      data.articles = Array.from({ length: 60 }, (_, index) => ({
        id: 50000 + index, feed_id: data.feeds[0].id, guid: `touch-${index}`,
        title: `Touch reading item ${index}`, url: `https://example.com/touch/${index}`,
        content: "<p>A short article for checking taps and native touch scrolling.</p><p>Keep reading without accidental commands.</p>",
        summary: "Touch input fixture", author: "", published: 1795341600 - index,
        read: false, starred: false, tags: [], note: "", read_at: null,
      }));
      data.next_id = 50100;
      data.settings.mark_read_on_scroll = false;
      data.stream_settings = {};
      await h.page.evaluate(`readerApp.importData(${JSON.stringify(JSON.stringify(data))})`);
      await h.clickLabel("Show subscriptions");
      await h.clickNavigation("All items");
      await h.page.press("2");
      await h.until(async () => (await h.state()).settings.view_mode === "List", "phone list view");
      const row = await visibleRow(h);
      assert(row.height >= 44, "touch article rows need a 44px target");
      const from = await headlineBackgroundPoint(h, row);
      const beforeOffset = (await h.snapshot()).articleScroll.offsetY;
      await touch.start(from);
      await sleep(60);
      assert.equal(readCount(await h.state()), 0, "touch down must not mark an item read");
      await touch.travel(from, [from[0], 145]);
      await touch.end();
      await h.until(async () => (await h.snapshot()).articleScroll.offsetY > beforeOffset + 30, "native touch scrolling");
      await settledScroll(h);
      assert.equal(readCount(await h.state()), 0, "a scroll beginning on an article must not open it");

      const canceled = await visibleRow(h);
      // Tap below the date's glyphs: the entire 44px row is the target.
      const point = await headlineBackgroundPoint(h, canceled);
      await touch.start(point);
      await touch.cancel();
      await sleep(120);
      assert.equal(readCount(await h.state()), 0, "touchCancel must not open an item");
      await touch.start(point);
      await touch.start(point, [point[0] - 45, point[1] + 8, 1]);
      await touch.end();
      await sleep(120);
      assert.equal(readCount(await h.state()), 0, "a second contact must cancel the pending tap");

      const starPoint = [canceled.x + 22, point[1]];
      await touch.start(starPoint);
      await touch.move([starPoint[0] + 18, starPoint[1]]);
      await touch.move(starPoint);
      await touch.end();
      await sleep(120);
      assert(!(await h.state()).articles.some(a => a.starred), "movement cancels a button tap even after returning to its origin");
      await touch.tap(...starPoint);
      await h.until(async () => (await h.state()).articles.find(a => a.id === canceled.articleId)?.starred, "star is an independent in-place touch action");
      assert.equal(readCount(await h.state()), 0, "starring does not open an item");

      await touch.tap(...point);
      await h.until(async () => (await h.state()).articles.find(a => a.id === canceled.articleId)?.read, "touch release opens the article");
      await h.until(async () => (await h.snapshot()).elements.some(e => e.name === "ArticleEntry" && e.articleId === canceled.articleId), "headline expands in place");
      assert.equal(readCount(await h.state()), 1);
      await h.capture("08-mobile-reading");
      await touch.tapControl(await h.control(e => e.name === "ArticleReadState" && e.automationName === "Mark as read", "in-place read state"));
      await h.until(async () => readCount(await h.state()) === 0, "touch read-state checkbox marks the item unread");
    });

    await check("browser Back dismisses mobile overlays before changing streams", async () => {
      const route = await h.page.evaluate("readerApp.currentView()");
      await touch.tapControl(await h.control(e => e.automationName === "Show subscriptions", "navigation drawer toggle"));
      await overlayReady(h);
      await h.capture("08-mobile-navigation");
      await back(h);
      await h.until(async () => !(await h.snapshot()).elements.some(e => e.name === "NavigationPane"), "Back closes the drawer");
      assert.equal(await h.page.evaluate("readerApp.currentView()"), route);

      await h.page.press("a");
      await h.control(e => e.name === "DialogInput", "subscription input");
      await overlayReady(h);
      await back(h);
      await h.until(async () => !(await h.snapshot()).elements.some(e => e.name === "ReaderDialog"), "Back closes the dialog");
      assert.equal(await h.page.evaluate("readerApp.currentView()"), route);

      await h.clickLabel("More actions");
      await h.control(e => /MenuItem$/.test(e.type) && e.text === "List view", "phone reading menu");
      await overlayReady(h);
      await h.capture("08-mobile-menu");
      await back(h);
      await h.until(async () => !(await h.snapshot()).elements.some(e => /MenuItem$/.test(e.type) && e.text === "List view"), "Back closes the reading menu");
      assert.equal(await h.page.evaluate("readerApp.currentView()"), route);
    });

    await check("held subscription menus, cancellation, reordering and folder movement", async () => {
      await h.clickLabel("Show subscriptions");
      const data = await h.state();
      const feed = data.feeds.find(f => f.title === "Smashing Magazine");
      assert(feed, "the starting library includes the reference subscription");
      const source = await h.control(e => /Button$/.test(e.type) && e.automationName?.startsWith(`${feed.title} (`), "drag source");
      const route = await h.page.evaluate("readerApp.currentView()");
      await touch.start(center(source));
      await sleep(650);
      await touch.cancel();
      await sleep(120);
      assert.deepEqual((await h.state()).feeds.find(f => f.id === feed.id).folders, feed.folders);
      assert.equal(await h.page.evaluate("readerApp.currentView()"), route, "cancelling a hold must not navigate");

      await touch.start(center(source));
      await sleep(650);
      await touch.end();
      await h.control(e => /MenuItem$/.test(e.type) && e.text === "Rename subscription...", "stationary hold opens subscription actions");
      await overlayReady(h);
      await back(h, { nested: true });
      await h.until(async () => !(await h.snapshot()).elements.some(e => /MenuItem$/.test(e.type) && e.text === "Rename subscription..."), "Back dismisses the held subscription menu");
      await h.until(async () => (await h.snapshot()).elements.some(e => e.name === "NavigationPane"), "menu dismissal preserves its underlying drawer");
      await overlayReady(h);
      assert.equal(await h.page.evaluate("readerApp.currentView()"), route);

      const sibling = data.feeds.find(f => f.title === "A List Apart");
      assert(sibling);
      const siblingRow = await h.control(e => /Button$/.test(e.type) && e.automationName?.startsWith(`${sibling.title} (`), "reorder destination");
      await touch.start(center(source));
      await sleep(650);
      await touch.travel(center(source), [siblingRow.x + siblingRow.width / 2, siblingRow.y + siblingRow.height - 5]);
      await h.capture("08-mobile-insertion");
      await touch.end();
      await h.until(async () => {
        const order = (await h.state()).subscription_order?.folders?.[feed.folder] || [];
        const rows = (await h.snapshot()).elements;
        const moved = rows.find(e => e.name === "SubscriptionLink" && e.automationName?.startsWith(`${feed.title} (`));
        const anchor = rows.find(e => e.name === "SubscriptionLink" && e.automationName?.startsWith(`${sibling.title} (`));
        return order.indexOf(feed.id) === order.indexOf(sibling.id) + 1
          && moved && anchor && moved.y > anchor.y;
      }, "touch drop persists the requested subscription order");
      assert.deepEqual((await h.state()).feeds.find(f => f.id === feed.id).folders, feed.folders,
        "reordering inside a folder retains all memberships");
      await h.capture("08-mobile-reordered");

      const destination = await h.control(e => /Button$/.test(e.type) && e.automationName === "Science", "destination folder");
      const reordered = await h.control(e => /Button$/.test(e.type) && e.automationName?.startsWith(`${feed.title} (`), "reordered drag source");
      await touch.drag(center(reordered), center(destination), 650);
      await h.until(async () => (await h.state()).feeds.find(f => f.id === feed.id).folder === "Science", "touch drag changes the subscription folder");
      assert.equal(await h.page.evaluate("readerApp.currentView()"), route, "drag release must not activate the subscription");
      const view = await h.snapshot();
      const pane = view.elements.find(e => e.name === "NavigationPane");
      assert(pane && pane.x + pane.width < view.width, "the drawer has an exposed dismiss surface");
      await touch.tap((pane.x + pane.width + view.width) / 2, view.height / 2);
      await h.until(async () => !(await h.snapshot()).elements.some(e => e.name === "NavigationPane"), "touch outside closes the drawer");
    });

    await check("small portrait and landscape dialogs keep actions reachable", async () => {
      for (const [width, height] of [[320, 568], [844, 390]]) {
        await h.resize(width, height);
        await h.page.press("a");
        const dialog = await h.control(e => e.name === "ReaderDialog", "responsive subscription dialog");
        const view = await h.snapshot();
        inside(dialog, view);
        inside(await h.control(e => e.name === "DialogInput", "visible dialog input"), view);
        inside(await h.control(e => e.automationName === "Close dialog", "visible dialog close"), view);
        inside(await h.control(e => /Button$/.test(e.type) && e.automationName === "Add", "reachable Add action"), view);
        inside(await h.control(e => /Button$/.test(e.type) && e.automationName === "Cancel", "reachable Cancel action"), view);
        await h.capture(width === 320 ? "08-mobile-small" : "08-mobile-landscape");
        await h.page.press("Escape");
      }
    });

    await check("visual viewport keyboard resize preserves dialog input and focus", async () => {
      await h.resize(390, 844);
      await h.page.press("a");
      await h.edit("DialogInput", "https://example.com/unfinished-feed.xml");
      await h.inputHints("url", "go");
      // Models the public visualViewport contract. A physical Safari keyboard is
      // not available on the Linux runner and is explicitly outside this check.
      await h.page.evaluate(`(() => {
        for (const [name, value] of Object.entries({ width: 390, height: 380, offsetTop: 30, offsetLeft: 0, scale: 1 }))
          Object.defineProperty(visualViewport, name, { configurable: true, value });
        visualViewport.dispatchEvent(new Event('resize'));
      })()`);
      await h.until(async () => (await h.snapshot()).height <= 380, "native surface follows the visible keyboard viewport");
      const host = await h.page.evaluate(`(() => {
        const { top, left, width, height } = document.getElementById('out').getBoundingClientRect();
        return { top, left, width, height };
      })()`);
      assert.deepEqual(host, { top: 30, left: 0, width: 390, height: 380 });
      const view = await h.snapshot();
      inside(await h.control(e => e.name === "ReaderDialog", "keyboard-sized dialog"), view);
      const input = await h.control(e => e.name === "DialogInput", "keyboard input");
      inside(input, view);
      assert.equal(input.text, "https://example.com/unfinished-feed.xml");
      assert(input.focused, "viewport resizing must preserve editing focus");
      await h.capture("08-mobile-keyboard");
      await restoreViewport(h);
      await h.until(async () => (await h.snapshot()).height === 844, "keyboard viewport restores");
      await h.page.press("Escape");
    });
  } catch (error) {
    failed = true;
    console.error("Mobile check failed:", error.stack || error);
    await h.capture("08-mobile-failure").catch(() => {});
    throw error;
  } finally {
    let cleanupError;
    for (const cleanup of [() => restoreViewport(h), touch.cancel, touch.disable, () => h.resize(1363, 936)]) {
      try { await cleanup(); }
      catch (error) { cleanupError ??= error; console.error("Mobile cleanup:", error.message); }
    }
    if (!failed && cleanupError) throw cleanupError;
  }
}
