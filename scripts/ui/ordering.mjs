import assert from "node:assert/strict";
import { sleep } from "../../FerroUI/scripts/browser/harness.mjs";
import { clickItem, itemState, viewportBounds } from "./menu-fixture.mjs";
import { dismiss, undo, keyboardContext } from "./folder-fixture.mjs";
import { createTouch } from "./touch.mjs";
import { BASE, ALPHA, BETA, GAMMA, TITLES, center, label, rows, fixture, replace,
  navigate, folder, occurrence, context, drag, expectRows } from "./ordering-fixture.mjs";

export async function orderingChecks(h, check) {
  const baseline = await h.state(), viewport = await h.snapshot();
  const route = await h.page.evaluate("readerApp.currentView()");
  const data = fixture(baseline), manual = [TITLES[3], BETA, TITLES[0], TITLES[1], TITLES[4],
    ALPHA, TITLES[0], TITLES[1], TITLES[2], GAMMA];
  let touch;
  const prepare = async () => {
    await h.resize(1363, 960); await replace(h, data); await navigate(h, {Feed: BASE + 3});
    await expectRows(h, manual);
  };
  const sort = async text => { await h.clickName("SubscriptionSettings"); await clickItem(h, text); };
  try {
    await check("native occurrence moves preserve other folders, articles and refresh epochs", async () => {
      await prepare();
      const before = await h.state();
      const epoch = await h.page.evaluate(`readerApp.subscriptionEpoch(${BASE})`);
      await context(h, await occurrence(h, TITLES[0], ALPHA));
      await itemState(h, "Move up", {enabled: false});
      await clickItem(h, "Move down");
      await h.until(async () => (await h.state()).subscription_order.folders[ALPHA][1] === BASE, "only the selected occurrence moves");
      await expectRows(h, [TITLES[3], BETA, TITLES[0], TITLES[1], TITLES[4], ALPHA, TITLES[1], TITLES[0], TITLES[2], GAMMA]);
      const moved = await h.state();
      assert.deepEqual(moved.subscription_order.folders[BETA], before.subscription_order.folders[BETA]);
      assert.deepEqual(moved.feeds, before.feeds); assert.deepEqual(moved.articles, before.articles);
      assert.equal(await h.page.evaluate(`readerApp.subscriptionEpoch(${BASE})`), epoch);
      await h.capture("23-independent-folder-order");
      await undo(h);
      await expectRows(h, manual);
      assert.deepEqual((await h.state()).subscription_order, before.subscription_order);
      await context(h, await folder(h, ALPHA)); await clickItem(h, "Sort subscriptions alphabetically");
      await h.until(async () => (await h.state()).subscription_order.folders[ALPHA][0] === BASE + 1, "one-shot folder sort");
      assert.deepEqual((await h.state()).subscription_order.folders[BETA], before.subscription_order.folders[BETA]);
      await undo(h); await expectRows(h, manual);
    });

    await check("folder drag orders mixed root nodes and keyboard traversal follows the same tree", async () => {
      await prepare();
      const source = await folder(h, GAMMA), target = (await rows(h))[0];
      const beforeRoute = await h.page.evaluate("readerApp.currentView()");
      await drag(h, source, [target.x + target.width / 2, target.y + 3], () => h.capture("23-folder-insertion"));
      await expectRows(h, [GAMMA, ...manual.slice(0, -1)]);
      assert.equal(await h.page.evaluate("readerApp.currentView()"), beforeRoute, "dragging a folder does not navigate");
      const root = (await h.state()).subscription_order.root;
      assert.deepEqual(root[0], {Folder: GAMMA}); assert.deepEqual(root[1], {Feed: BASE + 3});
      await h.click(e => e.name === "SubscriptionFolder" && e.automationName === ALPHA, "folder cursor anchor");
      for (const expected of [TITLES[0], TITLES[1], TITLES[2]]) {
        await h.page.press("n", ["shift"]);
        await h.until(async () => label((await rows(h)).find(e => e.treeCursor) || {}) === expected, "keyboard occurrence: " + expected);
      }
      await h.page.press("x", ["shift"]);
      await h.until(async () => (await rows(h)).find(e => e.treeCursor)?.automationName === ALPHA, "collapse promotes the occurrence cursor");
      assert(!(await rows(h)).some(e => e.name === "SubscriptionLink" && label(e) === TITLES[2]));
      await h.page.press("x", ["shift"]);
      await h.capture("23-mixed-root-order");
    });

    await check("subscription sorting radios persist and restore the untouched manual positions", async () => {
      await prepare();
      await h.clickName("SubscriptionSettings");
      await itemState(h, "Sort by drag and drop", {checked: true});
      await itemState(h, "Sort alphabetically", {checked: false});
      await h.capture("23-subscription-settings");
      await clickItem(h, "Sort alphabetically");
      const alpha = [ALPHA, TITLES[1], TITLES[2], TITLES[0], BETA, TITLES[1], TITLES[0], GAMMA, TITLES[4], TITLES[3]];
      await expectRows(h, alpha);
      assert.deepEqual((await h.state()).subscription_order.root, data.subscription_order.root);
      assert.deepEqual((await h.state()).subscription_order.folders, data.subscription_order.folders);
      await context(h, await occurrence(h, TITLES[0], ALPHA));
      await itemState(h, "Move up", {enabled: false}); await itemState(h, "Move down", {enabled: false}); await dismiss(h);
      await h.page.evaluate("readerApp.flushSave(); import('./host.js').then(module => module.flush())");
      await h.reload(); await h.ready(); await expectRows(h, alpha);
      await h.clickName("SubscriptionSettings"); await itemState(h, "Sort alphabetically", {checked: true}); await dismiss(h);
      await h.capture("23-alphabetical-order");
      await sort("Sort by drag and drop"); await expectRows(h, manual);
      assert.deepEqual((await h.state()).subscription_order, data.subscription_order);
    });

    await check("alphabetical drag changes only its source folder label and Undo restores both occurrences", async () => {
      await prepare(); await sort("Sort alphabetically");
      const before = await h.state();
      await drag(h, await occurrence(h, TITLES[0], ALPHA), center(await folder(h, GAMMA)));
      await h.until(async () => (await h.state()).feeds.find(f => f.id === BASE).folders.includes(GAMMA), "cross-folder drag in alphabetical mode");
      const after = await h.state(), moved = after.feeds.find(f => f.id === BASE);
      assert.deepEqual(moved.folders, [BETA, GAMMA]);
      assert.equal(after.subscription_order.sort, "Alphabetical");
      assert.deepEqual(after.subscription_order.root, before.subscription_order.root);
      assert.deepEqual(after.subscription_order.folders[BETA], before.subscription_order.folders[BETA]);
      assert.deepEqual(after.articles, before.articles);
      await occurrence(h, TITLES[0], BETA); await occurrence(h, TITLES[0], GAMMA);
      await h.capture("23-cross-folder-order");
      await undo(h);
      await h.until(async () => (await h.state()).feeds.find(f => f.id === BASE).folders.includes(ALPHA), "Undo restores the moved occurrence");
      assert.deepEqual((await h.state()).subscription_order, before.subscription_order);
      assert.deepEqual((await h.state()).feeds, before.feeds);
    });

    await check("a library replacement during a native drag cannot reorder the replacement tree", async () => {
      await prepare();
      const replacement = structuredClone(data);
      replacement.subscription_order.root.reverse();
      const source = await folder(h, GAMMA), target = (await rows(h))[0];
      await drag(h, source, [target.x + target.width / 2, target.y + 3], async () => {
        await replace(h, replacement); await sleep(80);
      });
      assert.deepEqual((await h.state()).subscription_order, replacement.subscription_order);
      assert.deepEqual((await h.state()).feeds, replacement.feeds);
      await h.page.press("Escape");
    });

    await check("phone subscription and folder menus retain reachable 44-pixel ordering actions", async () => {
      await prepare(); await h.resize(320, 740); await navigate(h, "All");
      touch = await createTouch(h.page);
      await h.clickLabel("Show subscriptions");
      await h.clickName("SubscriptionSettings");
      const menu = await h.control(e => e.name === "ReaderMenu", "phone subscription settings");
      viewportBounds(menu, await h.snapshot());
      for (const name of ["Sort alphabetically", "Sort by drag and drop", "Show all", "Show updated"])
        assert((await itemState(h, name)).height >= 44, name + " has a touch-sized row");
      await h.capture("23-phone-subscription-settings"); await dismiss(h);
      const target = await folder(h, BETA);
      await touch.start(center(target)); await sleep(650); await touch.end();
      viewportBounds(await h.control(e => e.name === "ReaderMenu", "held folder ordering menu"), await h.snapshot());
      const down = await itemState(h, "Move down", {enabled: true});
      assert(down.height >= 44 && down.hit, "the held folder exposes a reachable move command");
      await h.capture("23-phone-folder-order"); await clickItem(h, "Move down");
      await h.until(async () => (await h.state()).subscription_order.root[2]?.Folder === BETA, "phone folder move changes root order");
      await touch.disable(); touch = null;
      await h.resize(1363, 960); await navigate(h, {Feed: BASE + 3});
      await h.clickName("SubscriptionSettings"); await dismiss(h);
      await keyboardContext(h);
      await itemState(h, "Sort by drag and drop", {checked: true}); await dismiss(h);
      await h.record("23-ordering-verification", {independentOccurrences: true, mixedRootOrder: true,
        alphabeticalPreservesManual: true, reloadVerified: true, otherLabelsRetained: true,
        dragGenerationGuard: true, keyboardTraversal: true, touchCommands: true});
    });
  } catch (error) {
    await h.capture("23-ordering-failure").catch(() => {}); throw error;
  } finally {
    await touch?.cancel(); await touch?.disable();
    await h.page.press("Escape").catch(() => {});
    await replace(h, baseline); await h.page.evaluate(`readerApp.navigate(${JSON.stringify(route)})`);
    await h.resize(viewport.width, viewport.height);
    await h.page.evaluate("readerApp.flushSave(); import('./host.js').then(module => module.flush())");
  }
}
