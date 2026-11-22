import test from "node:test";
import assert from "node:assert/strict";
import { ArticleRequests } from "../crates/reader-browser/wwwroot/articles.js";

function fixture(timeout = 60_000) {
  const workers = [], received = [], errors = [];
  const requests = new ArticleRequests({ timeout,
    receive: (request, result) => received.push([request, result]),
    error: (request, message) => errors.push([request, message]),
    workerFactory() {
      const worker = { messages: [], stopped: false,
        postMessage(value) { this.messages.push(value); }, terminate() { this.stopped = true; },
        answer(value) { this.onmessage({ data: value }); } };
      workers.push(worker); return worker;
    },
  });
  return { requests, workers, received, errors };
}
const request = request_id => ({ request_id, id: 80 + request_id, generation: 4, url: "https://example.com/story" });
const result = request_id => ({ request_id, html: "<p>Readable article</p>", base_url: "https://example.com/story" });

test("at most two article jobs run and successful workers serve queued requests", () => {
  const f = fixture();
  try {
    for (let id = 1; id <= 5; id++) f.requests.request(request(id));
    assert.equal(f.workers.length, 2);
    f.workers[0].answer(result(1));
    assert.deepEqual(f.workers[0].messages.map(message => message.request_id), [1, 3]);
    assert.equal(f.received[0][0].generation, 4);
    assert.equal(f.received[0][1].html, result(1).html);
    assert.equal(f.workers.length, 2);
  } finally { f.requests.dispose(); }
});

test("cancel terminates an active parser and discards its late response", () => {
  const f = fixture();
  try {
    f.requests.request(request(1)); f.requests.request(request(2)); f.requests.request(request(3));
    f.requests.cancel(1);
    assert(f.workers[0].stopped); assert.equal(f.workers[2].messages[0].request_id, 3);
    f.workers[0].answer(result(1)); assert.equal(f.received.length, 0);
    f.workers[2].answer(result(3)); assert.equal(f.received.length, 1);
  } finally { f.requests.dispose(); }
});

test("cancelled queued jobs never start and wrong request replies cannot update the current job", () => {
  const f = fixture();
  try {
    for (let id = 1; id <= 3; id++) f.requests.request(request(id));
    f.requests.cancel(3); f.workers[0].answer(result(99));
    assert.equal(f.received.length, 0); assert.equal(f.requests.slots[0].message.request_id, 1);
    f.workers[0].answer(result(1)); assert.equal(f.workers[0].messages.length, 1);
  } finally { f.requests.dispose(); }
});

test("an unresponsive parser is terminated without blocking the next article", async () => {
  const f = fixture(15);
  try {
    f.requests.request(request(1));
    await new Promise(resolve => setTimeout(resolve, 35));
    assert(f.workers[0].stopped); assert.match(f.errors[0][1], /too long/);
    f.requests.request(request(2)); assert.equal(f.workers[1].messages[0].request_id, 2);
  } finally { f.requests.dispose(); }
});

test("failed workers are retired and queue overflow reports an actionable article error", () => {
  const f = fixture();
  try {
    for (let id = 1; id <= 11; id++) f.requests.request(request(id));
    assert.equal(f.errors.length, 1); assert.equal(f.errors[0][0].request_id, 11);
    f.workers[0].onerror({ preventDefault() {} });
    assert(f.workers[0].stopped); assert.equal(f.workers[2].messages[0].request_id, 3);
  } finally { f.requests.dispose(); }
});

test("public-service cooldown reaches other and replacement workers without blocking direct downloads", () => {
  const f = fixture();
  try {
    for (let id = 1; id <= 3; id++) f.requests.request(request(id));
    const deadline = Date.now() + 120_000;
    f.workers[0].answer({ request_id: 1, error: "Rate limited", retry_after: deadline });
    assert.equal(f.workers[0].messages[1].public_retry_after, deadline);
    f.requests.cancel(3);
    f.requests.request(request(4));
    assert.equal(f.workers[2].messages[0].public_retry_after, deadline);
    f.workers[1].answer(result(2));
    f.requests.request(request(5));
    assert.equal(f.workers[1].messages[1].public_retry_after, deadline);
  } finally { f.requests.dispose(); }
});
