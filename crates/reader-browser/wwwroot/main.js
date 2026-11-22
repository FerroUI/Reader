import { FerroExports } from "./ferroui.js";
import { loadModule } from "./ferroui-loader.js";
import { attachRuntime, width, now, flush, reportError } from "./host.js";
import { prepareStorage } from "./storage.js";
import { attachNavigation } from "./navigation.js";

const fontNames = ["LiberationSans-Regular.ttf", "LiberationSans-Bold.ttf", "LiberationSans-Italic.ttf", "LiberationSans-BoldItalic.ttf", "LiberationSerif-Regular.ttf"];
try {
  await prepareStorage();
  const [loaded, ...fonts] = await Promise.all([
    loadModule({ script: new URL("./reader-browser.js", import.meta.url).href, element: document.getElementById("out") }),
    ...fontNames.map(async name => {
      const response = await fetch(new URL(`./fonts/${name}`, import.meta.url));
      if (!response.ok) throw new Error(`Reader's font could not be loaded (${response.status}).`);
      return new Uint8Array(await response.arrayBuffer());
    }),
  ]);
  if (loaded) {
    const runtime = await loaded.createRuntime();
    FerroExports.attach(runtime);
    attachRuntime(runtime);
    globalThis.readerApp = runtime;
    runtime.runMain(location.search, ...fonts);
    attachNavigation(runtime);
    new ResizeObserver(() => runtime.resizeWindow(width())).observe(document.getElementById("out"));
    const update = () => runtime.tick(now());
    setInterval(update, 30000);
    document.addEventListener("visibilitychange", () => { if (document.hidden) void flush(); else update(); });
    addEventListener("pagehide", () => void flush());
    addEventListener("beforeunload", () => void flush());
    addEventListener("online", () => runtime.hostStatus("Back online. Refresh your subscriptions to get new articles."));
    addEventListener("offline", () => runtime.hostStatus("You're offline. Your saved articles are still available."));
    if ("serviceWorker" in navigator && !new URLSearchParams(location.search).has("nooffline")) {
      navigator.serviceWorker.register(new URL("./app-sw.js", import.meta.url), { scope: "./" })
        .catch(error => console.info("Offline installation is unavailable:", error.message));
    }
  }
} catch (error) {
  document.getElementById("loading-status").textContent = `Reader could not start. ${error.message || error}`;
  document.querySelector(".loading-line")?.remove();
  console.error(error); reportError(error);
}
