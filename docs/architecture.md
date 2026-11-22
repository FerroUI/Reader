# Reader architecture

Reader has one portable application and two platform hosts. Domain operations do not know about controls, networking or file pickers. The view model owns library state and exposes commands and observable projections. Shared RSX views create real FerroUI controls; the browser does not maintain a second HTML implementation of the Reader interface.

## Crate boundaries

| Crate | Main modules and responsibilities |
| --- | --- |
| `reader-domain` | `model` defines versioned documents and stable IDs; `action` applies validated operations; `merge` reconciles feed entries; `feed` parses RSS/Atom/JSON Feed; `query` and `stream` select content; `folders`, `opml`, `document` and `trends` keep their respective rules separate |
| `reader-state` | `AppViewModel`, ReactiveUI command serialization, DynamicData caches, incremental counts, query updates, read selection, navigation, undo and portable host requests |
| `reader-controls` | Material theme inclusion, Reader colors/typography, reusable buttons and fields, named-part control templates, icons |
| `reader-ui` | `shell`, `header`, `sidebar`, `toolbar`, `article`, `rows`, `reading`, `content`, `home`, `settings`, `dialogs`, `keyboard` and `drag` each cover a narrow presentation concern |
| `reader-fonts` | Register the same named font collection in both hosts; native embedding is feature gated, while the browser loads staged files |
| `reader-browser` | Rust application lifetime/exports plus small JavaScript adapters for storage, HTTP, file exchange, navigation and the offline shell |
| `reader-desktop` | Native application lifetime, platform services, curl transport, background execution and file persistence |

The dependency direction is host → UI → controls/state → domain. `reader-fonts` is shared by the hosts. The framework is a pinned submodule, excluded as an independent workspace member; its crates are path dependencies in the application workspace.

## Commands and state changes

`AppState` is the serializable source of truth. It contains feeds, articles, folder labels, global preferences, per-stream display settings, the schema version and the next ID. Runtime lookup indices and normalized search text are derived, so backups do not contain redundant indices.

The UI calls a view-model method or `dispatch(Action)`. `ReactiveCommand` routes mutation through a serialized action queue. Reentrant commands enqueue rather than taking a second mutable document borrow. Each successful domain action returns a `Changes` set describing updated/removed articles and whether feeds or settings changed.

The view model uses that set to update its `SourceCache<Rc<Article>, Id>` and unread count index. Small annotation changes update only affected query membership; a structural change or larger batch rebuilds the visible query. A separate visible cache and ordered ID projection describe the active stream. Notification and persistence scheduling occur after mutation and cache updates.

| Concern | Ownership |
| --- | --- |
| Saved subscription/article data | `reader-domain::AppState` |
| Active view, selection, keyboard cursor, unread counts, operation status | `reader-state::AppViewModel` |
| Open dialog, draft text, focus handles, collapsed folders, drag feedback | `reader-ui::Session` |
| HTTP, clipboard, files, browser history and persistence implementation | Host adapters |

Transient presentation state uses FerroUI signals. It is deliberately distinct from saved library data; an open dialog or hover is not serialized into the backup document. Saved start-page preferences initialize the view model, with a fallback to Home when their destination no longer exists.

## Stable identities and feed merging

Feeds and articles have application-assigned IDs. Feed merges deduplicate source identities and update publisher content while retaining the user's read state, star, tags, notes and recorded reading time. Refresh errors attach to the affected subscription without deleting its cached entries.

Hosts recheck a refresh's original feed ID and URL before applying its result. An unsubscribe while a network request is pending therefore cannot recreate the removed subscription. Feed parsing and a successful merge do not navigate: the host explicitly selects the resulting feed only for a successful user subscription. Background refresh leaves the current destination alone.

One subscription can belong to several folder labels. The legacy `folder` field remains as the primary compatibility label; normalized `folders` carries all memberships. OPML imports merge and deduplicate subscriptions while combining memberships, and export represents a feed in each applicable folder. Removing a folder removes its labels rather than deleting unrelated subscriptions.

Search terms are ANDed. Quotes group phrases, a leading `-` excludes a term/operator, and named operators select read/star state, feed, author, title, tag or UTC publication date. Search covers locally cached content; no server index is queried. `before:YYYY-MM-DD` is exclusive and `after:YYYY-MM-DD` is inclusive at UTC midnight.

## Reading behavior

Opening an article, scanning a list cursor and scrolling are different transitions. `n`/`p` moves the cursor without opening or marking read. `j`/`k` opens an item. In expanded mode, the reading tracker finds the article crossing a fixed reading line after a user scroll and posts the state change through the dispatcher; it does not mutate the library during layout.

Explicit Keep unread is held in session state so automatic read tracking does not immediately reverse the action. Programmatic scrolling has a short suppression window. Mark-all-read computes the intended stream and age cutoff in the domain layer. Settings determine whether the UI requests confirmation.

Undo is a bounded in-memory history, generally storing the affected fields or subscriptions instead of copying the complete document. Full document replacement retains a snapshot for undo. The history does not persist across a restart and is not an event-sourced synchronization protocol.

## Retained controls, RSX and performance

Views use the FerroUI declarative `rsx!` syntax and typed builders for conditional or repeated content. Both create the same underlying controls and participate in framework binding, input routing, layout and automation.

`Rows` retains one `FerroList` and one recycling data template for the window lifetime. Its identity-aware reconciliation updates rows whose article data, selection or presentation changed, preserving the surrounding virtualized `ListBox`. Article models are shared by `Rc`; pointer equality makes unchanged-row comparisons cheap. ID indices and incremental unread counts avoid repeatedly scanning the entire library for common read/star changes.

The implementation still has work proportional to stream length when it creates a new row projection or changes a stream query. Parsing, backup serialization and a large import can consume main-thread time in the browser. There is no claim of constant-time sorting or arbitrary-library-size latency. The major controls are retained and network/storage I/O is asynchronous, while focused behavior tests guard mutation and reentrancy invariants.

The Material base is included through FerroUI's build-time theme part selection. Reader styles override typography, density, border, gradient and selection resources. Reusable Button and ListBoxItem templates use `TemplateBinding` to keep content, alignment, brushes, borders and font properties live. Application code uses actual control semantics instead of replacing buttons with unstructured painted hit regions.

Liberation Sans supplies consistent Arial-like metrics across hosts, with bundled regular/bold/italic/bold-italic styles; Liberation Serif and Inter provide additional typography/fallback support. Identical fonts reduce platform variation but do not make the renderer pixel-identical to the original browser DOM.

## Article content boundary

Publisher HTML is projected into selectable FerroUI text, inline formatting, links and separately fetched images. Paragraphs, headings, emphasis, ordered/unordered list items, blockquotes and code receive corresponding presentation; preformatted code retains whitespace and table cell text is preserved in a simplified projection. Unsafe URL schemes, scripts, styles, embedded frames, SVG and form content are excluded from this projection.

This is a constrained HTML renderer. Publisher CSS, arbitrary table geometry/layout, active widgets, audio/video players and every HTML nesting rule are not reproduced. The original article link is the route to the publisher's full page. The browser's print adapter sanitizes a separate printable document; desktop printing creates an escaped text preview.

## Browser host

`main.js` loads the canonical FerroUI runtime, prepares storage, loads shared font bytes, attaches exports and starts the Rust application. A resize observer updates the view-model width. Browser history is bridged separately from library mutations, and file import/export, sharing and printing remain small host modules.

The application persistence module is named **`reader-storage.js`**. FerroUI stages its own platform module named `storage.js` into the same output directory; naming the Reader adapter `storage.js` overwrote that framework module and broke runtime imports. App adapters must use distinct names when sharing the generated output namespace. Imports and the service-worker manifest use the renamed Reader module; the framework's own `storage.js` remains in the built site.

The `HostRequest` enum is serialized with an explicit operation kind. Requests cover refresh/subscribe, safe external navigation, files, share/email/copy/print, article images, focus, scroll and scheduled saves. The application contains no networking client implementation or browser object inside the domain model.

### HTTP transport

`feeds.js` validates HTTP(S) URLs, rejects embedded credentials, caps response bodies and uses an abort timer. The browser decoder considers the HTTP charset, XML declaration and byte-order mark. Discovery follows the effective response URL and a valid HTML base URL. The optional proxy is used after a direct request fails; it must return the raw document. Refresh workers limit concurrency to four and identical in-flight subscriptions share the same download promise.

Browser CORS, mixed-content rules, network permissions and publisher availability still apply. There is no poller running on a server and no cross-device historical feed store. See [the README connection instructions](../README.md#connecting-live-feeds).

### Persistence and offline updates

`reader-storage.js` keeps an in-memory serialized snapshot and queues IndexedDB transactions. Each transaction checks the saved generation before replacing the document. A competing tab's successful write therefore produces a visible conflict instead of a silent overwrite. A localStorage fallback supports more limited environments without pretending to offer the same concurrency guarantee.

Save requests are coalesced by the host. Visibility/page lifecycle hooks request a flush, and errors are reported to the view model. Abrupt process termination or storage eviction can still lose changes that have not reached persistent storage. Portable JSON backups are the explicit export boundary.

The service-worker cache name derives from generated asset content. Installation populates the complete snapshot or fails; an active worker serves its matching index and assets together. New workers wait for old clients to close, avoiding mixed old/new runtime modules. External publisher responses are outside this application cache.

## Desktop host

The native host creates the same view model and shared view in a FerroUI `Window`. A small four-thread worker pool executes blocking curl requests and file operations. A future communicates each completion to the dispatcher without moving `Rc` controls or view models across threads.

The default transport invokes the operating system's `curl` executable with HTTP(S)-only redirects, a connection timeout, a total timeout and response limits. It captures the final URL for relative feed/discovery links. The current native decoder accepts UTF-8. Browser proxy settings do not affect native requests.

A separate writer owns a replaceable pending JSON snapshot. Saving writes a temporary file in the destination directory, synchronizes it, then renames it over `reader.json`; Unix also synchronizes the parent directory. The shutdown barrier waits for outstanding writes. A read/parse failure preserves a recovery file before enabling replacement writes, or disables writes if preservation fails. Desktop processes do not merge simultaneous edits, so one process should own a given data directory.

Native pickers import/export OPML and JSON. Clipboard and launcher integrations use FerroUI's platform contracts. Share copies title/link, email prepares a draft, and print opens a local text document. These host operations are intentionally explicit in the parity documentation rather than described as Google service compatibility.

## Build and release

The checked-in manifest's framework dependency block is synchronized from the pinned FerroUI checkout by `FerroUI/scripts/apps/sync-workspace.py`. `scripts/build-browser.sh` calls the upstream builder, stages fonts and runs `prepare-site.mjs` to stamp assets, build metadata and the offline manifest.

The Pages workflow has a build job and a deploy job. It caches installed tools before compilation, restores Cargo outputs keyed by framework/toolchain/build inputs, runs domain/state tests and focused JavaScript host tests, and builds the real browser application. Only successful `main` builds deploy. It does not build every native platform or perform a screenshot matrix on every push.

Rendered-browser verification is an explicit optional gate: `[verify-ui]` in the pushed commit message or `workflow_dispatch` with `verify_ui: true` runs `node scripts/ui-smoke.mjs target/browser/reader-browser target/ui-smoke`. The harness uses the canonical FerroUI DevTools-protocol driver with a fresh Chromium profile, serves the compiled site locally and sends real pointer/keyboard input to the FerroUI controls. Inspection exports locate live controls and read application state for assertions; screenshots preserve the observed render.

The requested workflow uploads the `reader-ui-verification` artifact even when the smoke gate fails, as long as evidence files were created. Its screenshots, control-tree snapshots and results/failure evidence are retained for seven days. A requested gate is part of the build job, so its failure prevents deployment. Ordinary pushes retain the small test/build/deploy path. See [verification.md](verification.md) for the passing browser run, captured control counts and remaining unrun checks; the presence of the harness is not by itself a passing result.

Visual and interaction validation remains a separate release activity. [reference.md](reference.md) defines the historical source and metrics; [parity.md](parity.md) records feature boundaries and the acceptance checklist. Passing compilation is not evidence of complete historical parity.
