import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { join } from "node:path";

// Pause newly created page targets before their first navigation, then fulfill
// every request locally. This also covers noopener windows that a page-scoped
// interceptor cannot reliably observe. The app's native OpenUrl path is intact.
export async function interceptSendToPopup(h, expected) {
  const {arguments: args} = await h.page.browser();
  const profile = args.find(arg => arg.startsWith("--user-data-dir="))?.slice("--user-data-dir=".length);
  assert(profile, "Chrome must expose its isolated test profile");
  const [port, endpoint] = (await readFile(join(profile, "DevToolsActivePort"), "utf8")).trim().split("\n");
  const socket = new WebSocket(`ws://127.0.0.1:${port}${endpoint}`);
  await new Promise((resolve, reject) => {
    const timer = setTimeout(() => reject(new Error("Send to browser connection timed out")), 5000);
    socket.addEventListener("open", () => { clearTimeout(timer); resolve(); }, {once: true});
    socket.addEventListener("error", () => { clearTimeout(timer); reject(new Error("Send to browser connection failed")); }, {once: true});
  });
  const existing = new Set((await h.page.browserSend("Target.getTargets")).targetInfos.map(target => target.targetId));
  const pending = new Map(), targets = new Set(), sessions = new Set(), errors = [], requests = [];
  let id = 0, fulfilled = false;
  function send(method, params = {}, sessionId) {
    return new Promise((resolve, reject) => {
      const key = ++id;
      const timer = setTimeout(() => { pending.delete(key); reject(new Error(`${method} timed out`)); }, 5000);
      pending.set(key, message => {
        clearTimeout(timer);
        if (message.error) reject(new Error(`${method}: ${message.error.message}`));
        else resolve(message.result);
      });
      socket.send(JSON.stringify({id: key, method, params, ...(sessionId ? {sessionId} : {})}));
    });
  }
  async function event(message) {
    if (message.method === "Target.attachedToTarget") {
      const {sessionId, targetInfo} = message.params;
      if (existing.has(targetInfo.targetId)) return;
      targets.add(targetInfo.targetId); sessions.add(sessionId);
      await send("Fetch.enable", {patterns: [{urlPattern: "*", requestStage: "Request"}]}, sessionId);
      await send("Runtime.runIfWaitingForDebugger", {}, sessionId);
    } else if (message.method === "Fetch.requestPaused" && sessions.has(message.sessionId)) {
      const {requestId, request} = message.params;
      requests.push(request.url);
      if (request.url === expected) {
        await send("Fetch.fulfillRequest", {requestId, responseCode: 200,
          responseHeaders: [{name: "Content-Type", value: "text/html; charset=utf-8"},
            {name: "Cache-Control", value: "no-store"}],
          body: Buffer.from('<!doctype html><title>Local Send to fixture</title><link rel="icon" href="data:,"><p>Article received by the isolated test fixture.</p>').toString("base64")}, message.sessionId);
        fulfilled = true;
      } else {
        // An unexpected address is stopped before DNS or an external request.
        await send("Fetch.failRequest", {requestId, errorReason: "BlockedByClient"}, message.sessionId);
        throw new Error(`Unexpected Send to popup request: ${request.url}`);
      }
    }
  }
  socket.addEventListener("message", ({data}) => {
    const message = JSON.parse(data);
    if (message.id) { pending.get(message.id)?.(message); pending.delete(message.id); }
    else void event(message).catch(error => errors.push(error));
  });
  try {
    await send("Target.setAutoAttach", {autoAttach: true, waitForDebuggerOnStart: true, flatten: true,
      filter: [{type: "page", exclude: false}, {exclude: true}]});
  } catch (error) { socket.close(); throw error; }
  return {
    async opened() {
      await h.until(async () => {
        if (errors.length) throw errors[0];
        if (!fulfilled) return false;
        return (await h.page.browserSend("Target.getTargets")).targetInfos
          .find(target => targets.has(target.targetId) && target.url === expected);
      }, "native Send to opens the exact encoded URL through the intercepted popup", 6000);
      if (errors.length) throw errors[0];
      assert.deepEqual(requests, [expected], "one native activation must make one locally fulfilled request");
    },
    async close() {
      for (const targetId of targets) await send("Target.closeTarget", {targetId});
      await send("Target.setAutoAttach", {autoAttach: false, waitForDebuggerOnStart: false, flatten: true});
      socket.close();
      await h.page.send("Page.bringToFront");
    },
  };
}
