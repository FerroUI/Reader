export class FeedRequestError extends Error {
  constructor(code, message, details = {}) {
    super(message);
    this.name = "FeedRequestError";
    this.code = code;
    Object.assign(this, details);
  }
}

export function retryAfterMilliseconds(value, now = Date.now()) {
  if (!value) return 0;
  const seconds = Number(value);
  const delay = Number.isFinite(seconds) ? seconds * 1000 : Date.parse(value) - now;
  return Number.isFinite(delay) ? Math.max(0, Math.min(delay, 86400000)) : 0;
}

export function responseError(response, source) {
  const status = response.status;
  const retryAfter = retryAfterMilliseconds(response.headers.get("retry-after"));
  let message = `${source} returned HTTP ${status}.`;
  if (status === 404) message = `The feed address was not found (HTTP 404 from ${source}).`;
  if (status === 401 || status === 403) message = `${source} denied access (HTTP ${status}). Check that this is a public feed.`;
  if (status === 429) message = `${source} is rate limited (HTTP 429). ${retryAfter ? `Try again in ${Math.ceil(retryAfter / 1000)} seconds.` : "Please wait before trying again."}`;
  return new FeedRequestError("http", message, { status, retryAfter });
}

export function networkError(error, source, timedOut) {
  if (globalThis.navigator?.onLine === false)
    return new FeedRequestError("offline", "You are offline. Connect to the internet, then try adding or refreshing the feed again.");
  if (timedOut || error?.name === "AbortError")
    return new FeedRequestError("timeout", `${source} took too long to respond. Try again shortly.`);
  return new FeedRequestError("network", `Could not connect to ${source}.`);
}
