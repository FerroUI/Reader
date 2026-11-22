import { download, decodeFeed } from "./feed-download.js";
import { httpUrl, feedConnection } from "./feed-connection.js";
import { discoverFeeds, isHtmlDocument } from "./feed-discovery.js";
import { FeedRequestError } from "./feed-errors.js";
import { normalizePublicFeed } from "./feed-public.js";
export { httpUrl, proxyUrl } from "./feed-connection.js";
export { decodeFeed } from "./feed-download.js";
export { discoverFeed } from "./feed-discovery.js";

const inFlight = new Map(), cooldowns = new Map();
const remaining = (deadline, maximum) => {
  const time = Math.min(maximum, deadline - Date.now());
  if (time <= 0) throw new FeedRequestError("timeout", "Finding this feed took too long. Paste its direct RSS or Atom address and try again.");
  return time;
};

async function fetchText(address, proxy, mode, deadline) {
  // Validate configured endpoints before requests; direct/private feeds remain
  // usable without sending their addresses to the public service.
  const connection = feedConnection(address, proxy, mode);
  try { return decodeFeed(await download(address, { timeout: remaining(deadline, 8000) })); }
  catch (error) {
    if (!["network", "timeout"].includes(error.code)) throw error;
    if (!connection) throw new FeedRequestError(error.code,
      `${error.message} This site may not allow direct browser access. Choose Automatic or a trusted custom proxy in Reader settings.`);
    if (connection.blocked) throw new FeedRequestError("private",
      "This address was not sent to a public feed service. For local or private feeds, use direct access or configure a trusted custom proxy in Reader settings.");
  }
  const key = new URL(connection.url).origin;
  const wait = (cooldowns.get(key) || 0) - Date.now();
  if (wait > 0) throw new FeedRequestError("http", `${connection.source} is rate limited. Try again in ${Math.ceil(wait / 1000)} seconds.`, { status: 429, retryAfter: wait });
  try {
    const response = decodeFeed(await download(connection.url, { timeout: remaining(deadline, 15000), source: connection.source }));
    if (connection.public) {
      let contents;
      try { contents = normalizePublicFeed(response.contents, address); }
      catch (error) { throw new FeedRequestError("service", `${error.message} Paste the direct RSS or Atom feed address, or choose a custom feed proxy in Reader settings.`); }
      return { contents, url: address, contentType: "application/feed+json" };
    }
    // A raw proxy's response.url is its own address, not the document's base.
    return { ...response, url: address };
  } catch (error) {
    if (error.status === 429) cooldowns.set(key, Date.now() + (error.retryAfter || 60000));
    if (error.code === "network") throw new FeedRequestError("network", `${connection.source} is unavailable. Try again shortly or choose a different feed connection in Reader settings.`);
    throw error;
  }
}

export async function fetchFeed(address, proxy = "", connection = "Automatic") {
  const url = httpUrl(address.trim()), key = `${connection}\0${proxy}\0${url}`;
  if (inFlight.has(key)) return inFlight.get(key);
  const task = (async () => {
    const deadline = Date.now() + 30000;
    const response = await fetchText(url, proxy, connection, deadline);
    if (isHtmlDocument(response)) {
      let failure;
      for (const candidate of discoverFeeds(response.contents, response.url)) {
        try {
          const feed = await fetchText(candidate, proxy, connection, deadline);
          if (isHtmlDocument(feed)) throw new FeedRequestError("discovery", "The advertised feed returned another web page.");
          return { url: feed.url, contents: feed.contents };
        } catch (error) {
          if (error.code === "offline" || error.status === 429 || error.code === "configuration") throw error;
          failure = error;
        }
      }
      throw failure;
    }
    return { url: response.url, contents: response.contents };
  })().finally(() => inFlight.delete(key));
  inFlight.set(key, task);
  return task;
}

export async function fetchImage(url) { return (await download(httpUrl(url), { limit: 12 * 1024 * 1024, source: "the image server" })).bytes; }

export async function refreshFeeds(feeds, proxy, receive, failure, connection = "Automatic") {
  let cursor = 0;
  await Promise.all(Array.from({ length: Math.min(4, feeds.length) }, async () => {
    while (cursor < feeds.length) {
      const feed = feeds[cursor++];
      try { const result = await fetchFeed(feed.url, proxy, connection); receive(feed.url, result.contents, feed.id, result.url); }
      catch (error) { failure(feed.url, error?.message || String(error)); }
    }
  }));
}
