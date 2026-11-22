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
  const healthy = () => assert.equal(page.errors.length, 0, page.errors.join("\n"));
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
  async function resize(width, height) {
    await page.resize(width, height);
    await until(async () => {
      const view = await snapshot();
      return view.width === width && view.height === height;
    }, `viewport ${width} × ${height}`);
  }
  return { page, state, snapshot, healthy, until, ready, reload, navigate, control, click, clickName, clickLabel, clickNavigation, clickText, edit, inputHints, capture, resize };
}
