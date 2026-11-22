import test from "node:test";
import assert from "node:assert/strict";
import { expectedNetworkFailure } from "./ui/harness.mjs";

const origin = "http://127.0.0.1:36431";
const publisher = "http://127.0.0.1:35091/publisher/index.html";
const missing = "http://127.0.0.1:35092/relay?url=missing";
const scope = () => ({origin, cors: new Set([publisher]), http: new Map([[missing, new Set([404])]]), requests: new Map([["publisher-request", publisher]])});
const corsEntry = (changes = {}) => ({level: "error", source: "security", url: `${origin}/index.html?nooffline=1#home`,
  text: `Access to fetch at '${publisher}' from origin '${origin}' has been blocked by CORS policy: No 'Access-Control-Allow-Origin' header is present on the requested resource.`, ...changes});

test("exact Chromium CORS errors match their embedded publisher URL, not the calling page URL", () => {
  for (const source of ["security", "javascript", "network"]) {
    const entry = corsEntry({source});
    assert.deepEqual(expectedNetworkFailure(entry, scope()), {kind: "cors-policy", url: publisher, text: entry.text});
  }
});

test("CORS allowance rejects another target, caller, source, or unrelated script error", () => {
  const entry = corsEntry();
  for (const changes of [
    {text: entry.text.replace(publisher, `${publisher}?unexpected=1`)},
    {text: entry.text.replace(`origin '${origin}'`, "origin 'https://unexpected.example'")},
    {source: "console-api"}, {level: "warning"},
    {text: "ReferenceError: blocked by CORS policy"},
    {text: `${entry.text} Additional unrelated error.`},
  ]) assert.equal(expectedNetworkFailure(corsEntry(changes), scope()), null);
});

test("ERR_FAILED requires a network entry tied to the exact fixture URL or request", () => {
  const entry = {level: "error", source: "network", text: "Failed to load resource: net::ERR_FAILED", url: publisher};
  assert.equal(expectedNetworkFailure(entry, scope()).kind, "cors-load");
  assert.equal(expectedNetworkFailure({...entry, url: undefined, networkRequestId: "publisher-request"}, scope()).url, publisher);
  assert.equal(expectedNetworkFailure({...entry, url: `${publisher}?unexpected=1`}, scope()), null);
  assert.equal(expectedNetworkFailure({...entry, source: "javascript"}, scope()), null);
});

test("HTTP failures require the declared URL and status and remain network-only", () => {
  const entry = {level: "error", source: "network", url: missing, text: "Failed to load resource: the server responded with a status of 404 (Not Found)"};
  assert.equal(expectedNetworkFailure(entry, scope()).kind, "http-status");
  assert.equal(expectedNetworkFailure({...entry, source: "security"}, scope()), null);
  assert.equal(expectedNetworkFailure({...entry, url: publisher}, scope()), null);
  assert.equal(expectedNetworkFailure({...entry, text: entry.text.replace("404", "503")}, scope()), null);
});
