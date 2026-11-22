// Publisher HTML is fetched and parsed only inside the article worker.
import { httpUrl, proxyUrl, publicFeedEligible } from "./feed-connection.js";

export const MAX_DOCUMENT = 4 * 1024 * 1024;
export const ARTICLE_SERVICE = "Jina Reader";
const PUBLIC_ENDPOINT = "https://r.jina.ai/";
let retryAfter = 0;

class ArticleRequestError extends Error {
  constructor(kind, message, retryAfter = 0) { super(message); this.name = "ArticleRequestError"; this.kind = kind; this.retryAfter = retryAfter; }
}

export function articleConnection(address, proxy = "", mode = "Automatic") {
  if (mode === "Direct") return null;
  if (mode !== "Automatic" && mode !== "Custom")
    throw new ArticleRequestError("configuration", "Choose a connection mode in Reader settings.");
  if (proxy.trim() || mode === "Custom") {
    if (!proxy.trim()) throw new ArticleRequestError("configuration", "Enter a trusted proxy in Reader settings, or choose Automatic.");
    return { url: proxyUrl(proxy.trim(), address), source: "your custom proxy", public: false };
  }
  if (!publicFeedEligible(address)) return { blocked: true };
  return { url: PUBLIC_ENDPOINT + address, source: ARTICLE_SERVICE, public: true };
}

async function download(url, { timeout, source, headers, public: publicRelay = false } = {}) {
  const controller = new AbortController();
  const timer = setTimeout(() => controller.abort(), timeout);
  let reader;
  try {
    let response;
    try {
      response = await fetch(url, { signal: controller.signal, credentials: "omit", referrerPolicy: "no-referrer", headers });
    } catch {
      throw new ArticleRequestError("network", `${source} could not be reached or does not allow browser access.`);
    }
    if (!response.ok) {
      await response.body?.cancel().catch(() => {});
      if (response.status === 429) {
        const value = response.headers.get("retry-after");
        const seconds = Number(value);
        const delay = value ? (Number.isFinite(seconds) ? seconds * 1000 : Date.parse(value) - Date.now()) : 60_000;
        if (publicRelay) retryAfter = Date.now() + Math.max(1_000, Math.min(Number.isFinite(delay) ? delay : 60_000, 3_600_000));
        throw new ArticleRequestError("rate", `${source} is busy. Wait a little before trying again, or use View original.`, publicRelay ? retryAfter : 0);
      }
      if (response.status === 401 || response.status === 403)
        throw new ArticleRequestError("access", "The publisher requires sign-in or blocked this request. Use View original to continue.");
      throw new ArticleRequestError("http", `${source} returned HTTP ${response.status}. Use View original or try again later.`);
    }
    const contentType = response.headers.get("content-type") || "";
    if (contentType && !/(?:text\/(?:html|plain)|application\/xhtml\+xml)/i.test(contentType)) {
      await response.body?.cancel().catch(() => {});
      throw new ArticleRequestError("format", "This link is not an HTML article. Use View original to open the file.");
    }
    const oversized = () => new ArticleRequestError("size", "The publisher page exceeds the 4 MB reading limit.");
    if (Number(response.headers.get("content-length")) > MAX_DOCUMENT) {
      await response.body?.cancel().catch(() => {});
      throw oversized();
    }
    reader = response.body?.getReader();
    const chunks = [];
    let total = 0;
    if (reader) {
      while (true) {
        const { value, done } = await reader.read();
        if (done) break;
        total += value.byteLength;
        if (total > MAX_DOCUMENT) throw oversized();
        chunks.push(value);
      }
    } else {
      const bytes = new Uint8Array(await response.arrayBuffer());
      if (bytes.byteLength > MAX_DOCUMENT) throw oversized();
      chunks.push(bytes); total = bytes.byteLength;
    }
    const bytes = new Uint8Array(total);
    let offset = 0;
    for (const chunk of chunks) { bytes.set(chunk, offset); offset += chunk.byteLength; }
    const prefix = new TextDecoder("ascii").decode(bytes.slice(0, 2048));
    const charset = /charset\s*=\s*["']?([^\s;"'>]+)/i.exec(contentType)?.[1]
      || /<meta[^>]*charset\s*=\s*["']?([^\s;"'>]+)/i.exec(prefix)?.[1] || "utf-8";
    let html;
    try { html = new TextDecoder(charset).decode(bytes); }
    catch { throw new ArticleRequestError("encoding", "The publisher uses an unsupported text encoding. Use View original."); }
    return { html, base_url: response.url || url };
  } catch (error) {
    await reader?.cancel().catch(() => {});
    if (error instanceof ArticleRequestError) throw error;
    throw new ArticleRequestError("network", `${source} stopped responding. Try again or use View original.`);
  } finally { clearTimeout(timer); }
}

export async function downloadArticle(message) {
  if (globalThis.navigator?.onLine === false)
    throw new ArticleRequestError("offline", "You are offline. Saved articles remain readable; connect to download this article.");
  const address = httpUrl(message.url);
  const fallback = articleConnection(address, message.proxy, message.connection);
  try {
    return await download(address, { timeout: 8000, source: "The publisher" });
  } catch (error) {
    // HTTP refusal, missing pages, size limits, and unsupported files are terminal.
    if (error.kind !== "network") throw error;
    if (!fallback) throw new ArticleRequestError("cors", "The publisher does not allow browser access. Choose Automatic or a trusted proxy in Reader settings, or use View original.");
    if (fallback.blocked) throw new ArticleRequestError("private", "This address is not sent to a public service. Configure your trusted proxy in Reader settings, or use View original.");
  }
  if (fallback.public && Date.now() < Math.max(retryAfter, Number(message.public_retry_after) || 0))
    throw new ArticleRequestError("rate", `${ARTICLE_SERVICE} asked Reader to wait. Try again later or use View original.`);
  const result = await download(fallback.url, {
    timeout: 22_000, source: fallback.source, public: fallback.public,
    headers: fallback.public ? { "X-Respond-With": "html", "X-Respond-Timing": "html", "Accept": "text/html" } : undefined,
  });
  // A relay URL is never the base of publisher-relative links or images.
  return { ...result, base_url: address };
}
