import init, { extractArticle } from "./extractor/reader_extract.js";
import { downloadArticle } from "./article-download.js";

let ready;
self.onmessage = async ({ data }) => {
  try {
    ready ??= init().catch(error => { ready = undefined; throw error; });
    // Do not leave a fetch running in a reused worker if initialization fails.
    await ready;
    const document = await downloadArticle(data);
    const result = JSON.parse(extractArticle(document.html, document.base_url));
    self.postMessage({ request_id: data.request_id, ...result });
  } catch (error) {
    self.postMessage({ request_id: data.request_id, error: String(error?.message || error).slice(0, 500), retry_after: Number(error?.retryAfter) || 0 });
  }
};
