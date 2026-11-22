import assert from "node:assert/strict";

export async function articleSelection(h) {
  const paragraph = await h.control(element => element.name === "ArticleBodyText"
    && element.text?.startsWith("Publisher body marker."), "native selectable publisher paragraph");
  const y = paragraph.y + Math.min(12, paragraph.height / 2);
  await h.page.mouseDown(paragraph.x + 3, y);
  for (const distance of [35, 75, 135]) {
    await h.page.send("Input.dispatchMouseEvent", {type: "mouseMoved", pointerType: "mouse",
      x: paragraph.x + distance, y, buttons: 1});
  }
  await h.page.mouseUp(paragraph.x + 135, y);
  const selection = await h.until(async () => {
    const selected = (await h.snapshot()).elements.find(element => element.name === "ArticleBodyText"
      && element.text === paragraph.text)?.textSelection;
    return selected?.start !== selected?.end && selected?.text?.length > 8 && selected;
  }, "mouse drag selects native rich article text");
  assert(paragraph.text.includes(selection.text), "selection exports the displayed publisher text");

  // Selection remains the assertion on platforms without clipboard read access.
  // When Chromium grants it, verify that Ctrl+C reaches the native view's copy.
  const permission = await h.page.browserSend("Browser.setPermission", {
    permission: {name: "clipboard-read"}, setting: "granted", origin: new URL(h.page.url).origin,
  });
  await h.page.press("c", ["ctrl"]);
  let copyVerified = false;
  if (!permission.error) {
    const copied = await h.until(async () => {
      const result = await h.page.evaluate(`navigator.clipboard?.readText
        ? navigator.clipboard.readText().then(text => ({text}), () => ({unavailable:true}))
        : ({unavailable:true})`);
      return result.unavailable || result.text === selection.text ? result : false;
    }, "native article copy reaches the browser clipboard");
    copyVerified = !copied.unavailable;
  }
  await h.page.click(paragraph.x + 3, y);
  return {selectedText: selection.text, copyVerified};
}

export async function articleLink(h, url) {
  let target;
  for (let attempt = 0; attempt < 10; attempt++) {
    const view = await h.snapshot();
    for (const paragraph of view.elements.filter(element => element.name === "ArticleBodyText")) {
      const link = paragraph.links?.find(link => link.url === url);
      const bounds = link?.bounds.find(bounds => bounds.width > 0 && bounds.height > 0
        && bounds.y >= 175 && bounds.y + bounds.height < view.height - 40);
      if (bounds) { target = {paragraph, bounds}; break; }
    }
    if (target) break;
    await h.page.wheel(view.width - 45, Math.min(460, view.height - 80), 0, 160);
  }
  assert(target, "a shaped inline publisher link is visible");
  assert(target.paragraph.text.startsWith("Before the ") && target.paragraph.text.endsWith(" after the link."),
    "the clickable link is surrounded by native unlinked text");
  const before = await h.page.browserSend("Target.getTargets");
  const existing = new Set(before.targetInfos.map(target => target.targetId));
  const readerUrl = await h.page.evaluate("location.href");
  const opened = label => h.until(async () => {
    const targets = await h.page.browserSend("Target.getTargets");
    return targets.targetInfos.find(target => !existing.has(target.targetId)
      && target.type === "page" && target.url === url);
  }, label);
  try {
    const {bounds} = target;
    await h.page.click(bounds.x + bounds.width / 2, bounds.y + bounds.height / 2);
    const pointer = await opened("native inline hyperlink opens its resolved publisher address");
    await h.page.browserSend("Target.closeTarget", {targetId: pointer.targetId});
    await h.page.send("Page.bringToFront");
    // Plain text focuses the native paragraph; Tab focuses its embedded link,
    // and Enter must navigate before Reader's global item-opening shortcut.
    await h.page.click(target.paragraph.x + 3, bounds.y + bounds.height / 2);
    await h.page.press("Tab");
    await h.page.press("Enter");
    await opened("Tab and Enter activate the native inline hyperlink");
    assert.equal(await h.page.evaluate("location.href"), readerUrl, "Reader stays open while the original link opens separately");
  } finally {
    const targets = await h.page.browserSend("Target.getTargets");
    for (const target of targets.targetInfos) {
      if (target.type === "page" && !existing.has(target.targetId))
        await h.page.browserSend("Target.closeTarget", {targetId: target.targetId});
    }
  }
}
