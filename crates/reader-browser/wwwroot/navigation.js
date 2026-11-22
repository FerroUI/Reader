const routes = { home: "Home", all: "All", starred: "Starred", trends: "Trends", explore: "Explore", settings: "Settings" };
export function parseRoute(hash) {
  const [name, ...tail] = hash.replace(/^#\/?/, "").split("/");
  if (routes[name]) return routes[name];
  let value; try { value = decodeURIComponent(tail.join("/")); } catch { return null; }
  if (name === "feed" && /^\d+$/.test(value) && Number.isSafeInteger(Number(value)) && Number(value) > 0) return { Feed: Number(value) };
  const key = { folder: "Folder", tag: "Tag", search: "Search" }[name];
  return key && value ? { [key]: value } : null;
}
export function routeFor(view) {
  if (typeof view === "string") return `#${view.toLowerCase()}`;
  const [key, value] = Object.entries(view)[0];
  return `#${key.toLowerCase()}/${encodeURIComponent(value)}`;
}
export function attachNavigation(runtime) {
  let last;
  const read = () => {
    const view = parseRoute(location.hash);
    if (view) { const serialized = JSON.stringify(view); if (serialized !== last) runtime.navigate(serialized); last = serialized; }
  };
  read();
  last = runtime.currentView();
  if (!location.hash) history.replaceState(null, "", routeFor(JSON.parse(last)));
  addEventListener("popstate", read);
  addEventListener("hashchange", read);
  // Observe only the route; article, scroll and cache updates never enter browser history.
  setInterval(() => {
    const view = runtime.currentView();
    if (view && view !== last) {
      last = view; history.pushState(null, "", routeFor(JSON.parse(view)));
      document.title = `${typeof JSON.parse(view) === "string" ? JSON.parse(view) : "Reading"} – Reader`;
    }
  }, 200);
}
