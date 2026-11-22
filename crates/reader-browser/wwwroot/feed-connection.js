import { FeedRequestError } from "./feed-errors.js";
import { PUBLIC_FEED_ENDPOINT, PUBLIC_FEED_SERVICE } from "./feed-public.js";

export function httpUrl(value, base) {
  let url;
  try { url = new URL(value, base); }
  catch { throw new FeedRequestError("url", "Enter a valid HTTP or HTTPS feed or website address."); }
  if (!/^https?:$/.test(url.protocol) || url.username || url.password)
    throw new FeedRequestError("url", "Use an HTTP or HTTPS feed URL without an embedded username or password.");
  url.hash = "";
  return url.href;
}

export function proxyUrl(proxy, url) {
  if (!proxy) return null;
  if (!proxy.includes("{url}")) throw new FeedRequestError("configuration", "The feed proxy address must contain {url}.");
  const endpoint = httpUrl(proxy.replaceAll("{url}", encodeURIComponent(url)));
  if (globalThis.location?.protocol === "https:" && new URL(endpoint).protocol !== "https:")
    throw new FeedRequestError("configuration", "The custom feed proxy must use HTTPS when Reader is opened over HTTPS.");
  return endpoint;
}

// Public relays must not receive local addresses or recognizable private links.
// Arbitrary secret paths cannot be inferred; the connection UI also explains
// provider visibility and lets users select Direct or their own trusted proxy.
export function publicFeedEligible(address) {
  const url = new URL(address), host = url.hostname.toLowerCase().replace(/\.+$/, "");
  if (!host.includes(".") || host.startsWith("[") || /^(?:\d+\.){3}\d+$/.test(host)
      || /(?:^|\.)(?:localhost|local|localdomain|lan|internal|home|home\.arpa|onion|test|invalid)$/.test(host)) return false;
  const sensitive = /token|secret|auth|password|passwd|credential|signature|session|private|api[_-]?key|access[_-]?key|(?:^|[_-])(?:key|sig|jwt)(?:$|[_-])/i;
  for (const key of url.searchParams.keys()) if (sensitive.test(key)) return false;
  let path;
  try { path = decodeURIComponent(url.pathname); } catch { return false; }
  return !/(?:^|\/)(?:private(?:[-_][^/]*)?|token|secret|auth|credentials?)(?:\/|$)|\/[a-f\d]{32,}(?:[/.]|$)/i.test(path);
}

export function feedConnection(address, proxy = "", mode = "Automatic") {
  if (mode === "Direct") return null;
  if (mode !== "Automatic" && mode !== "Custom") throw new FeedRequestError("configuration", "Choose a feed connection in Reader settings.");
  proxy = proxy.trim();
  if (proxy || mode === "Custom") {
    if (!proxy) throw new FeedRequestError("configuration", "Enter a custom feed proxy URL in Reader settings, or choose Automatic.");
    return { url: proxyUrl(proxy, address), source: "the custom feed proxy", public: false };
  }
  if (!publicFeedEligible(address)) return { blocked: true };
  return { url: proxyUrl(PUBLIC_FEED_ENDPOINT, address), source: PUBLIC_FEED_SERVICE, public: true };
}
