import assert from "node:assert/strict";
import { mkdir, writeFile } from "node:fs/promises";
import path from "node:path";
import { open, sleep, colours } from "../../FerroUI/scripts/browser/harness.mjs";

// Real browser input is delivered to the compiled FerroUI control tree.
export async function createHarness(site, output) {
  await mkdir(output, { recursive: true });
  const page = await open(site, { query: "?nooffline=1", width: 1363, height: 936, network: true });
  const snapshot = async () => JSON.parse(await page.evaluate("readerApp.inspectView()"));
  const state = async () => JSON.parse(await page.evaluate("readerApp.exportData()"));
  const expectedErrors = new Map(), networkScopes = new Set();
  page.on("Network.requestWillBeSent", (event, session) => {
    if (!session) for (const scope of networkScopes) scope.requests.set(event.requestId, event.request.url);
  });
  page.on("Log.entryAdded", ({entry}, session) => {
    if (session || entry.level !== "error" || entry.source !== "network") return;
    for (const scope of networkScopes) {
      const urls = [entry.url, scope.requests.get(entry.networkRequestId)].filter(Boolean);
      const cors = [...scope.cors].find(url => entry.text.includes(`at '${url}'`) && entry.text.includes("blocked by CORS policy"));
      const failed = urls.find(url => scope.cors.has(url) && /^Failed to load resource: net::ERR_FAILED\b/.test(entry.text));
      const status = /^Failed to load resource: the server responded with a status of (\d+)\b/.exec(entry.text)?.[1];
      const http = status && urls.find(url => scope.http.get(url)?.has(Number(status)));
      if (!cors && !failed && !http) continue;
      const line = `[log.${entry.level}] ${entry.text}${entry.url ? ` (${entry.url})` : ""}`;
      expectedErrors.set(line, (expectedErrors.get(line) || 0) + 1);
      scope.logs.push({kind: cors ? "cors-policy" : failed ? "cors-load" : "http-status", url: cors || failed || http, text: entry.text});
      break;
    }
  });
  const healthy = () => {
    // Consume only recorded occurrences. A later identical error outside its
    // explicitly scoped fixture is still unexpected; console/JS errors never qualify.
    const remaining = new Map(expectedErrors);
    const unexpected = page.errors.filter(line => {
      const count = remaining.get(line) || 0;
      if (count) { remaining.set(line, count - 1); return false; }
      return true;
    });
    assert.equal(unexpected.length, 0, unexpected.join("\n"));
  };
  async function expectNetworkErrors({cors = [], http = []}, run) {
    healthy();
    const scope = {cors: new Set(cors), http: new Map(), requests: new Map(), logs: []};
    for (const {url, status} of http) {
      if (!scope.http.has(url)) scope.http.set(url, new Set());
      scope.http.get(url).add(status);
    }
    networkScopes.add(scope);
    try { return await run(scope.logs); }
    finally { networkScopes.delete(scope); healthy(); }
  }
  async function until(check, label, timeout = 15000) {
    const end = Date.now() + timeout;
    let lastError;
    while (Date.now() < end) {
      healthy();
      try { const value = await check(); if (value) return value; }
      catch (error) { lastError = error; }
      await sleep(75);
    }
    throw new Error(`Timed out: ${label}${lastError ? ` (${lastError.message})` : ""}`);
  }
  async function ready() {
    await until(() => page.evaluate(`!!document.querySelector('canvas') &&
      (!document.querySelector('.ferroui-splash') || document.querySelector('.ferroui-splash').classList.contains('splash-close'))`), "native first frame", 60000);
    await until(() => page.evaluate("!!globalThis.readerApp?.inspectView"), "Reader startup", 60000);
    await control(e => e.name === "SearchReader", "the search field");
  }
  async function newDocument(method, params) {
    const previous = await page.evaluate("performance.timeOrigin");
    const result = await page.send(method, params);
    assert(!result?.error && !result?.errorText, JSON.stringify(result));
    await until(() => page.evaluate(`performance.timeOrigin !== ${JSON.stringify(previous)}`),
      "a new document after navigation", 60000);
  }
  const reload = () => newDocument("Page.reload", {});
  const navigate = url => newDocument("Page.navigate", { url });
  async function control(match, label) {
    return until(async () => {
      const matches = (await snapshot())?.elements.filter(e => e.hit && match(e)) || [];
      assert(matches.length <= 1, `Ambiguous control: ${label}`);
      return matches[0];
    }, label);
  }
  async function click(match, label) {
    const e = await control(match, label);
    await page.click(e.x + e.width / 2, e.y + e.height / 2);
  }
  const clickName = name => click(e => e.name === name, name);
  const clickLabel = label => click(e => /Button$/.test(e.type) && e.automationName === label, label);
  const clickNavigation = label => click(e => /Button$/.test(e.type) && e.automationName === label && e.x < 100 && e.y > 60, `navigation: ${label}`);
  const clickText = text => click(e => /(?:TextBlock|AccessText)$/.test(e.type) && e.text === text, text);
  async function edit(name, value, submit = false) {
    await clickName(name);
    await page.press("a", ["ctrl"]);
    await page.send("Input.insertText", { text: value });
    if (!value) await page.press("Backspace");
    if (submit) await page.press("Enter");
  }
  async function inputHints(mode, enter) {
    await until(async () => {
      const actual = await page.evaluate(`(() => {
        const input = document.querySelector('.ferroui-input-element');
        return input && ['inputmode', 'enterkeyhint', 'autocapitalize', 'autocorrect', 'spellcheck']
          .map(name => input.getAttribute(name));
      })()`);
      assert.deepEqual(actual, [mode, enter, "none", "off", "false"]);
      return true;
    }, `browser ${mode}/${enter} keyboard hints`);
  }
  async function capture(name) {
    await sleep(150);
    healthy();
    const image = await page.screenshot(path.join(output, `${name}.png`));
    assert(colours(image) > 40, `${name}: empty rendered surface`);
    await writeFile(path.join(output, `${name}.json`), JSON.stringify(await snapshot(), null, 2));
  }
  const record = (name, value) => writeFile(path.join(output, `${name}.json`), JSON.stringify(value, null, 2));
  async function resize(width, height) {
    await page.resize(width, height);
    await until(async () => {
      const view = await snapshot();
      return view.width === width && view.height === height;
    }, `viewport ${width} × ${height}`);
  }
  return { page, state, snapshot, healthy, until, ready, reload, navigate, control, click, clickName, clickLabel, clickNavigation, clickText, edit, inputHints, capture, record, resize, expectNetworkErrors };
}
