// Bounded workers isolate both HTML parsing and large downloads from the UI.
// Cancellation terminates the worker, including a parser which cannot yield.
export class ArticleRequests {
  constructor({ receive, error, workerFactory = () => new Worker(new URL("./article-worker.js", import.meta.url), { type: "module" }), timeout = 35_000 }) {
    this.receive = receive; this.error = error; this.workerFactory = workerFactory;
    this.timeout = timeout; this.pending = []; this.slots = []; this.retryAfter = 0;
  }
  request(message) {
    if (this.pending.length >= 8) { this.error(message, "Finish or cancel another article download before starting more."); return; }
    this.pending.push(message); this.pump();
  }
  cancel(id) {
    this.pending = this.pending.filter(item => item.request_id !== id);
    const slot = this.slots.find(slot => slot.message?.request_id === id);
    if (slot) this.retire(slot);
    this.pump();
  }
  dispose() {
    this.pending = [];
    for (const slot of [...this.slots]) this.retire(slot);
  }
  retire(slot) {
    clearTimeout(slot.timer); slot.message = null; slot.worker.terminate();
    this.slots = this.slots.filter(item => item !== slot);
  }
  pump() {
    while (this.pending.length) {
      let slot = this.slots.find(slot => !slot.message);
      if (!slot && this.slots.length >= 2) return;
      const message = this.pending.shift();
      if (!slot) {
        try {
          slot = { worker: this.workerFactory(), message: null, timer: null };
          this.slots.push(slot);
          const current = slot;
          slot.worker.onmessage = event => this.complete(current, event.data);
          slot.worker.onerror = event => { event.preventDefault?.(); this.fail(current, "The article reader could not start. Reload Reader and try again."); };
          slot.worker.onmessageerror = () => this.fail(current, "The article could not be transferred. Try again.");
        } catch { this.error(message, "Article reading requires Web Worker support. Reload Reader or use View original."); continue; }
      }
      slot.message = message;
      const current = slot;
      slot.timer = setTimeout(() => this.fail(current, "The article took too long to load. Try again or use View original."), this.timeout);
      try { slot.worker.postMessage({ ...message, public_retry_after: this.retryAfter }); }
      catch { this.fail(slot, "The article reader could not accept this request. Try again."); }
    }
  }
  complete(slot, result) {
    const message = slot.message;
    if (!message || message.request_id !== result?.request_id) return;
    this.retryAfter = Math.max(this.retryAfter, Math.min(Number(result.retry_after) || 0, Date.now() + 3_600_000));
    clearTimeout(slot.timer); slot.message = null;
    if (result.error) this.error(message, String(result.error));
    else if (typeof result.html === "string" && typeof result.base_url === "string") this.receive(message, result);
    else this.error(message, "The publisher did not return a readable article.");
    this.pump();
  }
  fail(slot, error) {
    const message = slot.message;
    this.retire(slot);
    if (message) this.error(message, error);
    this.pump();
  }
}
