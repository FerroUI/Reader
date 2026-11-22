const routes = { home: "Home", all: "All", starred: "Starred", trends: "Trends", explore: "Explore", settings: "Settings" };
let notify = () => {};
export function navigationChanged() { notify(); }
export function parseRoute(hash) {
  const [name, ...tail] = hash.replace(/^#\/?/, "").split("/");
  if (Object.hasOwn(routes, name) && !tail.length) return routes[name];
  let value; try { value = decodeURIComponent(tail.join("/")); } catch { return null; }
  if (name === "feed" && /^\d+$/.test(value) && Number.isSafeInteger(Number(value)) && Number(value) > 0) return { Feed: Number(value) };
  const key = name === "folder" ? "Folder" : name === "tag" ? "Tag" : name === "search" ? "Search" : null;
  return key && value ? { [key]: value } : null;
}
export function routeFor(view) {
  if (typeof view === "string") return `#${view.toLowerCase()}`;
  const [key, value] = Object.entries(view)[0];
  return `#${key.toLowerCase()}/${encodeURIComponent(value)}`;
}
export function attachNavigation(runtime, browser = globalThis) {
  const { history, location, document } = browser;
  let last, overlayEntry = false, closingEntry = false, applying = false, queued = false;
  const state = overlay => ({ ...history.state, reader: { view: last, overlay } });
  const title = () => {
    const view = JSON.parse(last);
    document.title = `${typeof view === "string" ? view : "Reading"} – Reader`;
  };
  const write = (replace, overlay) => {
    history[replace ? "replaceState" : "pushState"](state(overlay), "", routeFor(JSON.parse(last)));
  };
  const sync = () => {
    queued = false;
    if (applying || closingEntry) return;
    const view = runtime.currentView();
    if (!view) return;
    if (view !== last) {
      // A new reading location replaces any modal guard instead of leaving an
      // extra, empty Back stop between the old and new locations.
      for (let layer = 0; layer < 8 && runtime.hasOverlay(); layer++) {
        if (!runtime.dismissOverlay()) break;
      }
      last = view;
      write(overlayEntry, false);
      overlayEntry = false;
      title();
    }
    const overlay = runtime.hasOverlay();
    if (overlay && !overlayEntry) {
      write(false, true);
      overlayEntry = true;
    } else if (!overlay && overlayEntry) {
      closingEntry = true;
      history.back();
    }
  };
  const schedule = () => {
    if (!queued) { queued = true; browser.queueMicrotask(sync); }
  };
  const read = () => {
    if (closingEntry) {
      // A native close consumed its own guard. A route/overlay change made
      // while traversal was pending is reconciled here, never overwritten.
      closingEntry = false;
      overlayEntry = false;
      sync();
      return;
    }
    applying = true;
    try {
      runtime.dismissOverlay();
      const view = parseRoute(location.hash);
      if (view && JSON.stringify(view) !== runtime.currentView()) runtime.navigate(JSON.stringify(view));
      last = runtime.currentView();
      overlayEntry = false;
      // A reload/Forward never restores a stale dialog with an unsaved draft.
      // Invalid hashes are replaced with the actual current reading location.
      write(true, false);
      title();
    } finally { applying = false; }
  };
  read();
  notify = schedule;
  browser.addEventListener("popstate", read);
  browser.addEventListener("hashchange", read);
  browser.addEventListener("pageshow", schedule);
  return () => {
    if (notify === schedule) notify = () => {};
    browser.removeEventListener("popstate", read);
    browser.removeEventListener("hashchange", read);
    browser.removeEventListener("pageshow", schedule);
  };
}
