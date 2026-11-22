import { createHash } from "node:crypto";
import { readdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const workerTemplate = new URL("../crates/reader-browser/wwwroot/app-sw.js", import.meta.url);
const excluded = new Set([".nojekyll", "app-sw.js", "build.json", "qa", "test-results"]);
const versionMarker = "__BUILD_VERSION__";
const precacheMarker = '"__PRECACHE_FILES__"';

async function siteFiles(root, directory = root) {
  const entries = await readdir(directory, { withFileTypes: true });
  const nested = await Promise.all(entries.map(async entry => {
    if (excluded.has(entry.name) || entry.name.endsWith(".map")) return [];
    const filename = path.join(directory, entry.name);
    if (entry.isDirectory()) return siteFiles(root, filename);
    if (!entry.isFile()) throw new Error(`Unsupported site entry: ${filename}`);
    return [path.relative(root, filename)];
  }));
  return nested.flat().sort();
}

function stampIndex(source, version) {
  let index = source.replaceAll(versionMarker, version);
  if (/data-reader-build="[^"]*"/.test(index)) {
    index = index.replace(/data-reader-build="[^"]*"/, `data-reader-build="${version}"`);
  } else {
    index = index.replace(/<html\b/, `<html data-reader-build="${version}"`);
  }
  // Version direct entry points while keeping every resource relative to /Reader/.
  return index.replace(/\b(src|href)=(['"])([^'"]+)\2/g, (match, attribute, quote, value) => {
    if (/^(?:[a-z][a-z\d+.-]*:|\/\/|\/|#)/i.test(value)) return match;
    const url = new URL(value, "https://reader.invalid/");
    if (!/\.(?:m?js|css)$/.test(url.pathname)) return match;
    url.searchParams.set("v", version);
    const relative = value.split(/[?#]/, 1)[0];
    return `${attribute}=${quote}${relative}${url.search}${url.hash}${quote}`;
  });
}

export async function prepareSite(root, template) {
  const names = await siteFiles(root);
  const contents = new Map(await Promise.all(names.map(async name => [name, await readFile(path.join(root, name))])));
  for (const name of ["index.html", "main.js", "reader-storage.js", "reader-browser.js", "reader_browser.wasm", "ferroui.js", "ferroui-loader.js"]) {
    if (!contents.has(name)) throw new Error(`The built site is missing ${name}.`);
  }
  const worker = template ?? await readFile(workerTemplate, "utf8");
  if (!worker.includes(versionMarker) || !worker.includes(precacheMarker)) {
    throw new Error("app-sw.js must contain __BUILD_VERSION__ and a quoted __PRECACHE_FILES__ placeholder.");
  }
  let index = contents.get("index.html").toString();
  const previousVersion = /data-reader-build="([a-f\d]{16})"/.exec(index)?.[1];
  if (previousVersion) index = index.replaceAll(previousVersion, versionMarker);
  // Ignore only this script's query stamp when preparing an already-staged site.
  index = index.replace(/([?&])v=__BUILD_VERSION__(&?)/g, (_, start, end) => end ? start : "");
  index = index.replace(/ data-reader-build="__BUILD_VERSION__"/, "");
  contents.set("index.html", Buffer.from(index));
  const digest = createHash("sha256").update(worker);
  for (const [name, bytes] of contents) digest.update(`\0${name}\0${bytes.length}\0`).update(bytes);
  const version = digest.digest("hex").slice(0, 16);
  const assets = names.map(name => `./${name.split(path.sep).map(encodeURIComponent).join("/")}`);
  // The version is derived from normalized inputs; hashes describe the final stamped bytes.
  // Hashing before stamping the index would install an unverifiable, circular build manifest.
  const stamped = stampIndex(index, version);
  contents.set("index.html", Buffer.from(stamped));
  const manifest = names.map((name, index) => {
    const bytes = contents.get(name);
    return { path: assets[index], sha256: createHash("sha256").update(bytes).digest("hex"), size: bytes.length };
  });
  await writeFile(path.join(root, "index.html"), stamped);
  await writeFile(path.join(root, "app-sw.js"), worker
    .replaceAll(versionMarker, version)
    .replace(precacheMarker, JSON.stringify(manifest)));
  await writeFile(path.join(root, "build.json"), `${JSON.stringify({ version, assets: assets.length })}\n`);
  await writeFile(path.join(root, ".nojekyll"), "");
  return { version, assets, manifest };
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const result = await prepareSite(path.resolve(process.argv[2] ?? "target/browser/reader-browser"));
  console.log(`Prepared ${result.assets.length} offline resources (${result.version}).`);
}
