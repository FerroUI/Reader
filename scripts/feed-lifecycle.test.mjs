import test from "node:test";
import assert from "node:assert/strict";
import { attachRuntime, request } from "../crates/reader-browser/wwwroot/host.js";

test("refresh failures carry the captured subscription identity while new-subscription errors remain visible", async () => {
  const previous = globalThis.fetch;
  const errors = [];
  let complete;
  attachRuntime({
    libraryGeneration: () => 102,
    currentView: () => '"Starred"',
    receiveFeed: assert.fail,
    feedError: (...args) => errors.push(args),
    hostError: assert.fail,
    storageError: assert.fail,
    finishRefresh: () => complete(),
  });
  globalThis.fetch = async () => new Response("Unavailable", {status: 503});
  async function perform(message) {
    const finished = new Promise(resolve => { complete = resolve; });
    request(JSON.stringify({...message, connection: "Direct", proxy: ""}));
    await finished;
  }
  try {
    await perform({kind: "refresh", feeds: [{id: 81, url: "https://lifecycle.example/feed"}], epochs: {81: 19}});
    await perform({kind: "subscribe", url: "https://lifecycle.example/new"});
    assert.equal(errors.length, 2);
    assert.deepEqual(errors.map(([url, , id, epoch]) => [url, id, epoch]), [
      ["https://lifecycle.example/feed", 81, 19],
      ["https://lifecycle.example/new", 0, 0],
    ]);
    assert(errors.every(([, message]) => message.includes("503")), "the original HTTP cause survives both paths");
  } finally {
    globalThis.fetch = previous;
  }
});
