import assert from "node:assert/strict";
import { sleep } from "../../FerroUI/scripts/browser/harness.mjs";

// Chromium delivers these as trusted touch input through the browser platform,
// the FerroUI pointer pipeline and native gesture recognizers.
export async function createTouch(page) {
  async function send(method, params) {
    const result = await page.send(method, params);
    assert(!result?.error, JSON.stringify(result));
  }
  await send("Emulation.setTouchEmulationEnabled", { enabled: true, maxTouchPoints: 5 });
  const points = contacts => contacts.map(([x, y, id = 0]) => ({
    x, y, id, radiusX: 2, radiusY: 2, force: 1,
  }));
  const start = (...contacts) => send("Input.dispatchTouchEvent", { type: "touchStart", touchPoints: points(contacts) });
  const move = (...contacts) => send("Input.dispatchTouchEvent", { type: "touchMove", touchPoints: points(contacts) });
  const end = () => send("Input.dispatchTouchEvent", { type: "touchEnd", touchPoints: [] });
  const cancel = () => send("Input.dispatchTouchEvent", { type: "touchCancel", touchPoints: [] });
  async function tap(x, y) {
    await start([x, y]);
    await sleep(45);
    await end();
    await sleep(120);
  }
  async function travel(from, to, steps = 8) {
    for (let step = 1; step <= steps; step++) {
      const t = step / steps;
      await move([from[0] + (to[0] - from[0]) * t, from[1] + (to[1] - from[1]) * t]);
      await sleep(25);
    }
  }
  async function drag(from, to, hold = 45) {
    await start(from);
    await sleep(hold);
    await travel(from, to);
    await end();
    await sleep(120);
  }
  const tapControl = e => tap(e.x + e.width / 2, e.y + e.height / 2);
  const disable = () => send("Emulation.setTouchEmulationEnabled", { enabled: false });
  return { start, move, end, cancel, tap, tapControl, travel, drag, disable };
}
