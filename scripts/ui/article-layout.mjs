import assert from "node:assert/strict";
import { readingFixture } from "./fixtures.mjs";
import { createTouch } from "./touch.mjs";

const replace = (h, data) => h.page.evaluate(`readerApp.importData(${JSON.stringify(JSON.stringify(data))})`);
const item = (view, name, id) => view.elements.find(e => e.name === name && (id == null || e.articleId === id));
const within = (e, parent) => e.x >= parent.x - 1 && e.y >= parent.y - 1
  && e.x + e.width <= parent.x + parent.width + 1 && e.y + e.height <= parent.y + parent.height + 1;
const rowChild = (view, name, row) => view.elements.find(e => e.name === name && within(e, row));

function fixture(baseline, density = "Comfortable", mode = "List") {
  const data = readingFixture(baseline);
  data.settings = {...data.settings, density, view_mode: mode, sort: "Newest", unread_only: false,
    mark_read_on_scroll: false, font_size: 14, send_to: []};
  data.articles[0].title = "A short headline";
  data.articles[0].summary = "The snippet follows the title in the same native line.";
  data.articles[1].title = "A long headline must leave room for its publication date and original link. ".repeat(8);
  return data;
}

async function prepare(h, data) {
  await replace(h, data);
  await h.page.evaluate('readerApp.navigate("\\"All\\"")');
  // Streams with no saved entry intentionally restore their own default mode.
  // Choose the fixture's presentation through the same command as the user.
  await h.click(e => /TextBlock$/.test(e.type) && e.text?.startsWith("All items (")
    && e.y > 110 && e.y < 180, "layout fixture stream title");
  await h.page.press(data.settings.view_mode === "List" ? "2" : "1");
  return h.until(async () => {
    const view = await h.snapshot();
    const first = item(view, data.settings.view_mode === "List" ? "ArticleRow" : "ArticleEntry", data.articles[0].id);
    return first && (data.settings.view_mode !== "List"
      || rowChild(view, "ArticleHeadlineText", first)?.text?.startsWith(data.articles[0].title)) && view;
  }, "article layout fixture is realized");
}

export async function articleLayoutChecks(h, check) {
  const baseline = await h.state(), initial = await h.snapshot();
  const route = await h.page.evaluate("readerApp.currentView()");
  let touch;
  try {
    await h.resize(1363, 936);
    await check("list headlines retain native layout and independent actions across expansion", async () => {
      const data = fixture(baseline), id = data.articles[0].id;
      let view = await prepare(h, data), row = item(view, "ArticleRow", id);
      assert(row, "the first list headline is visible");
      const headline = rowChild(view, "ArticleHeadlineText", row);
      assert.equal(headline.fontSize, 13);
      assert.equal(headline.text, `${data.articles[0].title} - ${data.articles[0].summary}`);
      assert.deepEqual(headline.headlineRuns.map(run => run.weight), [700, 400]);
      assert.equal(headline.headlineRuns[1].foreground, "#777777");
      for (const name of ["ArticleHeadlineSource", "ArticleHeadlineDate"])
        assert.equal(rowChild(view, name, row).fontSize, 13, `${name} uses the original headline font`);
      assert.equal(rowChild(view, "ArticleHeadlineSource", row).width, 143);
      const long = item(view, "ArticleRow", data.articles[1].id);
      for (const name of ["ArticleHeadlineText", "ArticleHeadlineDate", "ArticleHeadlineOriginal"])
        assert(rowChild(view, name, long), `${name} stays inside a long-title row`);
      const original = rowChild(view, "ArticleHeadlineOriginal", row);
      assert.equal(original.automationName, "View original article");
      assert(original.x >= row.x + row.width - 27, "the original-link target stays at the far right");
      await h.page.click(row.x + 13, row.y + row.height / 2);
      await h.until(async () => (await h.state()).articles.find(a => a.id === id)?.starred, "headline star toggles in place");
      assert.equal((await h.state()).articles.find(a => a.id === id).read, false);
      assert(!item(await h.snapshot(), "ArticleEntry", id), "starring does not expand the headline");
      await h.clickLabel(data.articles[0].title);
      view = await h.until(async () => {
        const next = await h.snapshot();
        return item(next, "ArticleEntry", id) && next;
      }, "the list item expands below its retained headline");
      const expanded = item(view, "ArticleEntry", id), retained = item(view, "ArticleRow", id);
      assert(retained && Math.abs(retained.y - row.y) < 1 && retained.width === row.width);
      assert(expanded.y === retained.y && expanded.height > retained.height);
      assert(!view.elements.some(e => e.name === "ArticleCard"), "list expansion has no expanded-view card margins");
      const edge = view.elements.find(e => e.name === "ArticleHeadlineEdge" && e.y === retained.y);
      assert.deepEqual(edge.border, {brush: "#4D90F0", left: 1, top: 0, right: 0, bottom: 0});
      const current = rowChild(view, "ArticleHeadlineText", retained);
      assert.equal(current.headlineRuns[0].weight, 400, "opened/read title returns to normal weight");
      const readState = view.elements.find(e => e.name === "ArticleReadState" && within(e, expanded));
      assert.equal(readState?.height, 17, "list actions retain inline density without a Material 48px layout column");
      const footer = view.elements.find(e => e.name === "ArticleActions" && within(e, expanded));
      assert.equal(footer?.height, 30, "list footer contains one native text line, its focus border and 6px vertical padding");
      const outline = view.elements.find(e => e.name === "ReaderReadStateOutline" && within(e, readState));
      assert(outline && outline.width === 13 && outline.height === 13, "the compact native checkbox uses the original-sized outline");
      await h.capture("16-list-expanded-headline");
      // The date/background is part of the same headline toggle, not an inert
      // strip or a replacement card. Native action buttons remain independent.
      const date = rowChild(view, "ArticleHeadlineDate", retained);
      await h.page.click(date.x + date.width / 2, date.y + date.height / 2);
      await h.until(async () => !item(await h.snapshot(), "ArticleEntry", id), "clicking the retained date collapses the item");
      await h.clickLabel(data.articles[0].title);
      await h.until(async () => item(await h.snapshot(), "ArticleEntry", id), "the headline can reopen");
      await h.clickLabel(data.articles[0].title);
      await h.until(async () => !item(await h.snapshot(), "ArticleEntry", id), "clicking the title collapses the item");
    });

    await check("density insets and the selected article edge match expanded view", async () => {
      const evidence = [];
      for (const [density, right, height] of [["Comfortable", 33, 37], ["Cozy", 21, 31], ["Compact", 10, 27]]) {
        const data = fixture(baseline, density), id = data.articles[0].id;
        let view = await prepare(h, data);
        assert.equal(item(view, "ArticleRow", id).height, height, `${density} headline height`);
        await h.page.press("1");
        await h.page.press("n");
        view = await h.until(async () => {
          const next = await h.snapshot();
          return item(next, "ArticleSelectionEdge") && next;
        }, `${density} expanded selection edge`);
        const list = item(view, "ArticleList"), card = item(view, "ArticleCard");
        const frame = item(view, "ArticleCardFrame"), edge = item(view, "ArticleSelectionEdge");
        assert(Math.abs(card.x - list.x - 5) < 1, "expanded cards retain the five-pixel left inset");
        assert(Math.abs(list.x + list.width - card.x - card.width - right) < 1,
          `${density} card uses its original right inset`);
        assert.deepEqual(frame.border, {brush: "#DDDDDD", left: 1, top: 1, right: 1, bottom: 1});
        assert.deepEqual(edge.border, {brush: "#4D90F0", left: 1, top: 0, right: 0, bottom: 0});
        evidence.push({density, headlineHeight: height, card, edge: edge.border, frame: frame.border});
        await h.capture(`16-density-${density.toLowerCase()}`);
      }
      await h.record("16-article-layout-evidence", evidence);
    });

    await check("small-phone list expansion retains touch targets and collapse behavior", async () => {
      touch = await createTouch(h.page);
      for (const width of [320, 390]) {
        await h.resize(width, 844);
        const data = fixture(baseline), id = data.articles[0].id;
        let view = await prepare(h, data), row = item(view, "ArticleRow", id);
        assert(row.height >= 44);
        const original = rowChild(view, "ArticleHeadlineOriginal", row);
        assert(original.width >= 44 && original.height >= 44);
        assert(rowChild(view, "ArticleHeadlineText", row));
        assert(!rowChild(view, "ArticleHeadlineSource", row), "phone rows reserve width for the title");
        const title = await h.control(e => /Button$/.test(e.type) && e.automationName === data.articles[0].title, "phone headline title");
        await touch.tapControl(title);
        view = await h.until(async () => {
          const next = await h.snapshot();
          return item(next, "ArticleEntry", id) && next;
        }, "phone headline opens on touch release");
        view = await h.until(async () => {
          const next = await h.snapshot(), list = item(next, "ArticleList");
          const expanded = item(next, "ArticleEntry", id);
          const bottom = expanded.y + expanded.height;
          assert(list.y + list.height - bottom >= 90,
            "the fixture leaves room for two following phone headlines");
          let expectedTop = bottom + 1; // The expanded item's outer separator.
          for (const following of data.articles.slice(1, 3)) {
            const headline = item(next, "ArticleRow", following.id);
            assert(headline && headline.hit && Math.abs(headline.y - expectedTop) <= 1
              && headline.y + headline.height <= list.y + list.height + 1,
            `following headline ${following.id} fills the visible space after expansion`);
            expectedTop = headline.y + headline.height + 1;
          }
          return next;
        }, `phone ${width}px expansion fills its remaining viewport`);
        row = item(view, "ArticleRow", id);
        assert(row && row.height >= 43, "expanded headline retains its touch-sized line and separator");
        const date = rowChild(view, "ArticleHeadlineDate", row);
        assert(date && date.x + date.width <= width);
        await h.capture(`16-list-phone-${width}`);
        await touch.tapControl(date);
        await h.until(async () => !item(await h.snapshot(), "ArticleEntry", id), "touching the retained date collapses the item");
      }
    });
  } catch (error) {
    await h.capture("16-article-layout-failure").catch(() => {});
    throw error;
  } finally {
    await touch?.cancel();
    await touch?.disable();
    await h.resize(initial.width, initial.height);
    await replace(h, baseline);
    await h.page.evaluate(`readerApp.navigate(${JSON.stringify(route)})`);
  }
}
