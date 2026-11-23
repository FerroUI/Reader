// A waiting, fully verified snapshot is offered through native Reader controls.
// Installation never interrupts reading; only an explicit update requests activation.
const READY = "A new version of Reader is ready.";
const OTHER_TABS = "Close your other Reader tabs, then choose Update Reader.";
const INTERVAL = 30 * 60 * 1000;
let current;

// Changes can remain dirty in the native VM while an earlier IDB transaction
// is awaited. Drain both layers, then run reload in the same stable turn.
export async function flushBeforeUpdate(revision, save, commit, completed) {
  let saved;
  do { saved = revision(); save(); await commit(); } while (saved !== revision());
  completed?.();
}

export class AppUpdates {
  constructor({ serviceWorker, script, scope, flush, notify, reload, now = Date.now,
    channel = () => new MessageChannel(), timeout = 30000, log = console.info }) {
    Object.assign(this, { serviceWorker, script, scope, flush, notify, reload, now, channel, timeout, log });
    this.registration = null;
    this.announced = null;
    this.busy = false;
    this.reloadArmed = false;
    this.reloading = false;
    this.lastCheck = -Infinity;
    this.watched = new WeakSet();
    this.onController = () => { void this.finishActivation(); };
    serviceWorker.addEventListener("controllerchange", this.onController);
  }

  async start() {
    try {
      this.lastCheck = this.now();
      this.registration = await this.serviceWorker.register(this.script, {
        scope: this.scope, updateViaCache: "none",
      });
      this.registration.addEventListener("updatefound", () => this.observeInstalling());
      this.observeInstalling();
      this.offer();
    } catch (error) {
      // Offline first visits must remain usable, without replacing a feed/storage message.
      this.log("Offline installation is unavailable:", error?.message || error);
    }
  }

  observeInstalling() {
    const worker = this.registration?.installing;
    if (!worker || this.watched.has(worker)) return;
    this.watched.add(worker);
    worker.addEventListener("statechange", () => this.offer());
  }

  offer() {
    const waiting = this.registration?.waiting;
    if (!this.serviceWorker.controller || !waiting || waiting === this.announced || this.busy) return;
    this.announced = waiting;
    this.notify(READY, false);
  }

  async check() {
    if (!this.registration || this.busy || this.now() - this.lastCheck < INTERVAL) return;
    this.lastCheck = this.now();
    try { await this.registration.update(); this.observeInstalling(); this.offer(); }
    catch (error) { this.log("Reader will check for an update when connected:", error?.message || error); }
  }

  async apply() {
    if (this.busy) return;
    const waiting = this.registration?.waiting;
    if (!waiting) {
      this.notify("The update is no longer waiting. Reopen Reader to use the latest installed version.", false);
      return;
    }
    this.busy = true;
    clearTimeout(this.activationTimer);
    this.notify("Saving your library and updating Reader…", true);
    try {
      await this.flush();
      // Recheck after saving: another tab may have installed/activated a different worker.
      if (waiting !== this.registration.waiting) {
        this.busy = false;
        this.notify("The available update changed. Choose Update Reader again.", false);
        return;
      }
      this.previousController = this.serviceWorker.controller;
      this.reloadArmed = true;
      const reply = await this.requestActivation(waiting);
      if (reply.status === "blocked") {
        this.reloadArmed = false;
        this.busy = false;
        this.notify(OTHER_TABS, false);
        return;
      }
      if (reply.status !== "accepted") throw new Error("Reader could not activate the update. Please try again.");
      await this.finishActivation();
      if (this.reloadArmed && !this.reloading) {
        this.activationTimer = setTimeout(() => {
          // Activation was accepted and can still complete. Do not offer Later
          // or Retry while the user's original update request remains in flight.
          this.notify("Reader is still updating. You can close and reopen this tab to finish.", true);
        }, this.timeout);
      }
    } catch (error) {
      this.busy = false;
      this.reloadArmed = false;
      this.notify(`Reader was not reloaded. ${error?.message || error}`, false);
    }
  }

  requestActivation(worker) {
    return new Promise((resolve, reject) => {
      const { port1, port2 } = this.channel();
      const close = () => { clearTimeout(timer); port1.close(); port2.close(); };
      const timer = setTimeout(() => {
        close(); reject(new Error("The update did not respond. Close and reopen Reader, or try again."));
      }, this.timeout);
      port1.onmessage = event => { close(); resolve(event.data || {}); };
      port1.onmessageerror = () => { close(); reject(new Error("The update response could not be read.")); };
      try { worker.postMessage({ type: "reader:activate" }, [port2]); }
      catch (error) { close(); reject(error); }
    });
  }

  async finishActivation() {
    if (!this.reloadArmed || this.reloading ||
      !this.serviceWorker.controller || this.serviceWorker.controller === this.previousController) return;
    this.reloading = true;
    clearTimeout(this.activationTimer);
    try {
      // Also include changes made while the worker was finishing activation.
      await this.flush(() => {
        this.reloadArmed = false;
        this.reload();
      });
    } catch (error) {
      this.reloading = false;
      this.reloadArmed = false;
      this.busy = false;
      this.notify(`Your library could not be saved. Export a backup before reloading. ${error?.message || error}`, false);
    }
  }
}

export function attachUpdates(runtime, flush) {
  if (!("serviceWorker" in navigator) || new URLSearchParams(location.search).has("nooffline")) return;
  current = new AppUpdates({ serviceWorker: navigator.serviceWorker,
    script: new URL("./app-sw.js", import.meta.url), scope: new URL("./", import.meta.url).href,
    flush, notify: (message, applying) => runtime.appUpdateStatus(message, applying),
    reload: () => location.reload(),
  });
  void current.start();
  addEventListener("online", () => { void current.check(); });
  document.addEventListener("visibilitychange", () => { if (!document.hidden) void current.check(); });
}

export function applyAppUpdate() { return current?.apply(); }
