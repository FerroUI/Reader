import assert from "node:assert/strict";

const BASE = 480000;
const replace = (h, data) => h.page.evaluate(`readerApp.importData(${JSON.stringify(JSON.stringify(data))})`);
const first = view => view.elements.find(element => element.name === "ArticleBodyText"
  && element.text?.startsWith("Palette selection remains"));
const cells = view => view.elements.filter(element => element.name === "ArticleBodyText"
  && /^Palette table (heading|cell)/.test(element.text || ""));
const identity = element => ({...element.richText, links: element.links.map(link => link.id)});

function palette(element, body, link) {
  assert(element, "the native article paragraph remains realized");
  assert.equal(element.foreground, body, "native body foreground follows read/current state");
  assert(element.links.length, "the fixture contains a native hyperlink");
  for (const inline of element.links) {
    assert.equal(inline.foreground, link, "native hyperlink foreground follows read/current state");
    assert.equal(inline.underline, true, "body links retain the original underline");
  }
}

async function selectText(h, paragraph) {
  const y = paragraph.y + Math.min(12, paragraph.height / 2);
  await h.page.mouseDown(paragraph.x + 3, y);
  for (const distance of [35, 75, 135]) {
    await h.page.send("Input.dispatchMouseEvent", {type: "mouseMoved", pointerType: "mouse",
      x: paragraph.x + distance, y, buttons: 1});
  }
  await h.page.mouseUp(paragraph.x + 135, y);
  return h.until(async () => {
    const selection = first(await h.snapshot())?.textSelection;
    return selection?.start !== selection?.end && selection?.text?.length > 8 && selection;
  }, "a native article selection is established");
}

export async function articlePaletteChecks(h, check) {
  const baseline = await h.state(), viewport = await h.snapshot();
  const viewBefore = await h.page.evaluate("readerApp.currentView()");
  const evidence = [];
  try {
    await check("article palettes reconcile native links and table text without replacing selection", async () => {
      await h.resize(1363, 936);
      const data = {...structuredClone(baseline), folders: [], stream_settings: {}, next_id: BASE + 3};
      data.settings = {...data.settings, view_mode: "Expanded", unread_only: false,
        density: "Comfortable", sort: "Newest", font_size: 14, mark_read_on_scroll: false};
      data.feeds = [{id: BASE - 1, title: "Palette fixture", url: "https://palette.example/feed.xml",
        site_url: "https://palette.example/", fetched_at: 1795341600}];
      data.articles = [
        {id: BASE, feed_id: BASE - 1, guid: "palette-first", title: "Read article palette",
          url: "https://palette.example/first", published: 1795341600, read: true,
          content: '<p>Palette selection remains intact when the current article changes. <a href="https://palette.example/reference">Native link</a>.</p>' +
            '<table><tr><th>Palette table heading <a href="https://palette.example/header">Header link</a></th>' +
            '<td>Palette table cell <a href="https://palette.example/cell">Cell link</a></td></tr></table>'},
        {id: BASE + 1, feed_id: BASE - 1, guid: "palette-second", title: "Unread article palette",
          url: "https://palette.example/second", published: 1795341500, read: false,
          content: '<p>Unread article text keeps its full contrast and <a href="https://palette.example/next">blue link</a>.</p>'},
      ];
      await replace(h, data);
      await h.clickNavigation("All items");
      await h.page.press("1");
      const initial = await h.until(async () => {
        const view = await h.snapshot();
        return first(view) && cells(view).length === 2 && view;
      }, "both read article paragraphs and table cells are visible");
      palette(first(initial), "#222222", "#444444");
      for (const cell of cells(initial)) {
        palette(cell, "#222222", "#444444");
        assert.equal(cell.fontSize, data.settings.font_size,
          "table headings and ordinary cells inherit the configured body size");
      }
      const unread = initial.elements.find(element => element.name === "ArticleBodyText"
        && element.text?.startsWith("Unread article text"));
      palette(unread, "#000000", "#1155CC");
      const original = identity(first(initial));
      const tableIdentities = cells(initial).map(identity);
      const selection = await selectText(h, first(initial));

      const expect = (name, body, link) => h.until(async () => {
        const view = await h.snapshot(), paragraph = first(view), table = cells(view);
        palette(paragraph, body, link);
        assert.deepEqual(identity(paragraph), original, "the native control, paragraph, and link retain their identities");
        assert.deepEqual(paragraph.textSelection, selection, "the user's exact native text selection survives appearance changes");
        assert.equal(table.length, 2, "both table cells remain visible");
        table.forEach(cell => {
          palette(cell, body, link);
          assert.equal(cell.fontSize, data.settings.font_size,
            "table headings retain body sizing across read/current transitions");
        });
        assert.deepEqual(table.map(identity), tableIdentities, "table paragraphs also reconcile in place");
        evidence.push({name, foreground: paragraph.foreground, links: paragraph.links,
          identity: identity(paragraph), selection: paragraph.textSelection,
          table: table.map(cell => ({fontSize: cell.fontSize, foreground: cell.foreground,
            links: cell.links, identity: identity(cell)}))});
        return true;
      }, name);

      // Dragging selects the first article through native pointer routing.
      await expect("read current article", "#000000", "#1155CC");
      await h.page.press("n");
      await expect("read inactive article", "#222222", "#444444");
      await h.page.press("p");
      await expect("read article becomes current again", "#000000", "#1155CC");
      await h.page.press("m");
      await h.until(async () => !(await h.state()).articles.find(article => article.id === BASE).read,
        "the current article is kept unread");
      await expect("unread current article", "#000000", "#1155CC");
      await h.page.press("n");
      await expect("unread inactive article", "#000000", "#1155CC");
      await h.page.press("p");
      await h.page.press("m");
      await h.until(async () => (await h.state()).articles.find(article => article.id === BASE).read,
        "the article is marked read again");
      await h.page.press("n");
      await expect("read inactive article after toggling read state", "#222222", "#444444");
      await h.capture("16-article-palette");
      await h.record("16-article-palette-evidence", evidence);
    });
  } catch (error) {
    await h.capture("16-article-palette-failure").catch(() => {});
    throw error;
  } finally {
    await replace(h, baseline);
    await h.page.evaluate(`readerApp.navigate(${JSON.stringify(viewBefore)})`);
    await h.resize(viewport.width, viewport.height);
    await h.page.evaluate("readerApp.flushSave(); import('./host.js').then(module => module.flush())");
  }
}
