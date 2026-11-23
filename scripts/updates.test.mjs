import test from "node:test";
import assert from "node:assert/strict";
import { AppUpdates, flushBeforeUpdate } from "../crates/reader-browser/wwwroot/updates.js";

class Worker extends EventTarget {
  constructor(state = "installed") { super(); this.state = state; this.messages = []; }
  postMessage(message, ports) {
    this.messages.push(message);
    ports[0].postMessage({ status: this.reply || "accepted" });
  }
}

function fixture(overrides = {}) {
  const worker = new Worker(), old = new Worker("activated");
  const registration = new EventTarget();
  Object.assign(registration, { waiting: worker, installing: null, checks: 0,
    update: async () => { registration.checks++; },
  });
  const sw = new EventTarget();
  Object.assign(sw, { controller: old, register: async (script, options) => {
    sw.registrationOptions = options; return registration;
  } });
  let saves = 0, reloads = 0, now = 0;
  const notices = [];
  const updater = new AppUpdates({ serviceWorker: sw, script: "https://example.com/Reader/app-sw.js",
    scope: "https://example.com/Reader/", flush: async completed => { saves++; completed?.(); },
    notify: (message, applying) => notices.push({ message, applying }),
    reload: () => { reloads++; }, now: () => now, timeout: 50, log: () => {}, ...overrides,
  });
  return { updater, sw, registration, worker, notices, saves: () => saves, reloads: () => reloads,
    now: value => { now = value; }, activate: async () => {
      sw.controller = worker; sw.dispatchEvent(new Event("controllerchange"));
      await new Promise(resolve => setImmediate(resolve));
    },
  };
}

test("a waiting update is offered once without activating or reloading", async () => {
  const f = fixture(); await f.updater.start(); f.updater.offer();
  assert.deepEqual(f.sw.registrationOptions, { scope: "https://example.com/Reader/", updateViaCache: "none" });
  assert.equal(f.notices.length, 1); assert.match(f.notices[0].message, /new version/);
  assert.equal(f.saves(), 0); assert.equal(f.reloads(), 0); assert.equal(f.worker.messages.length, 0);
});

test("a first offline installation never asks to reload", async () => {
  const f = fixture(); f.sw.controller = null; await f.updater.start(); await f.activate();
  assert.equal(f.notices.length, 0); assert.equal(f.reloads(), 0);
});

test("the explicit update waits for storage, then reloads once after controller change", async () => {
  let finishSave, saves = 0;
  const f = fixture({ flush: async completed => { saves++; if (saves === 1) await new Promise(resolve => { finishSave = resolve; }); completed?.(); } });
  await f.updater.start(); const applying = f.updater.apply();
  assert.equal(f.worker.messages.length, 0); assert.equal(f.reloads(), 0);
  await f.updater.apply(); assert.equal(saves, 1, "rapid repeated activation is coalesced");
  finishSave(); await applying;
  assert.deepEqual(f.worker.messages, [{ type: "reader:activate" }]);
  assert.equal(f.reloads(), 0);
  await f.activate(); await f.activate();
  assert.equal(saves, 2, "changes during activation are also flushed");
  assert.equal(f.reloads(), 1);
});

test("storage failures keep the running page and never activate the worker", async () => {
  const f = fixture({ flush: async () => { throw new Error("Your library changed in another tab."); } });
  await f.updater.start(); await f.updater.apply();
  assert.equal(f.worker.messages.length, 0); assert.equal(f.reloads(), 0);
  assert.match(f.notices.at(-1).message, /not reloaded.*changed in another tab/);
  assert.equal(f.notices.at(-1).applying, false);
});

test("other Reader windows leave a clear retryable message", async () => {
  const f = fixture(); f.worker.reply = "blocked";
  await f.updater.start(); await f.updater.apply();
  assert.equal(f.reloads(), 0); assert.match(f.notices.at(-1).message, /Close your other Reader tabs/);
  f.worker.reply = "accepted"; await f.updater.apply(); await f.activate();
  assert.equal(f.reloads(), 1);
});

test("online/visibility checks are throttled and a newly installed worker is announced", async () => {
  const f = fixture(); f.registration.waiting = null;
  await f.updater.start(); await f.updater.check();
  assert.equal(f.registration.checks, 0);
  f.now(30 * 60 * 1000); await f.updater.check(); await f.updater.check();
  assert.equal(f.registration.checks, 1);
  f.registration.installing = f.worker; f.registration.dispatchEvent(new Event("updatefound"));
  f.registration.waiting = f.worker; f.worker.dispatchEvent(new Event("statechange"));
  assert.equal(f.notices.length, 1);
});

test("a nonresponsive waiting worker reports failure without reloading", async () => {
  const f = fixture({ timeout: 5 }); f.worker.postMessage = () => {};
  await f.updater.start(); await f.updater.apply();
  assert.equal(f.reloads(), 0); assert.match(f.notices.at(-1).message, /did not respond/);
});

test("an activation-time save error preserves the page and recovery message", async () => {
  let saves = 0;
  const f = fixture({ flush: async () => { if (++saves === 2) throw new Error("Storage full."); } });
  await f.updater.start(); await f.updater.apply(); await f.activate();
  assert.equal(f.reloads(), 0);
  assert.match(f.notices.at(-1).message, /Export a backup.*Storage full/);
});

test("an accepted slow activation stays in progress and reloads once when it completes", async () => {
  const f = fixture({ timeout: 5 });
  await f.updater.start(); await f.updater.apply();
  await new Promise(resolve => setTimeout(resolve, 10));
  assert.equal(f.notices.at(-1).applying, true, "Later must not be offered for an already accepted activation");
  assert.match(f.notices.at(-1).message, /still updating/);
  await f.updater.apply(); assert.equal(f.worker.messages.length, 1);
  await f.activate(); await f.activate(); assert.equal(f.reloads(), 1);
});

test("the reload barrier includes native changes made while a prior transaction is committing", async () => {
  let revision = 0, firstCommit;
  const saved = [], committed = [], order = [];
  const completion = flushBeforeUpdate(() => revision,
    () => { saved.push(revision); },
    async () => {
      const version = saved.at(-1);
      if (version === 0) await new Promise(resolve => { firstCommit = resolve; });
      committed.push(version);
    },
    () => { order.push("reload"); assert.equal(committed.at(-1), revision); });
  revision = 1; // A native command scheduled its coalesced save, not yet an IDB write.
  firstCommit();
  await completion;
  order.push("caller continuation");
  assert.deepEqual(saved, [0, 1]); assert.deepEqual(committed, [0, 1]);
  assert.deepEqual(order, ["reload", "caller continuation"], "reload belongs inside the final stable save turn");
});
