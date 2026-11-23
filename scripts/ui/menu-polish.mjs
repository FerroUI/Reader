import assert from "node:assert/strict";
import { createTouch } from "./touch.mjs";
import { fixture, replace, go, getMenu, clickItem, itemState, closed, dismiss,
  focusStream, inside, viewportBounds, palette } from "./menu-fixture.mjs";

export async function menuPolishChecks(h, check) {
  const baseline = await h.state(), route = await h.page.evaluate("readerApp.currentView()");
  let touch, passed = false;
  try {
    await h.resize(1363, 936);
    await replace(h, fixture(baseline)); await go(h);
    await check("Reader dropdowns preserve anchors, exclusive choices and native keyboard focus", async () => {
      const anchor = await h.control(element => element.name === "StreamFilter", "the stream filter anchor");
      // Re-select twice: the second save is a true model no-op and must not
      // leave the native checked property out of sync with the saved setting.
      for (let attempt = 0; attempt < 2; attempt++) {
        await h.clickName("StreamFilter");
        const menu = await getMenu(h);
        assert(Math.abs(menu.x - anchor.x) <= 1, "the filter opens at its control's left edge");
        assert(Math.abs(menu.y - anchor.y - anchor.height) <= 1, "the filter opens directly below its control");
        await itemState(h, "All items", {checked: true});
        await itemState(h, "New items", {checked: false});
        await clickItem(h, "All items"); await closed(h);
      }
      await h.clickName("StreamFilter");
      await itemState(h, "All items", {checked: true});
      await dismiss(h);
      await h.until(async () => (await h.snapshot()).elements.some(element => element.name === "StreamFilter" && element.focused),
        "Escape restores focus to the dropdown button");
      await h.page.press("ArrowDown");
      await itemState(h, "All items", {selected: true, focused: true});
      await palette(h, "All items", "#333333", true);
      await h.page.press("ArrowDown");
      await itemState(h, "New items", {selected: true, focused: true});
      await palette(h, "New items", "#333333", true);
      await h.page.press("Enter"); await closed(h);
      assert.equal((await h.state()).settings.unread_only, true);
      await h.clickName("StreamFilter");
      await itemState(h, "All items", {checked: false});
      await itemState(h, "New items", {checked: true});
      await dismiss(h);
      await h.page.press("ArrowUp");
      await itemState(h, "New items", {selected: true, focused: true});
      await h.capture("20-menu-keyboard");
      await clickItem(h, "All items"); await closed(h);
    });

    await check("right-edge menus clamp correctly and article-size commands expose native disabled states", async () => {
      const anchor = await h.control(element => /Button$/.test(element.type) && element.automationName === "Settings", "the gear anchor");
      await h.clickLabel("Settings");
      const menu = await getMenu(h), view = await h.snapshot();
      viewportBounds(menu, view);
      assert(Math.abs(menu.x + menu.width - anchor.x - anchor.width) <= 1,
        "the right-side gear flips horizontal alignment when start alignment would overflow");
      await itemState(h, "Comfortable", {checked: true});
      const cozy = await itemState(h, "Cozy");
      await h.page.send("Input.dispatchMouseEvent", {type: "mouseMoved", pointerType: "mouse",
        x: cozy.x + cozy.width / 2, y: cozy.y + cozy.height / 2});
      await palette(h, "Cozy", "#333333", true);
      await h.capture("20-menu-settings");
      await dismiss(h);

      await h.clickLabel("Feed settings");
      await itemState(h, "Larger article text (+)", {enabled: false});
      const disabled = await itemState(h, "Smaller article text (-)", {enabled: false});
      await palette(h, "Smaller article text (-)", "#CCCCCC");
      await h.page.click(disabled.x + disabled.width / 2, disabled.y + disabled.height / 2);
      await getMenu(h);
      await dismiss(h);

      await focusStream(h); await h.page.press("n");
      // Reset transient magnification from any earlier integration module.
      for (let index = 0; index < 5; index++) await h.page.press("-");
      await h.clickLabel("Feed settings");
      await itemState(h, "Larger article text (+)", {enabled: true});
      await itemState(h, "Smaller article text (-)", {enabled: false});
      await clickItem(h, "Larger article text (+)"); await closed(h);
      await h.until(async () => (await h.snapshot()).elements.some(element => element.name === "ArticleBodyText"
        && element.text?.startsWith("Menu article body") && Math.abs(element.fontSize - 15.4) < 0.01),
        "the menu uses the same current-article scale as the keyboard");
      assert.equal((await h.state()).settings.font_size, 14, "transient article magnification preserves the saved body size");
      await h.clickLabel("Feed settings");
      await itemState(h, "Smaller article text (-)", {enabled: true});
      await clickItem(h, "Smaller article text (-)"); await closed(h);
    });

    await check("long touch menus scroll within the phone and keyboard selection reaches clipped choices", async () => {
      await replace(h, fixture(baseline, 24)); await go(h);
      await h.resize(390, 640); touch = await createTouch(h.page);
      const anchor = await h.control(element => /Button$/.test(element.type) && element.automationName === "Search scope", "touch search scope");
      assert(anchor.width >= 44 && anchor.height >= 44);
      await touch.tapControl(anchor);
      const menu = await getMenu(h), view = await h.snapshot();
      viewportBounds(menu, view);
      const visible = view.elements.filter(element => /MenuItem$/.test(element.type) && element.hit);
      assert(visible.length > 4 && visible.every(element => element.height >= 44), "visible native menu choices have 44px touch targets");
      const scroll = view.elements.find(element => element.name === "MenuFlyoutPresenterScrollViewer" && element.scroll);
      assert(scroll?.scroll.extentHeight > scroll.scroll.viewportHeight + 100, "the long menu uses its native vertical scroll viewer");
      const articles = (await h.state()).articles;
      await touch.drag([menu.x + menu.width / 2, menu.y + menu.height - 35],
        [menu.x + menu.width / 2, menu.y + 45]);
      await h.until(async () => (await h.snapshot()).elements.some(element => element.name === "MenuFlyoutPresenterScrollViewer"
        && element.scroll?.offsetY > 30), "the native menu consumes a real touch scroll");
      assert.deepEqual((await h.state()).articles, articles, "scrolling choices does not trigger article commands");
      await dismiss(h);
      await h.page.press("ArrowUp");
      const label = fixture(baseline, 24).feeds.at(-1).title;
      const last = await itemState(h, label, {selected: true, focused: true, hit: true});
      const scrolled = (await h.snapshot()).elements.find(element => element.name === "MenuFlyoutPresenterScrollViewer" && element.scroll);
      assert(inside(last, scrolled), "native keyboard navigation scrolls the final choice fully into view");
      await h.capture("20-menu-phone-scroll");
      await h.until(() => h.page.evaluate("history.state?.reader?.overlay === true"), "the menu registers a browser Back boundary");
      const history = await h.page.send("Page.getNavigationHistory", {});
      await h.page.send("Page.navigateToHistoryEntry", {entryId: history.entries[history.currentIndex - 1].id});
      await closed(h);
      assert.deepEqual((await h.state()).articles, articles);
    });

    await check("mobile More actions keeps independent radio groups and skips disabled commands", async () => {
      await replace(h, fixture(baseline, 1, true)); await go(h);
      await h.clickLabel("More actions");
      await itemState(h, "Expanded view", {checked: true});
      await itemState(h, "Sort by newest", {checked: true});
      await itemState(h, "Previous item", {enabled: false});
      await itemState(h, "Next item", {enabled: false});
      await palette(h, "Previous item", "#CCCCCC");
      await clickItem(h, "List view"); await closed(h);
      await h.clickLabel("More actions");
      await itemState(h, "List view", {checked: true});
      await itemState(h, "Expanded view", {checked: false});
      await itemState(h, "Sort by newest", {checked: true});
      await clickItem(h, "Sort by oldest"); await closed(h);
      await h.clickLabel("More actions");
      await itemState(h, "List view", {checked: true});
      await itemState(h, "Sort by oldest", {checked: true});
      await dismiss(h);
      await h.page.press("ArrowDown");
      await itemState(h, "List view", {selected: true});
      await h.page.press("ArrowDown");
      await itemState(h, "Expanded view", {selected: true});
      await h.page.press("ArrowDown");
      await itemState(h, "Mark all as read", {selected: true, focused: true});
      assert(!(await h.snapshot()).elements.some(element => /MenuItem$/.test(element.type) && !element.enabled && element.selected),
        "native arrow navigation skips disabled Previous and Next commands");
      await h.capture("20-menu-phone-disabled");
      await dismiss(h);
    });
    passed = true;
  } finally {
    await touch?.disable();
    if (passed) {
      await h.resize(1363, 936);
      await replace(h, baseline);
      await h.page.evaluate("readerApp.navigate(" + JSON.stringify(route) + ")");
    }
  }
}
