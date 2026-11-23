import { FerroExports } from "./ferroui.js";
import { loadModule } from "./ferroui-loader.js";
import { attachRuntime, now, flush, flushForUpdate, reportError } from "./host.js";
import { prepareStorage } from "./reader-storage.js";
import { attachNavigation } from "./navigation.js";
import { attachViewport } from "./viewport.js";
import { attachInputGuard } from "./input.js";
import { attachUpdates } from "./updates.js";

const fontNames = ["LiberationSans-Regular.ttf", "LiberationSans-Bold.ttf", "LiberationSans-Italic.ttf", "LiberationSans-BoldItalic.ttf", "LiberationSerif-Regular.ttf", "LiberationMono-Regular.ttf"];
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
    runtime.runMain(...fonts);
    attachInputGuard(document.getElementById("out"));
    attachNavigation(runtime);
    attachViewport(runtime, document.getElementById("out"));
    const update = () => runtime.tick(now());
    setInterval(update, 30000);
    document.addEventListener("visibilitychange", () => { if (document.hidden) void flush(); else update(); });
    addEventListener("pagehide", () => void flush());
    addEventListener("pageshow", update);
    addEventListener("online", () => {
      if (!runtime.refreshInitialFeeds()) runtime.hostStatus("Back online. Refresh your subscriptions to get new articles.");
    });
    addEventListener("offline", () => runtime.hostStatus("You're offline. Your saved articles are still available."));
    attachUpdates(runtime, flushForUpdate);
  }
} catch (error) {
  document.getElementById("loading-status").textContent = `Reader could not start. ${error.message || error}`;
  document.querySelector(".loading-line")?.remove();
  console.error(error); reportError(error);
}
