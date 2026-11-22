// Normalize the public service's response at the browser boundary. The shared
// Rust JSON Feed parser retains ownership of article validation and merging.
export const PUBLIC_FEED_SERVICE = "rss2json";
export const PUBLIC_FEED_ENDPOINT = "https://api.rss2json.com/v1/api.json?rss_url={url}";
const MAX_ITEMS = 5000;
const object = value => value !== null && typeof value === "object" && !Array.isArray(value);
const string = value => typeof value === "string" && value.length ? value : undefined;

function httpUrl(value, base) {
  if (!string(value)) return undefined;
  try {
    const url = new URL(value, base);
    return /^https?:$/.test(url.protocol) && !url.username && !url.password ? url.href : undefined;
  } catch { return undefined; }
}

function published(value) {
  value = string(value)?.trim();
  if (!value) return undefined;
  // rss2json uses SQL-style timestamps without a zone. Interpret those as UTC,
  // never in the reader's local zone. Reject rollover dates before Date.parse.
  const iso = /^(\d{4})-(\d{2})-(\d{2})(?:[Tt ](\d{2}):(\d{2})(?::(\d{2})(\.\d+)?)?([Zz]|[+-]\d{2}:?\d{2})?)?$/.exec(value);
  if (iso) {
    const [, year, month, day, hour = "00", minute = "00", second = "00", fraction = "", zone = "Z"] = iso;
    const y = Number(year), m = Number(month), d = Number(day);
    const days = [31, y % 4 === 0 && (y % 100 !== 0 || y % 400 === 0) ? 29 : 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    if (m < 1 || m > 12 || d < 1 || d > days[m - 1] || +hour > 23 || +minute > 59 || +second > 59) return undefined;
    value = `${year}-${month}-${day}T${hour}:${minute}:${second}${fraction}${zone}`;
  } else if (!/^(?:[A-Za-z]{3},?\s+)?\d{1,2}\s+[A-Za-z]{3}\s+\d{4}\s+\d{2}:\d{2}(?::\d{2})?\s+(?:[+-]\d{4}|UT|UTC|GMT|EST|EDT|CST|CDT|MST|MDT|PST|PDT)$/i.test(value)) {
    return undefined; // Avoid Date.parse's local-time and ambiguous numeric forms.
  }
  const time = Date.parse(value);
  return Number.isFinite(time) ? new Date(time).toISOString() : undefined;
}

function nonnegative(value, integer = false) {
  if (typeof value !== "number" && !(typeof value === "string" && /^\d+(?:\.\d+)?$/.test(value))) return undefined;
  const number = Number(value);
  return Number.isFinite(number) && number >= 0 && (!integer || Number.isSafeInteger(number)) ? number : undefined;
}

function attachments(value, base) {
  const values = Array.isArray(value) ? value : object(value) ? [value] : [];
  const result = values.filter(object).flatMap(enclosure => {
    const url = httpUrl(enclosure.link ?? enclosure.url, base);
    if (!url) return [];
    return [{ url, mime_type: string(enclosure.type) || "application/octet-stream",
      title: string(enclosure.title), size_in_bytes: nonnegative(enclosure.length, true),
      duration_in_seconds: nonnegative(enclosure.duration) }];
  });
  return result.length ? result : undefined;
}

function article(item, base) {
  const url = httpUrl(item.link, base);
  const author = string(item.author);
  const description = string(item.description);
  const tags = Array.isArray(item.categories) ? item.categories.filter(value => string(value)) : [];
  return {
    id: string(item.guid) || url,
    title: string(item.title), url,
    content_html: string(item.content) || description || "",
    summary: description,
    authors: author ? [{ name: author }] : undefined,
    date_published: published(item.pubDate),
    image: httpUrl(item.thumbnail, url || base),
    tags: tags.length ? tags : undefined,
    attachments: attachments(item.enclosure, url || base),
  };
}

export function normalizePublicFeed(contents, requestedUrl) {
  let data;
  try { data = typeof contents === "string" ? JSON.parse(contents) : null; }
  catch { throw new Error("rss2json returned invalid JSON."); }
  if (!object(data) || typeof data.status !== "string") throw new Error("rss2json returned an invalid feed response.");
  if (data.status !== "ok") {
    const message = string(data.message)?.slice(0, 400);
    throw new Error(message ? `rss2json could not load this feed: ${message}` : "rss2json could not load this feed.");
  }
  if (!object(data.feed) || !Array.isArray(data.items) || data.items.some(item => !object(item))) {
    throw new Error("rss2json returned an invalid feed response.");
  }
  const feedUrl = httpUrl(requestedUrl);
  if (!feedUrl) throw new Error("The public feed service needs an HTTP or HTTPS publisher feed URL.");
  const author = string(data.feed.author);
  return JSON.stringify({
    version: "https://jsonfeed.org/version/1.1",
    // Provider URLs and the provider's own feed.url never become identities.
    feed_url: feedUrl,
    title: string(data.feed.title) || feedUrl,
    home_page_url: httpUrl(data.feed.link, feedUrl),
    description: string(data.feed.description),
    icon: httpUrl(data.feed.image, feedUrl),
    authors: author ? [{ name: author }] : undefined,
    items: data.items.slice(0, MAX_ITEMS).map(item => article(item, feedUrl)),
  });
}
