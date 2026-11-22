import { httpUrl } from "./feed-connection.js";
import { FeedRequestError } from "./feed-errors.js";

function documentStart(contents) {
  return contents.replace(/^\uFEFF/, "").replace(/^\s*(?:(?:<!--[\s\S]*?-->|<\?[\s\S]*?\?>)\s*)*/, "");
}

export function isHtmlDocument({ contents, contentType = "" }) {
  const start = documentStart(contents);
  if (/^(?:\{|<(?:rss|feed|(?:rdf:)?RDF)(?:\s|>))/i.test(start)) return false;
  return /(?:text\/html|application\/xhtml\+xml)/i.test(contentType)
    || /^<(?:!doctype\s+html|html|head|body)(?:\s|>)/i.test(start);
}

export function discoverFeeds(html, address) {
  const document = new DOMParser().parseFromString(html, "text/html");
  let base = address;
  const baseHref = document.querySelector("base[href]")?.getAttribute("href");
  if (baseHref) { try { base = httpUrl(baseHref, address); } catch { /* Ignore an unusable document base. */ } }
  const feeds = [];
  for (const link of document.querySelectorAll("link[href]")) {
    if (!link.rel?.toLowerCase().split(/\s+/).includes("alternate")
        || !/(?:rss|atom)\+xml|feed\+json|application\/json/i.test(link.type)) continue;
    try {
      const url = httpUrl(link.getAttribute("href"), base);
      if (url !== address && !feeds.includes(url)) feeds.push(url);
    } catch { /* One invalid advertised link must not hide a working feed. */ }
    if (feeds.length === 4) break;
  }
  if (!feeds.length) throw new FeedRequestError("discovery", "No RSS, Atom, or JSON Feed was found on this page. Paste its feed address.");
  return feeds;
}

export function discoverFeed(html, base) { return discoverFeeds(html, base)[0]; }
