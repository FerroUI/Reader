import assert from "node:assert/strict";
import { sleep } from "../../FerroUI/scripts/browser/harness.mjs";
import { readingFixture } from "./fixtures.mjs";

const replace = (h, data) => h.page.evaluate(`readerApp.importData(${JSON.stringify(JSON.stringify(data))})`);
const radio = name => e => /RadioButton$/.test(e.type) && e.automationName === name;
const restoreViewport = h => h.page.evaluate(`(() => {
  for (const name of ['width','height','offsetTop','offsetLeft','scale']) delete visualViewport[name];
  visualViewport.dispatchEvent(new Event('resize'));
})()`);
function inside(e, box, label = e.name || e.automationName) {
  assert(e.x >= box.x - 1 && e.y >= box.y - 1 && e.x + e.width <= box.x + box.width + 1 && e.y + e.height <= box.y + box.height + 1,
    `${label} must fit its visible container: ${JSON.stringify({element:e, box})}`);
}
async function close(h) {
  if (await h.page.evaluate("readerApp.hasOverlay()")) await h.clickLabel("Close dialog");
  await h.until(() => h.page.evaluate("!readerApp.hasOverlay()"), "settings dialog closes");
}
async function open(h, title) {
  await close(h);
  await h.clickLabel("Settings");
  await h.click(e => /MenuItem$/.test(e.type) && e.text === title, `${title} menu item`);
  await h.control(e => e.name === "ReaderDialog", `${title} dialog`);
}
async function reveal(h, match, label) {
  let direction = 1, previous;
  for (let attempt = 0; attempt < 28; attempt++) {
    const view = await h.snapshot(), scroll = view.elements.find(e => e.name === "DialogScroll")
      || view.elements.find(e => e.name === "ReaderPageScroll");
    assert(scroll, "settings use a native ScrollViewer");
    const item = view.elements.find(e => e.hit && match(e));
    if (item && item.y >= scroll.y - 1 && item.y + item.height <= scroll.y + scroll.height + 1) return item;
    const offset = scroll.scroll.offsetY;
    if (previous !== undefined && Math.abs(offset - previous) < 0.2) direction *= -1;
    previous = offset;
    await h.page.wheel(scroll.x + scroll.width / 2, scroll.y + scroll.height / 2, 0, direction * scroll.height * 0.65);
    await sleep(100);
  }
  throw new Error(`Could not reveal ${label}`);
}
async function clickVisible(h, match, label) {
  const item = await reveal(h, match, label);
  await h.page.click(item.x + item.width / 2, item.y + item.height / 2);
}
async function selected(h, names, expected) {
  await h.until(async () => {
    const options = (await h.snapshot()).elements.filter(e => /RadioButton$/.test(e.type) && names.includes(e.automationName));
    assert.equal(options.length, names.length);
    assert.deepEqual(options.filter(e => e.checked).map(e => e.automationName), [expected]);
    return true;
  }, `${expected} is the only checked native radio option`);
}

export async function settingsPolishChecks(h, check) {
  const baseline = await h.state(), viewport = await h.snapshot();
  const route = await h.page.evaluate("readerApp.currentView()");
  const data = readingFixture(baseline);
  Object.assign(data.settings, {density:"Comfortable", font_size:14, feed_connection:"Automatic", feed_proxy:"", send_to:[]});
  data.feeds[0].title = "The Rust Blog — language, libraries and release announcements";
  data.feeds[0].url = "https://ui-fixture.example/feeds/rust-language-and-library-release-announcements.xml";
  data.feeds[0].error = "The publisher did not allow browser access. Check the feed connection setting.";
  try {
    await h.resize(1363, 936);
    await replace(h, data);
    await h.page.evaluate(`readerApp.navigate(${JSON.stringify(JSON.stringify("Home"))})`);
    await check("settings expose native selection and compact wrapping preference controls", async () => {
      await open(h, "Reader settings");
      const checkbox = await h.control(e => e.name === "ConfirmMarkAll", "confirmation preference");
      assert.equal(checkbox.height, 28, "desktop settings must not inherit Material's fixed 48px checkbox row");
      await h.clickName("ConfirmMarkAll");
      await h.until(async () => (await h.state()).settings.confirm_mark_all === !checkbox.checked, "native checkbox click persists");
      await h.page.press("Space");
      await h.until(async () => (await h.state()).settings.confirm_mark_all === checkbox.checked, "Space toggles the focused checkbox");
      await selected(h, ["Comfortable","Cozy","Compact"], "Comfortable");
      await h.click(radio("Compact"), "Compact native option");
      await h.until(async () => (await h.state()).settings.density === "Compact", "density persists from native radio selection");
      await h.page.press("ArrowLeft");
      await h.until(async () => (await h.state()).settings.density === "Cozy", "Left selects previous density");
      await selected(h, ["Comfortable","Cozy","Compact"], "Cozy");
      assert((await h.control(radio("Cozy"), "focused Cozy option")).focused);
      await h.page.press("Tab");
      const medium = await h.control(radio("Medium"), "selected font option");
      assert(medium.focused, "Tab leaves the radio group at its selected option");
      await h.page.press("ArrowRight");
      await h.until(async () => (await h.state()).settings.font_size === 16, "font choice persists with arrow navigation");
      await selected(h, ["Small","Medium","Large","Extra large"], "Large");
      await h.capture("19-settings-desktop");
    });

    await check("320px settings keep connection drafts and errors through keyboard resizing", async () => {
      await h.resize(320, 740);
      await reveal(h, e => e.name === "ConfirmMarkAll", "wrapping confirmation preference");
      let view = await h.snapshot();
      const dialog = view.elements.find(e => e.name === "ReaderDialog");
      inside(dialog, {x:0,y:0,width:view.width,height:view.height});
      const checkbox = await h.control(e => e.name === "ConfirmMarkAll", "phone confirmation preference");
      assert(checkbox.height >= 44);
      inside(checkbox, dialog);
      const caption = view.elements.find(e => e.text === "Confirm before marking a stream with more than 50 unread items as read");
      assert(caption && caption.height > 20, "the long checkbox caption wraps on a phone");
      inside(caption, dialog);
      await h.capture("19-settings-phone-320");
      await clickVisible(h, radio("Custom proxy"), "custom connection option");
      await reveal(h, e => e.name === "FeedProxy", "proxy URL input");
      await h.edit("FeedProxy", "https://proxy.example/feed", true);
      await h.inputHints("url", "done");
      const error = await reveal(h, e => e.name === "FeedConnectionError", "proxy validation error");
      assert(error.text.includes("{url}"));
      assert.equal((await h.state()).settings.feed_connection, "Automatic", "invalid connection draft is not saved");
      await reveal(h, e => e.name === "FeedProxy", "invalid proxy draft");
      await h.clickName("FeedProxy");
      await h.page.evaluate(`(() => {
        for (const [name,value] of Object.entries({width:320,height:360,offsetTop:20,offsetLeft:0,scale:1}))
          Object.defineProperty(visualViewport,name,{configurable:true,value});
        visualViewport.dispatchEvent(new Event('resize'));
      })()`);
      await h.until(async () => (await h.snapshot()).height === 360, "settings follow visible keyboard viewport");
      const input = await reveal(h, e => e.name === "FeedProxy", "keyboard-sized proxy input");
      assert.equal(input.text, "https://proxy.example/feed");
      assert(input.focused, "keyboard resize keeps the native editor focused");
      const retained = await reveal(h, e => e.name === "FeedConnectionError", "retained proxy validation error");
      assert.equal(retained.text, error.text, "responsive layout must not clear validation");
      await h.capture("19-settings-keyboard-error");
      await restoreViewport(h);
      await h.until(async () => (await h.snapshot()).height === 740, "settings keyboard closes");
      await reveal(h, e => e.name === "FeedProxy", "proxy correction field");
      await h.edit("FeedProxy", "https://proxy.example/feed?url={url}", true);
      await h.until(async () => (await h.state()).settings.feed_connection === "Custom", "valid connection saved by Enter");
      const saved = await reveal(h, e => e.name === "FeedConnectionDraftStatus", "in-dialog saved status");
      assert.equal(saved.text, "Connection settings are saved.");
      await h.capture("19-settings-connection-saved");
    });

    await check("Send to editor advances fields and retains an invalid draft on narrow screens", async () => {
      await clickVisible(h, e => e.automationName === "Customize Send to", "Send to configuration");
      await h.clickLabel("Create a custom link");
      await h.edit("SendToName", "Personal reading list");
      await h.page.press("Enter");
      await h.until(async () => (await h.control(e => e.name === "SendToTemplate", "URL template after Next")).focused,
        "Enter in the name advances to URL without saving");
      assert.deepEqual((await h.state()).settings.send_to, []);
      await h.edit("SendToTemplate", "https://share.example/?value=${unsupported}", true);
      const error = await reveal(h, e => e.name === "SendToError", "template validation error");
      assert(error.text.includes("Unsupported placeholder"));
      await h.resize(390, 844);
      const retained = await reveal(h, e => e.name === "SendToError", "retained template error");
      assert.equal(retained.text, error.text);
      await reveal(h, e => e.name === "SendToTemplate", "retained template draft");
      assert.equal((await h.control(e => e.name === "SendToTemplate", "template draft")).text,
        "https://share.example/?value=${unsupported}");
      await h.capture("19-settings-send-to-draft");
      await clickVisible(h, e => e.automationName === "Cancel", "cancel Send to draft");
      assert.deepEqual((await h.state()).settings.send_to, []);
      await close(h);
    });

    await check("subscription filtering and contextual actions remain usable at 320px", async () => {
      await h.resize(320, 740);
      await open(h, "Subscriptions");
      const input = await reveal(h, e => e.name === "SubscriptionFilter", "subscription filter");
      assert(input.height >= 44);
      await h.edit("SubscriptionFilter", "  RUST  ");
      await h.until(async () => (await h.snapshot()).elements.some(e => e.name === "SubscriptionResultCount" && e.text === "1 of 6 subscriptions"),
        "filter trims whitespace and searches case-insensitively");
      const title = data.feeds[0].title;
      const rename = await reveal(h, e => e.automationName === `Rename subscription ${title}`, "specific subscription Rename action");
      assert(rename.height >= 44);
      const view = await h.snapshot(), dialog = view.elements.find(e => e.name === "ReaderDialog");
      inside(rename, dialog);
      assert.equal(view.elements.filter(e => e.name?.startsWith("Subscription-")).length, 1);
      await h.capture("19-settings-subscriptions-320");
      await h.page.click(rename.x + rename.width/2, rename.y + rename.height/2);
      assert.equal((await h.control(e => e.name === "DialogInput", "rename selected subscription")).text, title);
      await h.clickLabel("Cancel");
      await open(h, "Subscriptions");
      await reveal(h, e => e.name === "ClearSubscriptionFilter", "clear persistent filter");
      await h.clickName("ClearSubscriptionFilter");
      await h.until(async () => (await h.snapshot()).elements.some(e => e.name === "SubscriptionResultCount" && e.text === "6 subscriptions"), "clear restores all subscriptions");
      await h.edit("SubscriptionFilter", "no-matching-subscription");
      await h.control(e => e.text?.startsWith("No subscriptions match your filter."), "actionable empty filter result");
      await h.capture("19-settings-subscriptions-empty");
      await h.clickName("ClearSubscriptionFilter");
    });

    await check("the direct settings route saves a connection without a modal", async () => {
      await close(h);
      await h.resize(1363, 936);
      await h.page.evaluate(`readerApp.navigate(${JSON.stringify(JSON.stringify("Settings"))})`);
      await h.control(e => e.name === "ReaderPageScroll", "full-page settings scroll viewer");
      assert(!(await h.snapshot()).elements.some(e => e.name === "ReaderDialog"));
      await clickVisible(h, radio("Direct only"), "direct connection choice in full-page settings");
      await clickVisible(h, e => e.automationName === "Save feed connection", "full-page connection save");
      await h.until(async () => (await h.state()).settings.feed_connection === "Direct", "direct settings route commits its connection draft");
      assert.equal((await h.state()).settings.feed_proxy, "https://proxy.example/feed?url={url}", "Direct mode keeps the saved custom proxy");
      await h.capture("19-settings-direct-route");
    });
  } catch (error) {
    await h.capture("19-settings-failure").catch(() => {});
    throw error;
  } finally {
    await restoreViewport(h);
    await close(h);
    await replace(h, baseline);
    await h.page.evaluate(`readerApp.navigate(${JSON.stringify(route)})`);
    await h.resize(viewport.width, viewport.height);
    await h.page.evaluate("readerApp.flushSave(); import('./host.js').then(m=>m.flush())");
  }
}
