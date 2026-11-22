import { FeedRequestError, networkError, responseError } from "./feed-errors.js";

export const MAX_FEED = 8 * 1024 * 1024;

export async function download(url, { limit = MAX_FEED, timeout = 15000, source = "the feed server" } = {}) {
  const controller = new AbortController();
  let timedOut = false, reader;
  const timer = setTimeout(() => { timedOut = true; controller.abort(); }, timeout);
  try {
    let response;
    try {
      response = await fetch(url, { signal: controller.signal, credentials: "omit", referrerPolicy: "no-referrer" });
    } catch (error) { throw networkError(error, source, timedOut); }
    if (!response.ok) {
      await response.body?.cancel().catch(() => {});
      throw responseError(response, source);
    }
    const tooLarge = () => new FeedRequestError("size", `This resource exceeds the ${limit / 1024 / 1024} MB download limit.`);
    if (Number(response.headers.get("content-length")) > limit) {
      await response.body?.cancel().catch(() => {});
      throw tooLarge();
    }
    const result = bytes => ({ bytes, url: response.url || url, contentType: response.headers.get("content-type") || "" });
    reader = response.body?.getReader();
    if (!reader) {
      const bytes = new Uint8Array(await response.arrayBuffer());
      if (bytes.byteLength > limit) throw tooLarge();
      return result(bytes);
    }
    let total = 0;
    const chunks = [];
    while (true) {
      const { done, value } = await reader.read();
      if (done) break;
      total += value.byteLength;
      if (total > limit) throw tooLarge();
      chunks.push(value);
    }
    const bytes = new Uint8Array(total);
    let offset = 0;
    for (const chunk of chunks) { bytes.set(chunk, offset); offset += chunk.byteLength; }
    return result(bytes);
  } catch (error) {
    await reader?.cancel().catch(() => {});
    if (error instanceof FeedRequestError) throw error;
    throw networkError(error, source, timedOut);
  } finally { clearTimeout(timer); }
}

export function decodeFeed({ bytes, url, contentType = "" }) {
  const header = new TextDecoder("ascii").decode(bytes.slice(0, 256));
  const bom = bytes[0] === 255 && bytes[1] === 254 ? "utf-16le" : bytes[0] === 254 && bytes[1] === 255 ? "utf-16be" : null;
  const encoding = bom || /charset\s*=\s*["']?([^\s;"']+)/i.exec(contentType)?.[1]
    || /<\?xml[^>]*encoding\s*=\s*["']([^"']+)/i.exec(header)?.[1] || "utf-8";
  try { return { contents: new TextDecoder(encoding).decode(bytes), url, contentType }; }
  catch { throw new FeedRequestError("encoding", `This feed uses an unsupported text encoding: ${encoding.slice(0, 80)}.`); }
}
