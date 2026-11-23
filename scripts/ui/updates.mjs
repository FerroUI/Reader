import assert from "node:assert/strict";
import { readFile, writeFile, rm } from "node:fs/promises";
import path from "node:path";
import { prepareSite } from "../prepare-site.mjs";

// Exercise the real waiting-worker lifecycle against a temporary second build.
// Restore the exact deployable files before this gate returns, including on failure.
export async function updateChecks(h, check, site) {
  const paths = ["index.html", "app-sw.js", "build.json", ".nojekyll"];
  const originals = new Map(await Promise.all(paths.map(async name => [name, await readFile(path.join(site, name))])));
  const before = await h.state();
  const previousBuild = await h.page.evaluate("document.documentElement.dataset.readerBuild");
  let next, secondTarget;
  try {
    await check("a verified application update is offered without interrupting reading", async () => {
      await h.until(() => h.page.evaluate("navigator.onLine"), "network restored after offline check");
      await rm(path.join(site, "verification-feed.xml"), { force: true });
      const template = await readFile(new URL("../../crates/reader-browser/wwwroot/app-sw.js", import.meta.url), "utf8");
      next = await prepareSite(site, `${template}\n// Isolated application-update verification.\n`);
      assert.notEqual(next.version, previousBuild);
      await h.page.evaluate("navigator.serviceWorker.getRegistration().then(registration => registration.update())");
      await h.control(e => e.name === "ReaderUpdateNotice", "native update notice");
      assert.equal(await h.page.evaluate("document.documentElement.dataset.readerBuild"), previousBuild);
      assert.equal((await h.state()).articles.length, before.articles.length);
      await h.capture("21-update-available");
      await h.resize(320, 844);
      const notice = await h.control(e => e.name === "ReaderUpdateNotice", "phone update notice");
      const action = await h.control(e => /Button$/.test(e.type) && e.automationName === "Update Reader", "phone update action");
      assert(notice.x >= 0 && notice.x + notice.width <= 321);
      assert(action.height >= 44 && action.x >= 0 && action.x + action.width <= 321);
      await h.capture("21-update-phone");
      await h.clickLabel("Later");
      assert(!(await h.snapshot()).elements.some(e => e.name === "ReaderUpdateNotice"));
      assert.equal(await h.page.evaluate("document.documentElement.dataset.readerBuild"), previousBuild);
      await h.resize(1363, 936);
      await h.reload(); await h.ready();
      await h.control(e => e.name === "ReaderUpdateNotice", "waiting update survives a normal reload");
    });

    await check("updating protects other Reader tabs and restores the saved library", async () => {
      await writeFile(path.join(site, "update-client.html"), '<!doctype html><title>Reader update test window</title><link rel="icon" href="data:,"><p>Another Reader window.</p>');
      // A second document inside Reader's scope blocks activation without a
      // second application instance issuing unrelated storage writes.
      const target = await h.page.browserSend("Target.createTarget", { url: new URL("update-client.html?nooffline=1", h.page.url).href });
      secondTarget = target.targetId;
      assert(secondTarget);
      await h.until(async () => (await h.page.browserSend("Target.getTargets")).targetInfos
        .some(info => info.targetId === secondTarget && info.title === "Reader update test window"), "second Reader window");
      await h.page.send("Page.bringToFront");
      await h.clickLabel("Update Reader");
      await h.until(async () => (await h.snapshot()).elements.some(e => e.name === "ReaderUpdateMessage" && /Close your other Reader tabs/.test(e.text)), "other-tab update message");
      assert.equal(await h.page.evaluate("document.documentElement.dataset.readerBuild"), previousBuild);
      await h.capture("21-update-other-tab");
      await h.page.browserSend("Target.closeTarget", { targetId: secondTarget });
      secondTarget = null;
      await h.page.send("Page.bringToFront");
      const timeOrigin = await h.page.evaluate("performance.timeOrigin");
      await h.clickLabel("Update Reader");
      await h.until(() => h.page.evaluate(`performance.timeOrigin !== ${JSON.stringify(timeOrigin)} && document.documentElement.dataset.readerBuild === ${JSON.stringify(next.version)}`), "explicit update loads the waiting snapshot", 60000);
      await h.ready();
      const actual = await h.state();
      assert.deepEqual(actual.feeds, before.feeds);
      assert.deepEqual(actual.articles, before.articles);
      assert.deepEqual(actual.settings, before.settings);
      assert(!(await h.snapshot()).elements.some(e => e.name === "ReaderUpdateNotice"));
      await h.capture("21-update-complete");
      await h.record("21-update-verification", { previousBuild, stagedBuild: next.version,
        stagedResources: next.assets.length, savedArticles: actual.articles.length,
        subscriptions: actual.feeds.length, explicitActivation: true, otherTabsProtected: true,
        deployedFilesRestoredAfterFixture: true });
    });
  } finally {
    if (secondTarget) await h.page.browserSend("Target.closeTarget", { targetId: secondTarget }).catch(() => {});
    await rm(path.join(site, "update-client.html"), { force: true });
    for (const [name, contents] of originals) await writeFile(path.join(site, name), contents);
  }
}
