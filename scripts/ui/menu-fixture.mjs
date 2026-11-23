import assert from "node:assert/strict";

export const FEED = 720000;
export const menuItem = (element, label) => /MenuItem$/.test(element.type) && element.automationName === label;
export const replace = (h, data) => h.page.evaluate("readerApp.importData(" + JSON.stringify(JSON.stringify(data)) + ")");
export const go = h => h.page.evaluate("readerApp.navigate(" + JSON.stringify(JSON.stringify({Feed: FEED})) + ")");
export const getMenu = h => h.control(element => element.name === "ReaderMenu", "the native Reader dropdown");
export const clickItem = (h, label) => h.click(element => menuItem(element, label), label);

export async function itemState(h, label, expected = {}) {
  return h.until(async () => {
    const item = (await h.snapshot()).elements.find(element => menuItem(element, label));
    assert(item, label + " is present");
    for (const [key, value] of Object.entries(expected)) assert.equal(item[key], value, label + ": " + key);
    return item;
  }, "native menu state: " + label);
}

export async function closed(h) {
  await h.until(() => h.page.evaluate("!readerApp.hasOverlay()"), "the native menu closes");
}

export async function dismiss(h) {
  await h.page.press("Escape");
  await closed(h);
}

export async function focusStream(h) {
  await h.click(element => /TextBlock$/.test(element.type) && element.text?.startsWith("Menu source")
    && element.x > 200 && element.y < 180, "the non-interactive menu fixture stream title");
}

export function fixture(baseline, count = 1, empty = false) {
  const data = structuredClone(baseline), now = Math.floor(Date.now() / 1000);
  data.feeds = Array.from({length: count}, (_, index) => ({
    id: FEED + index, title: index ? "Menu source " + String(index).padStart(2, "0") + " with a long subscription title for constrained native menus" : "Menu source",
    url: "https://menus.invalid/feed/" + index, site_url: "https://menus.invalid/",
    folder: "", folders: [], description: "Native dropdown fixture", fetched_at: now, error: null, sample: false,
  }));
  data.articles = empty ? [] : [false, true].map((read, index) => ({
    id: FEED + 100 + index, feed_id: FEED, guid: "menu-item-" + index,
    title: "Menu article " + index, url: "https://menus.invalid/articles/" + index,
    content: "<p>Menu article body retains its text size and read state while a dropdown is open.</p>",
    summary: "Native dropdown verification", author: "", published: now - index,
    read, starred: false, tags: [], note: "", read_at: read ? now : null,
  }));
  data.folders = []; data.stream_settings = {}; data.next_id = FEED + 200;
  Object.assign(data.settings, {density: "Comfortable", view_mode: "Expanded", font_size: 14,
    unread_only: false, sort: "Newest", mark_read_on_scroll: false, send_to: []});
  return data;
}

export function inside(element, container) {
  return element.x >= container.x - 1 && element.y >= container.y - 1
    && element.x + element.width <= container.x + container.width + 1
    && element.y + element.height <= container.y + container.height + 1;
}

export function viewportBounds(menu, view) {
  assert(inside(menu, {x: 0, y: 0, width: view.width, height: view.height}), "the popup stays inside the viewport");
  assert(menu.width <= view.width - 15 && menu.height <= view.height - 15, "the popup respects its viewport constraints");
}

export async function palette(h, label, color, highlighted = false) {
  return h.until(async () => {
    const view = await h.snapshot(), item = view.elements.find(element => menuItem(element, label));
    assert(item, "the menu item is visible");
    const text = view.elements.find(element => /(?:TextBlock|AccessText)$/.test(element.type)
      && element.text === label && inside(element, item));
    assert.equal(text?.foreground, color, "the native menu text uses Reader's foreground");
    if (highlighted) {
      const layer = view.elements.find(element => element.name === "StateLayer"
        && Math.abs(element.x - item.x) <= 1 && Math.abs(element.y - item.y) <= 1
        && Math.abs(element.width - item.width) <= 1 && Math.abs(element.height - item.height) <= 1);
      assert.equal(layer?.background, "#EEEEEE", "the native focus/hover layer is neutral gray");
      assert.equal(layer?.opacity, 1, "the native highlight is not a Material tint overlay");
    }
    return true;
  }, "Reader menu palette: " + label);
}
