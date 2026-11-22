# FerroUI Reader

A feed reader in Rust that recreates Google Reader's final desktop interface with **FerroUI**. The browser and desktop hosts run the same declarative `rsx!` views, Material-based control themes, ReactiveUI view model and DynamicData article projections.

**Live application:** [ferroui.com/Reader](https://ferroui.com/Reader/) · [Sample library](https://ferroui.com/Reader/?demo=1) · [Pages alias](https://ferroui.github.io/Reader/) · [Build and publish](https://github.com/FerroUI/Reader/actions/workflows/pages.yml)

The workflow publishes successful builds from `main`; the deployment job is the source of truth for the currently published revision. This repository implements the core reading and subscription workflows. It does **not establish 100% visual or feature parity** with the retired Google service. [Fidelity and feature coverage](docs/parity.md) distinguishes the implemented experience, adaptations and remaining gaps. [Reference research](docs/reference.md) records preserved original CSS, JavaScript, screenshots and official feature announcements.

![Expanded reading in the compiled FerroUI application](docs/screenshots/expanded.png)

Actual browser verification captures: [Home](docs/screenshots/home.png) · [Expanded reading](docs/screenshots/expanded.png) · [Narrow layout](docs/screenshots/narrow.png). See the [verification record](docs/verification.md) for the tested revision, interaction checks and measured control counts.

## What you can do

- Read All items, individual subscriptions, folders, starred items and tags, in expanded or list view. Open a row in place, move between articles, keep an item unread, star it, edit tags and add a private note.
- Subscribe to RSS, Atom and JSON Feed addresses, or discover a feed from an accessible website. Refresh subscriptions, rename them, create/rename folders, assign several folders, drag subscriptions between folders or reorder them, and unsubscribe.
- Switch between all and unread items, sort newest or oldest first, and retain each stream's reading preferences. Mark a stream read, including items older than one day, one week or two weeks.
- Search saved article text, titles, authors, feed names, tags and notes. Query operators include `is:unread`, `is:starred`, `feed:`, `author:`, `title:`, `tag:`, `before:` and `after:`; quoted phrases and excluded terms are supported.
- Import and export OPML subscriptions with folders. Export or restore a complete JSON library containing cached articles, read state, stars, tags, notes and preferences. Undo recent library changes.
- See local reading totals, a 30-day activity chart and per-subscription reading percentages. Browse a small explicit starter catalog in Explore.
- Choose Comfortable, Cozy or Compact density, article text size and a start page. Use keyboard navigation, subscription/tag jump dialogs, a collapsible subscription pane and a reading focus mode. At narrow widths the sidebar becomes an overlay.
- Open original articles, copy their links, invoke the browser's sharing interface where supported, open an email draft and print an article.
- Keep a local library between launches. The browser caches the application for offline use after a complete successful installation; already saved article text remains readable offline.

A first launch includes a clearly identified starter library. Its articles are examples, not newly downloaded publisher content. Add your own subscriptions or import OPML to read live feeds. `?demo=1` in the browser and `--demo` on desktop create a separate in-memory sample session that does not save over the normal library.

## Build and run the browser app

Install Git, Rust through rustup, Node.js 24 with npm, Python 3 and a native C/C++ build toolchain. The canonical [FerroUI browser setup](FerroUI/scripts/browser/setup.sh) supports Linux and macOS and installs its pinned browser tools into the directory you select.

```bash
git clone --recurse-submodules https://github.com/FerroUI/Reader.git
cd Reader
bash FerroUI/scripts/browser/setup.sh "$PWD/.tools"
source .tools/env.sh
bash scripts/build-browser.sh
python3 -m http.server 8080 --directory target/browser/reader-browser
```

Open [localhost:8080](http://localhost:8080/) or [localhost:8080/?demo=1](http://localhost:8080/?demo=1). Serve the output through HTTP; opening `index.html` as a local file does not provide the browser capabilities the app needs.

The target is **`wasm32-unknown-emscripten`**. The pinned framework uses Rust **1.99.0**, Emscripten **6.0.10** and wasm-bindgen **0.2.129**. The setup script and checked-in toolchain select matching versions. The build wrapper delegates compilation and web runtime generation to FerroUI, copies the shared fonts, then generates a content-derived build identifier, offline manifest and `.nojekyll`.

Useful options pass through to FerroUI's builder:

```bash
bash scripts/build-browser.sh --debug
bash scripts/build-browser.sh --out target/site
```

The default deployment uses the normal single-threaded browser profile. FerroUI's optional threaded builds have additional toolchain and cross-origin-isolation requirements; GitHub Pages is not configured to provide those headers.

### Connecting live feeds

Use **Subscribe**, enter a feed or website URL, and press the dialog's subscribe action. Direct browser requests work when the publisher allows cross-origin access. A static GitHub Pages application cannot bypass a publisher's CORS policy or act as a feed crawler.

For a feed that blocks direct access, open **Reader settings → Feed connection** and enter a proxy endpoint you control or trust, with `{url}` where the encoded source address belongs:

```text
https://your-service.example/feed?url={url}
```

The endpoint must return the raw RSS, Atom, JSON Feed or HTML response, permit requests from your deployed Reader origin, and support HTTPS when Reader is served over HTTPS. Reader tries the source directly and then the configured proxy. The proxy operator can see the requested feed URLs and responses. There is no bundled public proxy or application-owned backend. The desktop host fetches directly and does not use this browser-only setting.

Feeds are refreshed on request, with at most four concurrent browser downloads. Requests have time and size limits; a failed fetch reports an error and preserves the existing library. Publisher images can also be restricted by CORS or removed from the source; the original article link remains available.

### Storage and offline operation

The browser stores the library in the `ferroui-reader` IndexedDB database. Writes use a transaction and a generation check to detect another tab's changes. When IndexedDB is unavailable, a localStorage fallback is used; that fallback does not provide the same cross-tab transaction guarantees. A conflicting tab asks you to export its backup and reload instead of replacing another tab's saved state.

Malformed saved data is retained in a recovery copy when storage allows it. If that copy cannot be written, normal writes are blocked to protect the original. Storage errors are shown in the application. **Import/Export → Download backup** produces a portable complete library; OPML only carries subscriptions and folders. Restoring a JSON backup replaces the current library after validation, while importing OPML merges subscriptions.

The service worker verifies the size and SHA-256 of every application asset against the build manifest before completing installation, including the stamped index, WASM and fonts. Downloads bypass stale HTTP-cache entries and a failed new installation preserves the prior cache. These cache-integrity paths have focused tests; the strengthened browser reload/offline gate is tracked separately in [verification.md](docs/verification.md). The worker does not cache every publisher image or fetch new articles while offline. Close all Reader tabs and reopen to activate a waiting application update. For development, `?nooffline=1` prevents registering a worker; if an older worker already controls the origin, unregister it through your browser's application tools before checking changed assets.

## Run the desktop host

The native host uses FerroUI's platform detection, Skia renderer, system clipboard, storage picker and launcher integrations. Install the corresponding [FerroUI platform prerequisites](FerroUI/README.md#getting-started), including the native graphics/text dependencies. On Linux this host selects X11 and needs a reachable X server or XWayland plus the platform libraries described in [FerroUI's X11 guide](FerroUI/docs/porting/x11-platform.md). macOS and Windows use their respective FerroUI backends. Platform dialogs and launcher availability depend on those native integrations.

Install `curl` on `PATH` for feed and image retrieval. Network work runs on a bounded background worker pool; parsing results return through the UI dispatcher.

```bash
cargo run --locked -p reader-desktop --profile fast-release
cargo run --locked -p reader-desktop --profile fast-release -- --demo
```

The library is stored as `reader.json` under the platform application-data directory:

| Platform | Default directory |
| --- | --- |
| Linux | `$XDG_DATA_HOME/FerroUI/Reader`, or `~/.local/share/FerroUI/Reader` |
| macOS | `~/Library/Application Support/FerroUI/Reader` |
| Windows | `%LOCALAPPDATA%\FerroUI\Reader` |

Set `READER_DATA_DIR` to use a different directory. Saves are coalesced on a dedicated worker and replace the file only after a complete temporary file is written and synchronized. Closing waits for pending writes. A save failure keeps the window open once and offers a chance to export a backup before exiting. Use one native process per data directory; there is no native cross-process merge protocol.

Desktop sharing copies an article title and link for pasting into another application. Email opens a `mailto:` draft. Printing opens a local text preview in the default browser. These are local platform integrations, not the former Google sharing or email service. Desktop feed decoding currently requires UTF-8; the browser host also honors supported XML/HTTP text encodings.

## Keyboard essentials

Shortcuts apply outside text inputs. Press `?` for the in-app reference.

| Keys | Action |
| --- | --- |
| `j` / `k` | Open next / previous article |
| `n` / `p` | Move the article cursor without opening |
| `o` / `Enter` | Expand or collapse the selected row |
| `Space` / `Shift+Space` | Page down / page up |
| `s`, `m`, `t` | Toggle star, toggle read state, edit tags |
| `v`, `e` | Open original article, prepare email |
| `Shift+s` / `Shift+t` | Share / send through the supported host integration |
| `Shift+a`, `r` | Mark stream read, refresh subscriptions |
| `1` / `2` | Expanded / list view |
| `g` then `h`, `a`, `s` | Home, All items, Starred |
| `g` then `t` / `u` | Choose a tag or folder / choose a subscription |
| `g` then `Shift+t` | Trends |
| `g` then `d` / `e` | Explore subscriptions |
| `+` / `-` | Increase / decrease article text size |
| `/`, `a` | Focus search, add a subscription |
| `u`, `f` | Toggle subscriptions, toggle reading focus |
| `Escape` | Close the dialog or selected entry |
| `Ctrl+z` / `Cmd+z` | Undo a recent library change |

Some original shortcuts and social actions are not reproduced. Subscription traversal opens feeds directly; it does not reproduce the complete original keyboard cursor and folder-toggle behavior of the navigation tree. The complete historical map in [reference.md](docs/reference.md) is a fidelity target, not a claim that every original key is implemented.

## Architecture and checks

| Crate | Responsibility |
| --- | --- |
| `reader-domain` | Serializable models, validated actions, feed parsing, search, OPML and backup documents |
| `reader-state` | ReactiveUI commands, DynamicData caches, counts, navigation, stream preferences and undo |
| `reader-controls` | Trimmed Material base, Reader styles, reusable templates and vector icons |
| `reader-ui` | Shared RSX shell, virtualized article rows, menus, dialogs, keyboard and drag interactions |
| `reader-fonts` | Shared font registration and licensed Liberation font assets |
| `reader-browser` | Browser lifetime, WASM exports, network/storage bridge, navigation, files and offline shell |
| `reader-desktop` | Native lifetime, background transport, atomic file storage and platform services |

Read [architecture.md](docs/architecture.md) for data flow, retained controls, performance decisions and host boundaries.

```bash
cargo test --locked -p reader-domain -p reader-state
node --test scripts/*.test.mjs
cargo check --locked -p reader-desktop
```

The tests cover domain and view-model invariants and focused browser-host behavior. They do not certify every original screen, pixel, browser or native operating system. The ordinary Pages workflow runs the first two commands and builds one browser target, without a platform matrix or native packaging gate. [Verification status](docs/verification.md) records the final eleven-check browser pass, inspected visual corrections, measured control counts and verified new-document offline reload with 10,000 saved articles.

### Optional rendered-browser verification

After building the site, install Chrome or Chromium and run:

```bash
node scripts/ui-smoke.mjs target/browser/reader-browser target/ui-smoke
```

The harness serves the built application locally and launches a **fresh isolated Chromium profile**. It does not reuse your signed-in browser, feed library or service-worker cache. Set `CHROME` to the browser executable if automatic discovery cannot find it. Node.js 24 is sufficient; no Playwright or browser-testing package is required.

The requested checks exercise first paint, actual control input, reading/annotation shortcuts, search, a local feed subscription, settings, reload persistence, conflicting storage writers, narrow navigation, a 10,000-article virtualization fixture and offline reload. PNG screenshots, control-tree JSON and a success report or failure evidence are written to `target/ui-smoke`. A passing scripted run is evidence for those exercised paths, not complete historical visual parity.

To request this gate in GitHub Actions, include **`[verify-ui]`** in the pushed commit message or manually dispatch **Build and publish** with **`verify_ui: true`**. The workflow runs the browser script after compilation and uploads **`reader-ui-verification`** with a seven-day retention period, including available failure evidence. A failed requested gate blocks that run's deployment. Ordinary `main` pushes keep the domain/state + JavaScript checks, build and deploy path.

## Framework pin and deployment

`FerroUI/` is a Git submodule pinned to [`0f5bbb50dd40522f911826efe0bc3df6e968b884`](https://github.com/wieslawsoltes/FerroUI/commit/0f5bbb50dd40522f911826efe0bc3df6e968b884). Framework dependencies resolve to paths in this checkout. Recover a missing checkout with:

```bash
git submodule update --init --recursive
```

An intentional framework upgrade should update the gitlink, generated workspace dependency block and lockfile together:

```bash
git submodule update --remote FerroUI
python3 FerroUI/scripts/apps/sync-workspace.py
cargo update --workspace
bash scripts/build-browser.sh
```

The [Pages workflow](.github/workflows/pages.yml) runs on application pushes to `main` and manual dispatch; documentation-only pushes skip the build. It caches the Emscripten SDK, wasm-bindgen tool and Cargo compilation outputs, keyed by the framework revision, toolchain and build inputs. After the small behavior checks and actual FerroUI browser build, it runs rendered-browser verification only when requested, uploads `target/browser/reader-browser` and deploys through GitHub's Pages actions. Set repository Pages publishing to **GitHub Actions** when using a fork. GitHub Pages hosts only static assets; it does not provide account synchronization or feed transport services.

## Fidelity and licensing

The interface targets the final 2011–2013 desktop design, using measurements from the [preserved original client](https://readerisdead.com/reader/view/). Fonts, rendering, several dialogs, Home/Explore, chart detail, HTML projection and narrow layouts differ. Google's account service, historical feed archive, global search, proprietary ranking, recommendations, translation, subscriber statistics and Google+ integration are unavailable here. [parity.md](docs/parity.md) is explicit about these boundaries.

Application code is [MIT licensed](LICENSE). FerroUI retains its own license and third-party notices. Bundled Liberation fonts are distributed under the [SIL Open Font License](crates/reader-fonts/assets/fonts/OFL.txt). This is an independent recreation; Google Reader and Google branding belong to their respective owners.
