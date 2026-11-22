// App storage is distinct from FerroUI's storage.js platform module.
// IndexedDB writes are atomic. A generation check prevents two tabs overwriting each other.
const KEY = "library", FALLBACK = "ferroui.reader.v1", RECOVERY = "ferroui.reader.recovery";
let db, contents = "", generation = 0, queue = Promise.resolve(), failure = null;
let report = () => {};
export function setStorageReporter(callback) { report = callback; }
const request = value => new Promise((resolve, reject) => {
  value.onsuccess = () => resolve(value.result);
  value.onerror = () => reject(value.error || new Error("Reader storage failed"));
});
export async function prepareStorage() {
  try {
    const opening = indexedDB.open("ferroui-reader", 1);
    opening.onupgradeneeded = () => opening.result.createObjectStore("state");
    db = await request(opening);
    db.onversionchange = () => { db.close(); failure = "Storage changed in another tab. Reload Reader."; };
    const stored = await request(db.transaction("state").objectStore("state").get(KEY));
    contents = stored?.contents || "";
    generation = stored?.generation || 0;
    if (!contents) contents = localStorage.getItem(FALLBACK) || "";
  } catch (error) {
    db = null;
    try { contents = localStorage.getItem(FALLBACK) || ""; }
    catch { throw new Error(`Browser storage is unavailable: ${error.message}`); }
  }
}
export function load() { return contents; }
export function save(value) {
  if (failure) throw new Error(failure);
  contents = value;
  if (!db) { localStorage.setItem(FALLBACK, value); return; }
  queue = queue.then(() => new Promise((resolve, reject) => {
    const tx = db.transaction("state", "readwrite"), store = tx.objectStore("state");
    const read = store.get(KEY);
    read.onsuccess = () => {
      if ((read.result?.generation || 0) !== generation) {
        failure = "Your library changed in another tab. Export this tab's backup, then reload before continuing.";
        tx.abort(); return;
      }
      store.put({ contents: value, generation: generation + 1 }, KEY);
    };
    tx.oncomplete = () => { generation++; resolve(); };
    tx.onerror = tx.onabort = () => reject(new Error(failure || tx.error?.message || "The library could not be saved"));
  })).catch(error => { preserveRecovery(value); report(error); });
}
export function preserveRecovery(value) {
  try { localStorage.setItem(RECOVERY, value); return true; }
  catch {
    failure ||= "The original saved library is protected because a recovery copy could not be written. Export a backup before reloading.";
    return false;
  }
}
export function flushStorage() { return queue; }
