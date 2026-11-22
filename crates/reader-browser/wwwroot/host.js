import { load, save, preserveRecovery, flushStorage, setStorageReporter } from "./reader-storage.js";
import { fetchFeed, fetchImage, refreshFeeds } from "./feeds.js";
import { downloadFile, pickFile, openUrl, copyText, share, email, printArticle } from "./transfers.js";
import { ArticleRequests } from "./articles.js";
export { load, save, preserveRecovery };
export { navigationChanged } from "./navigation.js";
export { inputHintsChanged } from "./input.js";
export const now = () => Math.floor(Date.now() / 1000);
export const width = () => document.getElementById("out").getBoundingClientRect().width;
export const height = () => document.getElementById("out").getBoundingClientRect().height;
let runtime, saveTimer, pendingOperations = 0, latestSubscription = 0, operationGeneration;
const articles = new ArticleRequests({
  receive: (message, result) => runtime.receiveArticle(message.id, message.request_id, message.generation, message.url, result.html, result.base_url),
  error: (message, error) => runtime.articleError(message.id, message.request_id, message.generation, message.url, error),
});
export function attachRuntime(value) {
  articles.dispose();
  runtime = value;
  setStorageReporter(error => runtime.storageError(error?.message || String(error)));
}
export function reportError(error) { runtime?.hostError(error?.message || String(error)); }
export function flush() {
  clearTimeout(saveTimer);
  try { runtime?.flushSave(); } catch (error) { reportError(error); }
  return flushStorage();
}
export function request(json) {
  let message;
  try { message = JSON.parse(json); } catch (error) { reportError(error); return; }
  // Keep user-gesture-sensitive file/share/window operations in the input callback.
  switch (message.kind) {
    case "fetchArticle": articles.request(message); return;
    case "cancelArticle": articles.cancel(message.request_id); return;
    case "export": try { downloadFile(message.filename, message.contents, message.mime_type); } catch (e) { reportError(e); } return;
    case "import": try { pickFile(message.format, runtime); } catch (e) { reportError(e); } return;
    case "openUrl": try { openUrl(message.url); } catch (e) { reportError(e); } return;
    case "email": try { email(message.title, message.url); } catch (e) { reportError(e); } return;
    case "print": try { printArticle(message.title, message.html, message.base_url); } catch (e) { reportError(e); } return;
    case "share": void share(message.title, message.url).then(s => s && runtime.hostStatus(s)).catch(reportError); return;
    case "copy": void copyText(message.contents).then(() => runtime.hostStatus("Copied to clipboard")).catch(reportError); return;
    case "scheduleSave": clearTimeout(saveTimer); saveTimer = setTimeout(flush, 150); return;
  }
  const generation = runtime?.libraryGeneration();
  queueMicrotask(() => { void perform(message, generation).catch(reportError); });
}
function beginOperation(generation) {
  if (generation !== runtime.libraryGeneration()) return false;
  if (generation !== operationGeneration) { operationGeneration = generation; pendingOperations = 0; }
  pendingOperations++;
  return true;
}
function finishOperation(generation) {
  if (generation === operationGeneration && --pendingOperations === 0 && generation === runtime.libraryGeneration()) runtime.finishRefresh();
}
async function perform(message, generation) {
  switch (message.kind) {
    case "subscribe": {
      if (!beginOperation(generation)) break;
      const subscription = ++latestSubscription;
      const startingView = runtime.currentView();
      try {
        const result = await fetchFeed(message.url, message.proxy, message.connection);
        if (generation !== runtime.libraryGeneration()) break;
        const open = subscription === latestSubscription && runtime.currentView() === startingView;
        runtime.receiveFeed(result.url, result.contents, open, 0, result.url);
      } catch (error) { if (generation === runtime.libraryGeneration()) runtime.feedError(message.url, error?.message || String(error)); }
      finally { finishOperation(generation); }
      break;
    }
    case "refresh":
      if (!beginOperation(generation)) break;
      try { await refreshFeeds(message.feeds, message.proxy,
        (url, xml, id, base) => { if (generation === runtime.libraryGeneration()) runtime.receiveFeed(url, xml, false, id, base); },
        (url, error) => { if (generation === runtime.libraryGeneration()) runtime.feedError(url, error); }, message.connection); }
      finally { finishOperation(generation); }
      break;
    case "fetchImage":
      try { runtime.receiveImage(message.url, await fetchImage(message.url)); }
      catch { runtime.receiveImage(message.url, new Uint8Array()); }
      break;
    case "focusSearch": runtime.focusSearch(); break;
    case "scrollToArticle": requestAnimationFrame(() => runtime.scrollToArticle(message.id)); break;
    default: throw new Error(`Unsupported Reader operation: ${message.kind}`);
  }
}
