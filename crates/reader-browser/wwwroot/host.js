import { load, save, preserveRecovery, flushStorage, setStorageReporter } from "./storage.js";
import { fetchFeed, fetchImage, refreshFeeds } from "./feeds.js";
import { downloadFile, pickFile, openUrl, copyText, share, email, printArticle } from "./transfers.js";
export { load, save, preserveRecovery };
export const now = () => Math.floor(Date.now() / 1000);
export const width = () => document.getElementById("out").getBoundingClientRect().width;
let runtime, saveTimer, pendingOperations = 0, latestSubscription = 0;
export function attachRuntime(value) { runtime = value; setStorageReporter(reportError); }
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
    case "export": downloadFile(message.filename, message.contents, message.mime_type); return;
    case "import": pickFile(message.format, runtime); return;
    case "openUrl": try { openUrl(message.url); } catch (e) { reportError(e); } return;
    case "email": try { email(message.title, message.url); } catch (e) { reportError(e); } return;
    case "print": printArticle(message.title, message.html); return;
    case "share": void share(message.title, message.url).then(s => s && runtime.hostStatus(s)).catch(reportError); return;
    case "copy": void copyText(message.contents).then(() => runtime.hostStatus("Copied to clipboard")).catch(reportError); return;
    case "scheduleSave": clearTimeout(saveTimer); saveTimer = setTimeout(flush, 150); return;
  }
  queueMicrotask(() => { void perform(message).catch(reportError); });
}
async function perform(message) {
  switch (message.kind) {
    case "subscribe": {
      const subscription = ++latestSubscription;
      pendingOperations++;
      try {
        const result = await fetchFeed(message.url, message.proxy);
        runtime.receiveFeed(result.url, result.contents, subscription === latestSubscription, 0);
      } catch (error) { runtime.feedError(message.url, error.message); }
      finally { if (--pendingOperations === 0) runtime.finishRefresh(); }
      break;
    }
    case "refresh":
      pendingOperations++;
      try { await refreshFeeds(message.feeds, message.proxy,
        (url, xml, id) => runtime.receiveFeed(url, xml, false, id), (url, error) => runtime.feedError(url, error)); }
      finally { if (--pendingOperations === 0) runtime.finishRefresh(); }
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
