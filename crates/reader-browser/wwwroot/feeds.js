const MAX_FEED = 8 * 1024 * 1024;
const inFlight = new Map();
export function httpUrl(value, base) {
  const url = new URL(value, base);
  if (!/^https?:$/.test(url.protocol) || url.username || url.password) throw new Error("Use a public HTTP or HTTPS feed URL.");
  url.hash = "";
  return url.href;
}
export function proxyUrl(proxy, url) {
  if (!proxy) return null;
  if (!proxy.includes("{url}")) throw new Error("The feed proxy address must contain {url}.");
  return httpUrl(proxy.replaceAll("{url}", encodeURIComponent(url)));
}
async function download(url, limit = MAX_FEED) {
  const controller = new AbortController();
  const timer = setTimeout(() => controller.abort(), 15000);
  try {
    const response = await fetch(url, { signal: controller.signal, credentials: "omit", referrerPolicy: "no-referrer" });
    if (!response.ok) throw new Error(`The server returned HTTP ${response.status}.`);
    if (Number(response.headers.get("content-length")) > limit) throw new Error("This resource is too large.");
    const result = bytes => ({ bytes, url: response.url || url, contentType: response.headers.get("content-type") || "" });
    const reader = response.body?.getReader();
    if (!reader) {
      const bytes = new Uint8Array(await response.arrayBuffer());
      if (bytes.byteLength > limit) throw new Error("This resource is too large.");
      return result(bytes);
    }
    let total = 0; const chunks = [];
    while (true) {
      const { done, value } = await reader.read();
      if (done) break;
      total += value.byteLength;
      if (total > limit) { await reader.cancel(); throw new Error("This resource is too large."); }
      chunks.push(value);
    }
    const bytes = new Uint8Array(total); let offset = 0;
    for (const chunk of chunks) { bytes.set(chunk, offset); offset += chunk.byteLength; }
    return result(bytes);
  } finally { clearTimeout(timer); }
}
async function fetchText(url, proxy) {
  try { return decodeFeed(await download(url)); }
  catch (error) {
    const via = proxyUrl(proxy, url);
    if (via) return { ...decodeFeed(await download(via)), url };
    throw new Error(`${error.message} This feed may block cross-origin access. Set a trusted feed proxy in Reader settings.`);
  }
}
export function decodeFeed({ bytes, url, contentType = "" }) {
  const header = new TextDecoder("ascii").decode(bytes.slice(0, 256));
  const encoding = /charset\s*=\s*["']?([^\s;"']+)/i.exec(contentType)?.[1]
    || /<\?xml[^>]*encoding\s*=\s*["']([^"']+)/i.exec(header)?.[1]
    || (bytes[0] === 255 && bytes[1] === 254 ? "utf-16le" : bytes[0] === 254 && bytes[1] === 255 ? "utf-16be" : "utf-8");
  return { contents: new TextDecoder(encoding).decode(bytes), url };
}
export function discoverFeed(html, base) {
  const document = new DOMParser().parseFromString(html, "text/html");
  const links = [...document.querySelectorAll('link[rel~="alternate"][href]')];
  const link = links.find(l => /(?:rss|atom)\+xml|feed\+json|application\/json/i.test(l.type));
  if (!link) throw new Error("No RSS, Atom, or JSON Feed was found on this page. Paste its feed address.");
  const baseHref = document.querySelector("base[href]")?.getAttribute("href");
  return httpUrl(link.getAttribute("href"), baseHref ? httpUrl(baseHref, base) : base);
}
export async function fetchFeed(address, proxy = "") {
  let url = httpUrl(address.trim());
  const key = `${proxy}\0${url}`;
  if (inFlight.has(key)) return inFlight.get(key);
  const task = (async () => {
    let response = await fetchText(url, proxy);
    let contents = response.contents;
    url = response.url;
    if (/^\s*(?:<!doctype\s+html|<html[\s>])/i.test(contents)) {
      url = discoverFeed(contents, url);
      response = await fetchText(url, proxy);
      contents = response.contents;
      url = response.url;
    }
    return { url, contents };
  })().finally(() => inFlight.delete(key));
  inFlight.set(key, task);
  return task;
}
export async function fetchImage(url) { return (await download(httpUrl(url), 12 * 1024 * 1024)).bytes; }
export async function refreshFeeds(feeds, proxy, receive, failure) {
  let cursor = 0;
  await Promise.all(Array.from({ length: Math.min(4, feeds.length) }, async () => {
    while (cursor < feeds.length) {
      const feed = feeds[cursor++];
      try { const result = await fetchFeed(feed.url, proxy); receive(feed.url, result.contents, feed.id, result.url); }
      catch (error) { failure(feed.url, error.message); }
    }
  }));
}
