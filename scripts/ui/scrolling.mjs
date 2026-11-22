import assert from "node:assert/strict";
import http from "node:http";
import { deflateSync } from "node:zlib";
import { createTouch } from "./touch.mjs";

const BASE = 240000, COUNT = 1200, DELAYED = 1103;
const replace = (h, data) => h.page.evaluate(`readerApp.importData(${JSON.stringify(JSON.stringify(data))})`);
const entries = view => view.elements.filter(element => element.name === "ArticleEntry");
const entry = (view, id) => entries(view).find(element => element.articleId === id);
const stream = view => view.elements.find(element => element.name === "ArticleList");
const bounded = view => {
  assert(view.articleContainers > 0 && view.articleContainers < 80,
    `${view.articleContainers} live/pooled article controls for ${COUNT} mixed-height articles`);
  assert(view.visualCount < 9000, `${view.visualCount} retained visuals after heterogeneous scrolling`);
  assert(Number.isFinite(view.articleScroll.extentHeight) && Number.isFinite(view.articleScroll.offsetY));
};

async function settled(h) {
  let previous, stable = 0;
  return h.until(async () => {
    const view = await h.snapshot();
    const offset = view.articleScroll?.offsetY;
    stable = Math.abs(offset - previous) < 0.2 ? stable + 1 : 0;
    previous = offset;
    return stable >= 3 && view;
  }, "article layout and scroll anchor settle");
}

async function seek(h, index) {
  await h.page.evaluate(`readerApp.scrollToArticle(${BASE + index})`);
  await h.until(async () => entry(await h.snapshot(), BASE + index), `far article ${index} is realized`);
  const view = await settled(h);
  bounded(view);
  const article = entry(view, BASE + index), list = stream(view);
  assert(article && article.y < list.y + list.height && article.y + article.height > list.y,
    `requested article ${index} intersects its actual viewport`);
  return view;
}

function png(width, height) {
  const raw = Buffer.alloc((width * 4 + 1) * height);
  for (let y = 0; y < height; y++) {
    for (let x = 0; x < width; x++) {
      const offset = y * (width * 4 + 1) + 1 + x * 4;
      raw.set(y % 80 < 40 ? [73, 120, 171, 255] : [220, 232, 240, 255], offset);
    }
  }
  const chunk = (name, data) => {
    const type = Buffer.from(name), body = Buffer.concat([type, data]);
    let crc = 0xffffffff;
    for (const byte of body) {
      crc ^= byte;
      for (let bit = 0; bit < 8; bit++) crc = (crc >>> 1) ^ (crc & 1 ? 0xedb88320 : 0);
    }
    const size = Buffer.alloc(4), checksum = Buffer.alloc(4);
    size.writeUInt32BE(data.length); checksum.writeUInt32BE((crc ^ 0xffffffff) >>> 0);
    return Buffer.concat([size, body, checksum]);
  };
  const header = Buffer.alloc(13);
  header.writeUInt32BE(width); header.writeUInt32BE(height, 4); header[8] = 8; header[9] = 6;
  return Buffer.concat([Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]),
    chunk("IHDR", header), chunk("IDAT", deflateSync(raw)), chunk("IEND", Buffer.alloc(0))]);
}

async function imageFixture() {
  const image = png(320, 800), waiting = [];
  let released = false, delayedRequests = 0;
  const respond = response => {
    response.writeHead(200, {"content-type": "image/png", "content-length": image.length,
      "access-control-allow-origin": "*", "cache-control": "no-store"});
    response.end(image);
  };
  const server = http.createServer((request, response) => {
    if (request.url === `/image-${DELAYED}.png` && !released) {
      delayedRequests++; waiting.push(response);
    } else respond(response);
  });
  await new Promise((resolve, reject) => {
    server.once("error", reject);
    server.listen(0, "127.0.0.1", resolve);
  });
  return { origin: `http://127.0.0.1:${server.address().port}`,
    requested: () => delayedRequests,
    release() { released = true; waiting.splice(0).forEach(respond); },
    async close() {
      released = true; waiting.splice(0).forEach(response => response.destroy());
      server.closeAllConnections();
      await new Promise(resolve => server.close(resolve));
    } };
}

function body(index, origin) {
  const paragraph = n => `<p>Mixed paragraph ${index}.${n}: ` +
    "Reader preserves the current reading position while articles wrap, expand and load images. ".repeat(5) + "</p>";
  if (index === DELAYED) return `<p>Before delayed image ${index}.</p><img src="${origin}/image-${index}.png" alt="Delayed tall image">` +
    Array.from({length: 24}, (_, n) => paragraph(n)).join("");
  switch (index % 6) {
    case 0: return `<p>Brief item ${index}.</p>`;
    case 1: return Array.from({length: 5}, (_, n) => paragraph(n)).join("");
    case 2: return Array.from({length: 30}, (_, n) => paragraph(n)).join("");
    case 3: return paragraph(0) + `<pre>${Array.from({length: 36}, (_, n) =>
      `let sample_${n} = "${"long_source_line_".repeat(14)}";`).join("\n")}</pre>` + paragraph(1);
    case 4: return `<img src="${origin}/known.png" width="320" height="800" alt="Known tall image">` + paragraph(0);
    default: return Array.from({length: 12}, (_, n) => paragraph(n)).join("");
  }
}

export async function scrollingChecks(h, check) {
  const baseline = await h.state(), images = await imageFixture();
  let passed = false, touch;
  try {
    await check("mixed-height articles keep bounded realization and exact far targets", async () => {
      await h.resize(1363, 936);
      const data = structuredClone(baseline);
      data.articles = Array.from({length: COUNT}, (_, index) => ({
        id: BASE + index, feed_id: data.feeds[0].id, guid: `mixed-${index}`,
        title: `Heterogeneous article ${index}`, url: `https://scroll-fixture.example/articles/${index}`,
        author: "Scrolling fixture", content: body(index, images.origin), summary: `Mixed content ${index}`,
        published: 1795341600 - index, read: false, starred: false, tags: [], note: "", read_at: null,
      }));
      data.next_id = BASE + COUNT + 1;
      data.settings.view_mode = "Expanded"; data.settings.unread_only = false;
      data.settings.mark_read_on_scroll = false; data.stream_settings = {};
      await replace(h, data);
      await h.clickNavigation("All items");
      await h.page.press("1");
      await seek(h, 1000);
      await seek(h, 37);
      await seek(h, 1002);
      const before = (await h.snapshot()).articleScroll.offsetY;
      for (const deltaY of [900, 1300, 700, 1100]) {
        await h.page.send("Input.dispatchMouseEvent", {type: "mouseWheel", x: 950, y: 500,
          deltaX: 0, deltaY, pointerType: "mouse"});
      }
      const after = await settled(h);
      assert(after.articleScroll.offsetY > before + 300, "rapid wheel input moves the native article viewport");
      bounded(after);
      await h.capture("15-mixed-height-stream");
    });

    await check("read/star updates preserve the visible article and keyboard seeks its neighbor", async () => {
      await seek(h, 602);
      const paragraph = await h.control(element => element.name === "ArticleBodyText"
        && element.text?.startsWith("Mixed paragraph 602.0:"), "article paragraph to select");
      await h.page.click(paragraph.x + 15, paragraph.y + Math.min(12, paragraph.height / 2));
      await h.until(async () => (await h.state()).articles.find(article => article.id === BASE + 602).read, "reading selects the requested article");
      const before = await settled(h), y = entry(before, BASE + 602).y;
      await h.page.press("s");
      await h.until(async () => (await h.state()).articles.find(article => article.id === BASE + 602).starred, "star toggles in place");
      await h.until(async () => {
        const view = await h.snapshot(), article = entry(view, BASE + 602);
        return article && view.elements.some(element => element.automationName === "Remove star"
          && element.y >= article.y && element.y < article.y + 120);
      }, "the retained item template renders its updated star action");
      await h.page.press("m");
      await h.until(async () => !(await h.state()).articles.find(article => article.id === BASE + 602).read, "read toggles in place");
      const after = await settled(h);
      assert(Math.abs(entry(after, BASE + 602).y - y) <= 1.5, "annotation updates retain the article's pixel anchor");
      assert(Math.abs(after.articleScroll.offsetY - before.articleScroll.offsetY) <= 1.5, "annotation updates retain the scroll offset");
      assert(Math.abs(after.articleScroll.extentHeight - before.articleScroll.extentHeight) <= 1.5, "annotation updates retain measured extent");
      await h.page.press("n");
      await h.until(async () => entry(await h.snapshot(), BASE + 603), "keyboard scanning brings the long-code neighbor into view");
      assert(!(await h.state()).articles.find(article => article.id === BASE + 603).read, "n scans without reading");
      await h.page.press("j");
      await h.until(async () => (await h.state()).articles.find(article => article.id === BASE + 604).read, "j selects the next image article");
      bounded(await settled(h));
    });

    await check("late image decoding preserves the paragraph inside a tall article", async () => {
      await seek(h, DELAYED);
      await h.until(() => images.requested() > 0, "unknown image waits for its real dimensions");
      await h.page.wheel(950, 500, 0, 660);
      const before = await settled(h), list = stream(before), article = entry(before, BASE + DELAYED);
      const paragraph = before.elements.find(element => element.name === "ArticleBodyText"
        && element.text?.startsWith(`Mixed paragraph ${DELAYED}.`) && element.y >= list.y + 8 && element.y < list.y + 240);
      assert(paragraph && article, "a paragraph after the pending image is visible");
      images.release();
      await h.until(async () => {
        const current = entry(await h.snapshot(), BASE + DELAYED);
        return current && current.height > article.height + 400;
      }, "decoded tall image changes actual article height");
      const after = await settled(h);
      const anchored = after.elements.find(element => element.name === "ArticleBodyText" && element.text === paragraph.text);
      assert(anchored && Math.abs(anchored.y - paragraph.y) <= 3,
        `reading paragraph moved after image decode: ${paragraph.y} -> ${anchored?.y}`);
      bounded(after);
      await h.capture("15-image-scroll-anchor");
    });

    await check("phone resize and touch retain long-article position without opening another item", async () => {
      await seek(h, 602);
      await h.page.wheel(950, 500, 0, 420);
      touch = await createTouch(h.page);
      await h.page.evaluate("readerApp.setTouchMode(true)");
      await h.resize(390, 844);
      const resized = await settled(h);
      assert(entry(resized, BASE + 602), "resizing preserves the article identity");
      bounded(resized);
      const readCount = (await h.state()).articles.filter(article => article.read).length;
      const from = [350, 650], to = [350, 240];
      await touch.start(from); await touch.travel(from, to); await touch.end();
      const after = await settled(h);
      assert(after.articleScroll.offsetY > resized.articleScroll.offsetY + 30, "native touch scrolls nonuniform article content");
      assert.equal((await h.state()).articles.filter(article => article.read).length, readCount, "touch movement does not select another article");
      bounded(after);
      await h.capture("15-phone-long-article");
    });
    passed = true;
  } finally {
    images.release();
    if (touch) { await touch.cancel(); await touch.disable(); }
    await h.page.evaluate("readerApp.setTouchMode(false)");
    await images.close();
    if (passed) {
      await h.resize(1363, 936);
      await replace(h, baseline);
      await h.clickNavigation("All items");
    }
  }
}
