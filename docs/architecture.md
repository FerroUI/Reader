# Reader architecture

Reader has one portable application and two platform hosts. Domain operations do not know about controls, networking or file pickers. The view model owns library state and exposes commands and observable projections. Shared RSX views create real FerroUI controls; the browser does not maintain a second HTML implementation of the Reader interface.

## Crate boundaries

| Crate | Main modules and responsibilities |
| --- | --- |
| `reader-domain` | `model` defines versioned documents and stable IDs; `action` applies validated operations; `merge` reconciles feed entries; `feed` parses RSS/Atom/JSON Feed; `query` and `stream` select content; `folders`, `opml`, `document` and `trends` keep their respective rules separate |
| `reader-state` | `AppViewModel`, ReactiveUI command serialization, DynamicData caches, incremental counts, query updates, read selection, navigation, undo and portable host requests |
| `reader-content` | Pure, bounded HTML projection into native presentation blocks; optional HTML5/Readability article extraction shared by both hosts |
| `reader-extract` | A separate `wasm32-unknown-unknown` worker bridge; no FerroUI, rendering or network dependencies |
| `reader-controls` | Material theme inclusion, Reader colors/typography, reusable buttons and fields, named-part control templates, icons |
| `reader-ui` | `shell`, `header`, `sidebar`, `toolbar`, `article`, `rows`, `reading`, `content`, `home`, `settings`, `dialogs`, `keyboard` and `drag` each cover a narrow presentation concern |
| `reader-fonts` | Register the same named font collection in both hosts; native embedding is feature gated, while the browser loads staged files |
| `reader-browser` | Rust application lifetime/exports plus small JavaScript adapters for storage, HTTP, file exchange, navigation and the offline shell |
| `reader-desktop` | Native application lifetime, platform services, curl transport, background execution and file persistence |

The dependency direction is host → UI → controls/state → domain, with UI → content → domain for article projection. Only the desktop host and extraction worker enable `reader-content/extraction`; the browser UI does not link the DOM extraction engine. `reader-fonts` is shared by the hosts. The framework is a pinned submodule, excluded as an independent workspace member; its crates are path dependencies in the application workspace.

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

Hosts recheck a refresh's original feed ID and URL before applying its result. An unsubscribe while a network request is pending therefore cannot recreate the removed subscription. A transient library generation advances on full backup replacement and its undo; both hosts reject earlier-generation responses even when the replacement reuses the same IDs and URLs. Pending-operation counts are separated by library generation so obsolete requests cannot hold open or finish the new library's busy state. Feed parsing and a successful merge do not navigate: a host may select a newly subscribed feed only when that request is the latest subscription and the user has stayed at its starting reading location. Background refresh leaves the current destination alone.

One subscription can belong to several folder labels. The legacy `folder` field remains as the primary compatibility label; normalized `folders` carries all memberships. OPML imports merge and deduplicate subscriptions while combining memberships, and export represents a feed in each applicable folder. Removing a folder removes its labels rather than deleting unrelated subscriptions.

Search terms are ANDed. Quotes group phrases, a leading `-` excludes a term/operator, and named operators select read/star state, feed, author, title, tag or UTC publication date. Search covers locally cached content; no server index is queried. `before:YYYY-MM-DD` is exclusive and `after:YYYY-MM-DD` is inclusive at UTC midnight.

## Reading behavior

Opening an article, scanning a list cursor and scrolling are different transitions. `n`/`p` moves the cursor without opening or marking read. `j`/`k` opens an item. In expanded mode, the reading tracker finds the article crossing a fixed reading line after a user scroll and posts the state change through the dispatcher; it does not mutate the library during layout.

Explicit Keep unread is held in session state so automatic read tracking does not immediately reverse the action. Programmatic scrolling has a short suppression window. Mark-all-read computes the intended stream and age cutoff in the domain layer. Settings determine whether the UI requests confirmation.

The reading tracker requires both real user input and a changed offset from the article list's own `ScrollViewer` in the same dispatcher frame. A viewport resize, item reflow or layout notification alone is insufficient. Modal surfaces, navigation changes and programmatic cursor scrolling invalidate pending reading intent.

Touch and pen activate article content after an accepted release. A shared contact guard records maximum movement, so a pan that returns to its starting point remains cancelled. Additional contacts, capture loss, cancellation and a hold also invalidate a tap. Native scrolling and held subscription dragging use FerroUI gesture recognizers. A stationary held release opens subscription actions; movement after the hold uses the same validated drop operations as mouse dragging. Edge scrolling runs only during a live held drag, and insertion markers distinguish before/after positions.

Subscription position is stored in the feed sequence. Every folder projects that sequence; a feed with several memberships therefore keeps the same relative order across those projections. Reordering within a folder preserves all memberships. A row drop into another folder changes membership and position atomically. Undo restores the user's order/membership change while retaining unrelated background feed metadata and new subscriptions. Folder ordering and independent per-folder feed orders are not implemented.

Undo is a bounded in-memory history, generally storing the affected fields or subscriptions instead of copying the complete document. Full document replacement retains a snapshot for undo. The history does not persist across a restart and is not an event-sourced synchronization protocol.

## Retained controls, RSX and performance

Views use the FerroUI declarative `rsx!` syntax and typed builders for conditional or repeated content. Both create the same underlying controls and participate in framework binding, input routing, layout and automation.

`Rows` retains one `FerroList` and one recycling data template for the window lifetime. Its identity-aware reconciliation updates rows whose article data, selection or presentation changed, preserving the surrounding virtualized `ListBox`. Article models are shared by `Rc`; pointer equality makes unchanged-row comparisons cheap. ID indices and incremental unread counts avoid repeatedly scanning the entire library for common read/star changes.

FerroUI's `.with(...)` callback runs only when a control mounts. It must not capture a changing article identity in a recycled row. Article pointer hooks keep a weak control reference and resolve the current `ArticleRow`/`ArticleEntry` ancestor tag when input arrives. Declarative `on_click` callbacks are exchanged by reconciliation. Subscription drag hooks are protected by keys containing the feed ID and folder, and publisher text blocks by article ID and content hash. The rendered touch fixture imports replacement IDs, scrolls recycled rows, stars one, then opens that same row to exercise this lifecycle boundary.

`reading::panel::ArticlePanel` composes the public `TreeDataGridPresenterBase` behind the retained native `ListBox`. Reader keeps the ListBox's selection, generator, template, automation and input semantics, while TreeDataGrid performs realization, recycling and viewport bookkeeping. The pinned presenter declares position/size hooks which its default paths do not yet call, so Reader supplies the anchor hook, arrangement and total extent explicitly through a Fenwick index. Measured-height updates and pixel-to-row seeks are O(log n); unseen rows retain independent estimates instead of inheriting the latest window average. A half-viewport buffer reduces churn, and the idle container pool is capped at eight beyond realized/focused controls.

Geometry caches use stable article IDs, body and display geometry. Read/star/cursor changes retain measured heights and the same container; changing content, text size or meaningful width invalidates the affected measurements. Source changes preserve the visible article and its pixel offset. Up to 64 nearby body blocks also participate in native anchoring, preserving text below a late image inside a tall article. Their positions are found by binary search over the ordered body blocks. Resize restoration prefers a surviving block's screen position, with article identity/pixel offset as fallback. User gestures cancel pending restoration, and explicit keyboard seeks use the indexed row start and finite layout width. These rules avoid making selection and refresh behave like a new scroll operation.

The implementation still has work proportional to stream length when it creates a new row projection, rebuilds an index or changes a query. Feed parsing, backup serialization and a large import can consume main-thread time in the browser. Publisher HTML extraction runs in workers. Native content projection is bounded and cached by content/base URL under an 8 MiB/96-document budget. This is not a claim of constant-time sorting or arbitrary-library-size latency.

The Material base is included through FerroUI's build-time theme part selection. Reader styles override typography, density, border, gradient and selection resources. Reusable Button and ListBoxItem templates use `TemplateBinding` to keep content, alignment, brushes, borders and font properties live. Application code uses actual control semantics instead of replacing buttons with unstructured painted hit regions.

Liberation Sans supplies consistent Arial-like metrics across hosts, with bundled regular/bold/italic/bold-italic styles; Liberation Serif and Inter provide additional typography/fallback support. Bundled Liberation Mono gives article code fixed-width metrics in both hosts. Identical fonts reduce platform variation but do not make the renderer pixel-identical to the original browser DOM.

## Article content boundary

Publisher HTML is projected into selectable FerroUI text, inline formatting, links and separately fetched images. Paragraphs, headings, emphasis, nested ordered/unordered lists, blockquotes and preformatted code retain structure and whitespace. Tables preserve cells, spans and emphasis in a horizontally scrollable native grid; overly wide tables retain readable cell text. Unsafe URL schemes, scripts, styles, embedded frames, SVG and forms never execute. Audio/video/embedded-media addresses become explicit external links.

Image controls reserve publisher dimensions before loading, use bounded aspect-ratio placeholders otherwise, and notify only their own reactive leases when decoded. Native effective-viewport notifications restrict loading and bitmap retention to visible images and a half-screen margin. Leaving that region releases the control's bitmap lease while retaining known dimensions, so a tall image-heavy article does not retain every decoded bitmap. The shared image cache is bounded to 32 MiB. Publisher CSS, arbitrary HTML layout, active widgets and embedded media players are not reproduced. The browser print adapter sanitizes a separate printable document; desktop printing creates an escaped text preview.

**Read full article** creates a `FetchArticle` request carrying the article ID, monotonically increasing request ID, library generation and original URL. The browser downloads and extracts in at most two module workers with a 35-second outer deadline; cancelling terminates that worker. Desktop uses its existing four-thread blocking pool and a cancellable curl child. Both use the same pinned `dom_smoothie` HTML5/Readability implementation. Neither embeds a live publisher page nor executes publisher scripts. The result becomes an `Rc<FullArticle>` only if request identity and library generation still match. A feed refresh retains the saved copy when the article URL is unchanged; changed URLs discard it. Reentrant document replacement invalidates queued cache mutations as well as pending requests.

Each extracted article is capped at 2 MiB, publisher documents at 4 MiB, and saved full text at 24 MiB per library. Oldest fetched copies are evicted first while feed content and user annotations remain. Full text and the selected reading mode survive reload/backup; transient errors and downloads do not. Search and print use the saved body, with shared derived search strings avoiding large annotation-copy costs. [Article reading](article-reading.md) documents service selection and publisher limits.

## Browser host

`main.js` loads the canonical FerroUI runtime, prepares storage, loads shared font bytes, attaches exports and starts the Rust application. `viewport.js` tracks the host's measured width and height, visual-viewport resize/pan events and the primary-pointer media query. The host rectangle excludes CSS safe-area insets. Keyboard shrinkage therefore reaches the retained view and dialog layout; pinch magnification keeps the existing logical layout instead of reflowing it to the zoomed viewport. The native desktop host publishes both client dimensions through the same view-model API.

Search and subscription fields declare typed `TextInputOptions`. The pinned framework applies browser input hints only to its exact-edit connection, which ordinary TextBoxes do not use. Reader's browser adapter therefore observes native `GotFocus`, reads the actual focused TextBox options and updates the existing legacy DOM input's keyboard/return/capitalization hints before the platform focuses it. Moving between native fields updates those hints even if DOM focus is unchanged. The adapter does not change input type, value, selection, composition or the exact-edit textarea; browser corrections stay disabled on the legacy path.

Browser history is bridged separately from library mutations. View-model notifications and presentation-panel/menu changes schedule a small host reconciliation rather than polling the current view. An open menu, modal or narrow subscription drawer adds a transient history guard; Back dismisses the top layer before leaving the stream, retaining a guard while another layer remains. Closing the final overlay in the UI consumes that entry, and changing streams closes the layers and replaces the guard without introducing an empty Back stop. In-flight close traversal does not overwrite a newer native route. Reload/Forward does not reconstruct discarded dialog drafts. Article selection and read/star changes do not add history entries.

`input.js` captures only composition-related keyboard events before the framework's generic key callback, leaving browser default behavior and composition/beforeinput/input events intact. This prevents Enter/Escape or candidate-selection keys from executing application shortcuts while an IME is composing. The framework's persistent text-input bridge continues to own text editing. Normal browser modifier shortcuts remain governed by native keyboard routing.

The application persistence module is named **`reader-storage.js`**. FerroUI stages its own platform module named `storage.js` into the same output directory; naming the Reader adapter `storage.js` overwrote that framework module and broke runtime imports. App adapters must use distinct names when sharing the generated output namespace. Imports and the service-worker manifest use the renamed Reader module; the framework's own `storage.js` remains in the built site.

The `HostRequest` enum is serialized with an explicit operation kind. Requests cover refresh/subscribe, safe external navigation, files, share/email/copy/print, article images, focus, scroll and scheduled saves. The application contains no networking client implementation or browser object inside the domain model.

### HTTP transport

`feeds.js` validates HTTP(S) URLs, rejects embedded credentials, caps response bodies and uses an abort timer. The browser decoder considers the HTTP charset, XML declaration and byte-order mark. Discovery follows the effective response URL and a valid HTML base URL. The optional proxy is used after a direct request fails; it must return the raw document. Refresh workers limit concurrency to four and identical in-flight subscriptions share the same download promise.

Browser CORS, mixed-content rules, network permissions and publisher availability still apply. There is no poller running on a server and no cross-device historical feed store. See [the README connection instructions](../README.md#connecting-live-feeds).

A subscription completion opens its new feed only if it is the latest subscription request and the user is still at the reading location where it started. The feed is added even when navigation has moved elsewhere. External article, email/share and printed links preserve URL fragments; feed-identity normalization still strips fragments. Clipboard fallback restores the prior focus/selection and does not activate a phone keyboard. Print uses a sanitized, offscreen printable iframe instead of a `display:none` frame.

### Persistence and offline updates

`reader-storage.js` keeps an in-memory serialized snapshot and queues IndexedDB transactions. Each transaction checks the saved generation before replacing the document. A competing tab's successful write therefore produces a visible conflict instead of a silent overwrite. A localStorage fallback supports more limited environments without pretending to offer the same concurrency guarantee.

Save requests are coalesced by the host. Visibility/pagehide hooks request a flush, and errors are reported to the view model. There is no permanent `beforeunload` listener, allowing browsers to retain the page in their back/forward cache; `pageshow` refreshes time, viewport and navigation state. IndexedDB version changes and unexpected connection closure report the failure immediately and block writes until reload. Abrupt process termination or storage eviction can still lose changes that have not reached persistent storage. Portable JSON backups are the explicit export boundary.

The service-worker cache name derives deterministically from normalized build inputs and the worker template. `prepare-site.mjs` first computes that identifier, stamps the index, then generates `{ path, sha256, size }` entries from each asset's **final bytes**, including the stamped index. This sequence avoids a circular build-identifier/index-hash dependency.

Installation fetches at most two assets concurrently with `cache: "reload"` and a `?__reader_build=VERSION` query, then validates their exact decoded size and SHA-256 before storing them under canonical URLs. The trusted manifest bounds allocation for each response. A partial response, wrong size or different hash fails installation. Cancellation waits for outstanding fetches/cache writes before deleting only a newly created failed-generation cache; previously existing caches are preserved.

Only successful activation removes older Reader cache generations. There is no `skipWaiting`: existing clients retain their previous worker until the normal service-worker lifecycle permits the update. An active worker serves its cached index and matching assets together; external publisher responses are outside this application cache. Focused tests verify final-byte hashes, idempotent preparation, canonical cache keys, activation ordering and same-length tampering rollback. The final browser gate passed fresh installation and verified new-document offline reload; [verification.md](verification.md) records the precise tested revision and lifecycle-coverage limits.

## Desktop host

The native host creates the same view model and shared view in a FerroUI `Window`. A small four-thread worker pool executes blocking curl requests and file operations. A future communicates each completion to the dispatcher without moving `Rc` controls or view models across threads.

`FeedOperations` keeps a ticket for each refresh or subscription, rather than cancelling every older request when a new one starts. All successful concurrent subscriptions can merge; only the latest subscription may change navigation. The last current-generation operation clears the busy state. Two small tests exercise concurrent subscription completion and replacement-generation isolation without linking the UI test profile.

The default transport invokes the operating system's `curl` executable with HTTP(S)-only redirects, a connection timeout, a total timeout and response limits. It captures the final URL for relative feed/discovery links. The current native decoder accepts UTF-8. Browser proxy settings do not affect native requests.

A separate writer owns a replaceable pending JSON snapshot. Saving writes a temporary file in the destination directory, synchronizes it, then renames it over `reader.json`; Unix also synchronizes the parent directory. The shutdown barrier waits for outstanding writes. A read/parse failure preserves a recovery file before enabling replacement writes, or disables writes if preservation fails. Desktop processes do not merge simultaneous edits, so one process should own a given data directory.

Native pickers import/export OPML and JSON. Clipboard and launcher integrations use FerroUI's platform contracts. Share copies title/link, email prepares a draft, and print opens a local text document. These host operations are intentionally explicit in the parity documentation rather than described as Google service compatibility.

## Build and release

The checked-in manifest's framework dependency block is synchronized from the pinned FerroUI checkout by `FerroUI/scripts/apps/sync-workspace.py`. `scripts/build-browser.sh` calls the upstream builder, stages fonts and runs `prepare-site.mjs` to stamp assets, build metadata and the offline manifest.

The Pages workflow has a build job and a deploy job. It caches installed tools before compilation, restores Cargo outputs keyed by framework/toolchain/build inputs, runs domain/state tests and focused JavaScript host tests, and builds the real browser application. Only successful `main` builds deploy. It does not build every native platform or perform a screenshot matrix on every push.

Rendered-browser verification is an explicit optional gate: `[verify-ui]` in the pushed commit message or `workflow_dispatch` with `verify_ui: true` runs `node scripts/ui-smoke.mjs target/browser/reader-browser target/ui-smoke`. The harness uses the canonical FerroUI DevTools-protocol driver with a fresh Chromium profile, serves the compiled site locally and sends real pointer/keyboard/touch input to the FerroUI controls. Inspection exports locate live controls and read application state for assertions; screenshots preserve the observed render. Navigation/reload verification waits for a changed `performance.timeOrigin`, and offline verification uses bounded worker-activation checks plus explicit offline/build-identity assertions. The 17-check gate also covers native gesture cancellation, recycled row identity, held subscription actions and movement, nested overlay history, small/landscape/keyboard viewport bounds, DOM keyboard hints and scrolling-driven reading state. [verification.md](verification.md) records the observed results for the exact release.

The requested workflow uploads the `reader-ui-verification` artifact even when the smoke gate fails, as long as evidence files were created. Its screenshots, control-tree snapshots and results/failure evidence are retained for seven days. A requested gate is part of the build job, so its failure prevents deployment. Ordinary pushes retain the small test/build/deploy path. See [verification.md](verification.md) for the passing browser run, captured control counts and coverage limits; the presence of the harness is not by itself a passing result.

Visual and interaction validation remains a separate release activity. [reference.md](reference.md) defines the historical source and metrics; [parity.md](parity.md) records feature boundaries and the acceptance checklist. Passing compilation is not evidence of complete historical parity.
