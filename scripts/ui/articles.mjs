import assert from "node:assert/strict";
import http from "node:http";
import { articleLink, articleSelection } from "./article-interactions.mjs";

const replace = (h, data) => h.page.evaluate(`readerApp.importData(${JSON.stringify(JSON.stringify(data))})`);
const flush = h => h.page.evaluate("readerApp.flushSave(); import('./host.js').then(m => m.flush())");
const bodyText = view => view.elements.filter(e => e.name === "ArticleBodyText").map(e => e.text || "").join("\n");

async function publisher() {
  const requests = [], pending = [];
  let mode = "article";
  const prose = "A native reading view preserves the article's meaning and useful details. The original feed summary remains available, and the saved publisher copy can be read without downloading the page again. ";
  const document = '<!doctype html><html><head><title>Reading the publisher article</title><base href="/journal/"></head><body>' +
    '<nav>Unrelated publisher navigation</nav><article><h1>Reading the publisher article</h1>' +
    '<p>Publisher body marker. ' + prose + '</p>' +
    '<table><thead><tr><th>Feature</th><th>Result</th></tr></thead><tbody><tr><td rowspan="2">Saved article</td><td>Available offline</td></tr><tr><td>Feed copy retained</td></tr></tbody></table>' +
    '<pre><code>fn read_article() {\n    let message = &quot;' + 'preserved code indentation and horizontal scrolling; '.repeat(12) + '&quot;;\n}</code></pre>' +
    '<blockquote><p>Quoted publisher text remains distinct.</p></blockquote><ul><li>Outer item<ul><li>Nested item</li></ul></li></ul>' +
    '<p>Before the <a href="details#reference">publisher reference link</a> after the link.</p><img src="pixel.png" alt="Publisher image" width="600" height="300">' +
    '<p>' + prose.repeat(5) + '</p><script>globalThis.publisherExecution = true</script></article><footer>Unrelated footer</footer></body></html>';
  const send = (response, type, body) => {
    response.writeHead(200, {"content-type": type, "cache-control": "no-store", "access-control-allow-origin": "*"});
    response.end(body);
  };
  const server = http.createServer((request, response) => {
    const pathname = new URL(request.url, "http://fixture.invalid").pathname;
    requests.push({pathname, cookie: request.headers.cookie || "", authorization: request.headers.authorization || ""});
    if (pathname === "/article") {
      if (mode === "hold") { pending.push(response); return; }
      send(response, "text/html; charset=utf-8", mode === "empty" ? "<html><body>Empty</body></html>" : document);
    } else if (pathname === "/feed.xml") {
      send(response, "application/rss+xml", '<rss version="2.0"><channel><title>Publisher reading fixture</title>' +
        '<link>' + origin + '/</link><description>Article integration fixture</description><item><guid>publisher-reading</guid>' +
        '<title>Reading the publisher article</title><link>' + origin + '/article</link><description><![CDATA[' +
        '<p>Feed summary marker. A <strong>formatted summary</strong> remains available before, during, and after publisher reading.</p>' +
        '<audio src="' + origin + '/journal/audio.mp3"></audio>]]></description></item></channel></rss>');
    } else if (pathname === "/journal/pixel.png") {
      send(response, "image/png", Buffer.from("iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNk+A8AAQUBAScY42YAAAAASUVORK5CYII=", "base64"));
    } else if (pathname === "/journal/details") {
      send(response, "text/html", "<!doctype html><title>Publisher reference fixture</title><p id='reference'>Opened from a native Reader inline.</p>");
    } else {
      response.writeHead(404); response.end();
    }
  });
  await new Promise((resolve, reject) => {
    server.once("error", reject);
    server.listen(0, "127.0.0.1", () => { server.off("error", reject); resolve(); });
  });
  const origin = `http://127.0.0.1:${server.address().port}`;
  const release = () => {
    mode = "article";
    for (const response of pending.splice(0)) if (!response.destroyed) send(response, "text/html", document);
  };
  return {
    origin, requests, count: () => requests.filter(request => request.pathname === "/article").length,
    setMode: value => { mode = value; }, release,
    async close() { release(); server.closeAllConnections(); await new Promise(resolve => server.close(resolve)); },
  };
}

async function top(h) {
  const view = await h.snapshot();
  await h.page.wheel(view.width - 45, Math.min(460, view.height - 80), 0, -20000);
  await h.until(async () => (await h.snapshot()).articleScroll.offsetY < 1, "reading position returns to article header");
}

async function reveal(h, name) {
  for (let step = 0; step < 8; step++) {
    const view = await h.snapshot();
    const found = view.elements.find(e => e.name === name && e.y >= 175 && e.y + e.height < view.height - 40);
    if (found) return found;
    await h.page.wheel(view.width - 45, Math.min(460, view.height - 80), 0, 160);
  }
  throw new Error(`Could not reveal ${name} in the native article`);
}

export async function articleChecks(h, check) {
  const baseline = await h.state(), network = await publisher();
  let passed = false, articleId, cached, selection;
  try {
    await check("publisher article downloads into native selectable reading controls", async () => {
      const data = {...structuredClone(baseline), feeds: [], articles: [], folders: [], stream_settings: {}};
      data.settings = {...data.settings, view_mode: "Expanded", unread_only: false, mark_read_on_scroll: false, feed_connection: "Direct"};
      await replace(h, data);
      await h.clickNavigation("Home");
      await h.clickName("SubscribeButton");
      await h.edit("DialogInput", `${network.origin}/feed.xml`, true);
      const article = await h.until(async () => (await h.state()).articles.find(item => item.guid === "publisher-reading"), "publisher fixture subscription");
      articleId = article.id;
      await h.page.press("1");
      await h.until(async () => bodyText(await h.snapshot()).includes("Feed summary marker."), "formatted feed summary appears natively");
      assert(bodyText(await h.snapshot()).includes("Listen to audio"), "embedded feed audio has a native safe link fallback");
      assert.equal(network.count(), 0, "ordinary feed reading does not download publisher pages");
      await h.clickLabel("Read full article");
      cached = await h.until(async () => {
        const item = (await h.state()).articles.find(item => item.id === articleId);
        return item?.show_full_content && item.full_content;
      }, "Rust extraction returns a saved publisher copy", 30000);
      assert.equal(cached.source_url, `${network.origin}/article`);
      assert.equal(cached.base_url, `${network.origin}/journal/`);
      assert(cached.html.includes(`${network.origin}/journal/details`));
      assert(cached.html.includes("Available offline"));
      assert(!cached.html.includes("<script"));
      await h.until(async () => bodyText(await h.snapshot()).includes("Publisher body marker."), "publisher text is rendered by native controls");
      selection = await articleSelection(h);
      const native = await h.snapshot();
      const nativeEntry = native.elements.find(e => e.name === "ArticleEntry" && e.articleId === articleId);
      for (const name of ["ArticleBody", "ArticleReadingControls"]) {
        const content = native.elements.find(e => e.name === name);
        assert(content && nativeEntry && content.x >= nativeEntry.x && content.x <= nativeEntry.x + 24,
          `${name} keeps the original left-aligned reading column`);
      }
      await reveal(h, "ArticleTableScroll");
      assert(bodyText(await h.snapshot()).includes("Available offline"), "table cells retain their readable values");
      const code = await reveal(h, "ArticleCodeScroll");
      assert(code.scroll.extentWidth > code.scroll.viewportWidth + 100, "long code preserves its indentation in a horizontally scrollable viewport");
      await h.page.wheel(code.x + code.width / 2, code.y + code.height / 2, 220, 0);
      await h.until(async () => (await h.snapshot()).elements.some(e => e.name === "ArticleCodeScroll" && e.scroll.offsetX > 10), "native code horizontal scrolling");
      await h.capture("14-article-native");
      assert.equal(await h.page.evaluate("!!globalThis.publisherExecution || !!document.querySelector('article,iframe,video')"), false,
        "publisher content must not execute or replace the native surface with a DOM page");
      await articleLink(h, `${network.origin}/journal/details#reference`);
      await top(h);
      const count = network.count();
      await h.clickLabel("Show feed content");
      await h.until(async () => bodyText(await h.snapshot()).includes("Feed summary marker."), "original feed copy remains readable");
      await h.clickLabel("Read full article");
      await h.until(async () => bodyText(await h.snapshot()).includes("Publisher body marker."), "saved full copy opens immediately");
      assert.equal(network.count(), count, "switching between feed and saved publisher content does not fetch again");
    });

    await check("article cancellation and publisher errors preserve the saved copy", async () => {
      network.setMode("hold");
      const before = network.count();
      await h.clickLabel("Reload full article");
      await h.until(() => network.count() > before, "reload reaches the held publisher");
      await h.clickLabel("Cancel article download");
      await h.control(e => e.automationName === "Reload full article", "cancel restores reading actions");
      network.release();
      assert.deepEqual((await h.state()).articles.find(item => item.id === articleId).full_content, cached);
      network.setMode("empty");
      await h.clickLabel("Reload full article");
      await h.control(e => /TextBlock$/.test(e.type) && /No readable article|too little article text/.test(e.text || ""), "publisher extraction failure is shown inline");
      const item = (await h.state()).articles.find(item => item.id === articleId);
      assert(item.show_full_content && item.full_content.html === cached.html, "a failed reload preserves the last readable copy");
      await h.capture("14-article-error");
      network.setMode("article");
      await h.clickLabel("Reload full article");
      await h.control(e => e.automationName === "Reload full article", "article reload recovers");
      await h.until(async () => !(await h.snapshot()).elements.some(e => /No readable article|too little article text/.test(e.text || "")), "successful reload clears its inline error");
    });

    await check("saved article survives reload and remains usable offline on mobile", async () => {
      await flush(h);
      await h.reload(); await h.ready();
      const restored = (await h.state()).articles.find(item => item.id === articleId);
      assert(restored.show_full_content && restored.full_content.html === cached.html);
      await h.clickNavigation("All items"); await h.page.press("1");
      await h.resize(390, 844);
      await h.until(async () => bodyText(await h.snapshot()).includes("Publisher body marker."), "mobile native full article");
      const table = await reveal(h, "ArticleTableScroll");
      assert(table.x >= 0 && table.x + table.width <= 390, "wide article content stays inside the mobile reading viewport");
      const code = await reveal(h, "ArticleCodeScroll");
      assert(code.x >= 0 && code.x + code.width <= 390, "code overflow uses its own horizontal viewport");
      await h.capture("14-article-mobile");
      // All three widths remain in the phone breakpoint. A row whose equality
      // tracks only narrow/touch would keep the first image's explicit width.
      for (const [width, expected] of [[390, 320], [320, 250], [390, 320]]) {
        await h.resize(width, 844);
        await top(h);
        await reveal(h, "ArticleImage");
        await h.until(async () => {
          const image = (await h.snapshot()).elements.find(e => e.name === "ArticleImage");
          return image && Math.abs(image.width - expected) <= 1
            && image.x >= -1 && image.x + image.width <= width + 1;
        }, `publisher image follows same-breakpoint resize to ${width}px`);
        if (width === 320) await h.capture("14-article-mobile-small-image");
      }
      await top(h);
      const count = network.count();
      await h.page.send("Network.emulateNetworkConditions", {offline: true, latency: 0, downloadThroughput: 0, uploadThroughput: 0});
      await h.until(() => h.page.evaluate("navigator.onLine === false"), "offline article fixture");
      await h.clickLabel("Show feed content");
      await h.clickLabel("Read full article");
      await h.until(async () => bodyText(await h.snapshot()).includes("Publisher body marker."), "saved full article opens while offline");
      assert.equal(network.count(), count);
      assert(network.requests.every(request => !request.cookie && !request.authorization));
      await h.record("14-article-verification", {articleId, sourceUrl: restored.full_content.source_url,
        fullContentRetained: true, offlineReopen: true, cancellationPreserved: true,
        extractionFailurePreserved: true, sameBreakpointImageWidths: [320, 250, 320],
        nativeInlineLinkOpened: true, selection, requests: network.requests});
    });
    passed = true;
  } finally {
    await h.page.send("Network.emulateNetworkConditions", {offline: false, latency: 0, downloadThroughput: -1, uploadThroughput: -1});
    if (passed) {
      await h.resize(1363, 936);
      await replace(h, baseline);
      await h.clickNavigation("All items");
    }
    await network.close();
  }
}
