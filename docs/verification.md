# Verification record

## Current feed-connection release — 22 November 2026

The browser feed fix is deployed on [GitHub Pages](https://ferroui.com/Reader/#home). The final run passed **24 rendered-browser scenarios**, including a publisher that deliberately rejects cross-origin reads, and independent live Chrome checks successfully added **BBC News** and **Hacker News** through the native Subscribe control. The connection behavior and its limits are documented in [feed-connection.md](feed-connection.md).

| Evidence | Observed result |
| --- | --- |
| Workflow | [45065501775](https://github.com/FerroUI/Reader/actions/runs/45065501775): checks, browser build and Pages deployment succeeded |
| Deployed revision | [`eba78c5bcd25dc87ce1f5e5aafa32197199eb3ee`](https://github.com/FerroUI/Reader/commit/eba78c5bcd25dc87ce1f5e5aafa32197199eb3ee); feed implementation introduced by [`49f06fc`](https://github.com/FerroUI/Reader/commit/49f06fcd48e41cce3124a7c5161fcd5d8f366981) |
| Build identifier | `3e729d7df7656e42`, also read from the deployed page's DOM |
| Rust checks | **34 domain/state**, **5 tree**, **4 motion** and **2 native-operation** tests passed in CI; the native desktop host also compiled locally |
| JavaScript checks | **41 passed**, including public-service normalization, routing/privacy guards, terminal errors, download limits, string parsing errors, rate-limit cooldowns and scoped Chromium diagnostic matching |
| Rendered checks | **24 passed**; exact scenario names and durations preserved in [feed-verification-results.json](feed-verification-results.json) |
| Cross-origin regression | **3,110ms** for discovery, subscription, refresh, missing-feed and malformed-feed cases |
| Offline and virtualization | New-document offline reload passed with **31 verified resources** and **10,000 saved articles**; the stress snapshot retained **28 article containers / 939 native visuals** |
| Release metadata | [feed-release.json](evidence/feed-release.json) records the revision, build, workflow, counts and live observations |

### What the cross-origin fixture proves

The isolated test runs Reader, a publisher and a raw-response relay on separate origins. The publisher deliberately omits `Access-Control-Allow-Origin`. Chromium actually blocks its direct requests, and Automatic mode uses the saved custom relay, exercising migration precedence without depending on a public service during CI.

The native Subscribe control accepts an HTML page with a leading comment, a relative `<base>` and a relative Atom alternate. Rust parses the resulting article, the subscription retains the publisher feed URL, and relative article links resolve against that publisher. Refresh retains the feed and existing article identities and adds exactly one new article. HTTP 404 and HTTP 200 with malformed XML both report meaningful native status messages, preserve the current feed route and create no empty subscription. A cookie seeded in the isolated profile reaches neither publisher nor relay; authorization headers are also absent.

| Preserved evidence | Content |
| --- | --- |
| [Subscription screenshot](screenshots/12-cross-origin-feed.png) / [control tree](evidence/12-cross-origin-feed.json) | Article rendered after enforced CORS failure and relay discovery |
| [Missing feed screenshot](screenshots/12-feed-error.png) / [control tree](evidence/12-feed-error.json) | Readable HTTP 404 without navigation or subscription mutation |
| [Malformed feed screenshot](screenshots/12-feed-parse-error.png) / [control tree](evidence/12-feed-parse-error.json) | Rust XML error retained across the JavaScript boundary |
| [Network evidence](evidence/12-feed-network.json) | Exact expected CORS diagnostics, publisher/relay requests, credential omission and refresh result |

The initial [45064732347](https://github.com/FerroUI/Reader/actions/runs/45064732347) gate rendered the cross-origin article but failed because its diagnostic filter excluded Chromium's security-source CORS entry. The final matcher accepts only the exact expected CORS sentence with matching publisher URL and caller origin. Four focused tests cover this diagnostic shape and rejection of unrelated errors; console errors and exceptions remain fatal. The failed run did not deploy.

### Independent checks on the published application

Chrome initially loaded the preceding offline build. Closing the old Reader client and opening a new tab activated `3e729d7df7656e42` normally, without clearing the library or forcing service-worker activation.

Using the visible Subscribe field, `https://feeds.bbci.co.uk/news/rss.xml` produced **BBC News with 97 unread items** at the time of observation. Entering the bare address `news.ycombinator.com/rss` produced a new **Hacker News subscription with 10 unread items**. Refresh completed on that same feed route and retained the 10-item count. Reader settings displayed the Automatic disclosure naming rss2json and the Direct only, Custom proxy and Save feed connection controls. These counts describe that observation; they are not provider item-count guarantees.

The live checks establish successful public subscriptions on the deployed build. The deterministic CI fixture separately establishes actual CORS rejection and relay behavior. Public-provider availability, quotas, cached responses and returned item counts remain external dependencies. JSON Feed requires direct access or a custom raw proxy; blocked website HTML may require the actual RSS/Atom URL. See the [connection guide](feed-connection.md) for private-feed handling and these boundaries.

The previous UI review below is retained as historical evidence for its explicitly identified revision. Its screenshots and per-scenario timings were not relabeled as captures from this feed release. Existing touch, responsive-editor, browser-history and offline scenarios ran again in the current 24-check gate. Physical iOS testing and complete historical Google service/pixel parity remain unverified.

## Previous UI/UX review — revision 05a865b

The final review release passed **all 23 rendered-browser checks**, including source-feed Home navigation, article tag links, empty-stream recovery, anchored Subscribe editing across a narrow resize, the subscription-tree keyboard cursor, responsive shortcut help and dated mark-all confirmation. The existing touch, browser-history, persistence and new-document offline checks also passed with **10,000 saved articles**. Build and deployment succeeded, and the deployed build was independently observed in Chrome. These results verify the recorded implementation and scenarios; they do not establish 100% historical visual or service parity. See [parity.md](parity.md) for boundaries and [reference.md](reference.md) for the original sources.

### Reviewed release — 22 November 2026

| Evidence | Observed result |
| --- | --- |
| Workflow | [45056258119](https://github.com/FerroUI/Reader/actions/runs/45056258119): checks, build and deploy succeeded |
| Application revision | [`05a865b30c87cc9ba04050294d6de313badc64cd`](https://github.com/FerroUI/Reader/commit/05a865b30c87cc9ba04050294d6de313badc64cd) |
| Application build identifier | `23a85d01c9f9e4a8` |
| Rendered checks | **23 passed**; exact output preserved in [verification-results.json](verification-results.json) |
| Offline manifest | **26 resources**, verified by decoded size and SHA-256 during installation |
| Independent live check | Chrome loaded [ferroui.com/Reader/#home](https://ferroui.com/Reader/#home); its DOM build identifier matched `23a85d01c9f9e4a8`, with native Home controls and the updated date/snippet layout visible |
| Live input/navigation | The 302px Subscribe popup was anchored to its button; its focused DOM input exposed URL/Go hints with capitalization disabled. Browser Back dismissed the popup and retained Home after asynchronous navigation completed |
| Persistent evidence | [Screenshots](screenshots/) and [native-control snapshots](evidence/) copied from this release's CI artifact |

### Checks and measured scope

| Check family | Verified evidence |
| --- | --- |
| Domain and view model | **28 tests passed in final CI**: feed identity/merge, folders and order, OPML, validation/search, read state, undo, reactive reentrancy, backup restoration, generation/busy-state transitions and bulk-confirmation rules |
| Subscription tree | **5 pure tree tests passed in final CI**: visible traversal, boundaries, distinct feed occurrences and cursor recovery |
| Native touch state | **4 motion tests passed in final CI**: tap excursion, cancellation, multiple contacts and capture loss |
| Native network bookkeeping | **2 operation tests passed in final CI**: concurrent subscriptions and replacement-generation isolation |
| JavaScript host/offline behavior | **18 tests passed in final CI**, including input-hint preservation and offline manifest/tampering checks |
| Full native host compilation | Final local check passed in **5.64 seconds**; this is compilation time, not native application startup time |
| Earlier native I/O/parser coverage | Six production native I/O tests and five article-parser/image-header tests passed in the earlier verification record; these are retained evidence, not claims that this browser run repeated them |
| Rendered browser | **23 passed in final CI**, including actual DOM input-hint assertions while the native Search and Subscribe fields had focus and caret preservation through responsive editor replacement |

The following durations are the final release's recorded scenario times.

| Scenario | What it checks | Observed step time |
| --- | --- | ---: |
| Native first paint | Loaded FerroUI controls and a nonempty rendered surface | 1,606 ms |
| Stream navigation/list shortcut | All items and the native list presentation | 1,094 ms |
| Reading actions | Scan without marking read, open, star, tag and Keep unread | 860 ms |
| Search typing | Search/Search hints, text entry and navigation without accidental article commands | 690 ms |
| Browser modifiers/composition guard | Alt/modified shortcuts and synthetic composition-key events preserve annotations | 529 ms |
| Feed subscription | URL/Go hints, browser transport, real local RSS document and Rust parsing | 857 ms |
| Preferences | Native settings menu and Compact density | 1,497 ms |
| IndexedDB reload | Saved feeds, annotations and density survive a verified new document | 872 ms |
| Conflicting writer | A stale writer reports conflict and preserves the newer committed generation | 27 ms |
| Home and article tags | Home stories open their containing feed; the inline tag editor saves tags and footer tag links open the tag stream | 1,666 ms |
| Empty-stream recovery | Empty unread recovery retains the feed, the message sizes to its content, and empty search retains its query | 1,601 ms |
| Anchored Subscribe/resize | Popup placement and width, native input hints, preserved draft/focus/selection/caret during narrow resize, appended typing, Cancel and outside dismissal | 2,117 ms |
| Subscription-tree cursor | Initial backward boundary, scan without navigation/read changes, parent collapse, re-expansion and opening the selected occurrence | 1,134 ms |
| Keyboard help | All four groups on desktop and a visible 44px close action on phone | 1,309 ms |
| Dated mark-all | Confirmation for 60 unread items preserves the age cutoff, changes only the 30 eligible items, and stores the opt-out for the next action | 1,590 ms |
| Phone touch reading | Actual touch scrolling, cancellation, multiple contacts, independent star and inline expansion | 4,009 ms |
| Browser Back | Drawer, dialog and reading menu dismiss before the reading location changes | 983 ms |
| Held subscription interactions | Cancelled hold, held menu, nested Back, persisted reorder and folder movement | 4,474 ms |
| Small/landscape dialogs | Dialog, input, close, Add and Cancel remain visible and hit-testable | 1,029 ms |
| Keyboard-sized viewport | Native viewport follows synthetic geometry while retaining dialog text and focus | 908 ms |
| Read tracking | User scrolling marks the reading line; layout/programmatic movement do not; Keep unread remains explicit | 1,420 ms |
| Virtualization | A 10,000-item library retains a bounded realized native tree | 1,110 ms |
| Offline reload | Activated Reader worker, disabled network, changed document and matching build/library | 1,829 ms |

The scenario times sum to **33,211 ms**. They include harness readiness, assertions and capture work. The **1,606 ms first-paint step** is a headless CI observation, not production download latency, a memory measurement or a device-independent startup guarantee.

#### Viewports and control counts

Desktop captures use **1363 × 936**. Touch checks use **390 × 844**, small portrait dialogs **320 × 568**, and landscape dialogs **844 × 390**. The keyboard scenario overrides the public visual-viewport values to **390 × 380**, with **offsetTop = 30** and scale 1. It verifies both the DOM host rectangle and the native dialog bounds, text and focus, then restores the viewport. This is a synthetic contract test, not a physical phone keyboard test.

The final [10,000-article stress snapshot](evidence/09-virtualized.json) contains **28 realized article containers and 939 native visual-tree elements** after scrolling. These are observed snapshot counts, not a universal bound for arbitrary article bodies or a measurement of peak memory.

Final screenshot inspection confirmed the blue focus rectangle, fully visible desktop **All items** filter, readable mobile menu labels, reachable portrait/landscape dialog actions, and blue two-pixel subscription insertion line. The touch test also activates an article through the blank area of its date column, exercising the full row target rather than only its text glyphs.

The new [empty-feed snapshot](evidence/11-empty-feed.json) measures its message border at **1101 × 134px**, at `(249, 151)` in the desktop viewport. The [empty-search message](evidence/11-empty-search.json) is **147px high**, with its additional explanatory text. Neither stretches through the viewport. The [desktop Subscribe snapshot](evidence/11-subscribe.json) records a **302px-wide** popup at `(16, 96)`, immediately below the native button. The [phone shortcut snapshot](evidence/11-mobile-shortcuts.json) records its fully visible **44 × 44px** close button at `(322, 89)`; the help body scrolls vertically.

| Evidence | Final release captures |
| --- | --- |
| Desktop appearance | [Home](screenshots/home.png), [expanded reading](screenshots/expanded.png) |
| Phone reading/navigation | [Narrow layout](screenshots/narrow.png), [touch reading](screenshots/mobile-reading.png), [navigation drawer](screenshots/mobile-navigation.png) |
| Constrained dialogs | [Small portrait](screenshots/mobile-small.png), [landscape](screenshots/mobile-landscape.png), [synthetic keyboard viewport](screenshots/mobile-keyboard.png) |
| Menus and subscription interaction | [Insertion indicator](screenshots/mobile-insertion.png), [reading actions menu](screenshots/mobile-menu.png) |
| Home and source/tag navigation | [Phone Home](screenshots/mobile-home.png), [article tags](screenshots/tags.png), [subscription-tree cursor](screenshots/tree-cursor.png) |
| Empty feedback | [Unread feed](screenshots/empty-feed.png), [search](screenshots/empty-search.png) |
| Responsive quick add | [Desktop anchored popup](screenshots/subscribe.png), [phone editor after appended typing](screenshots/mobile-subscribe.png) |
| Help and confirmation | [Desktop shortcuts](screenshots/shortcuts.png), [phone shortcuts](screenshots/mobile-shortcuts.png), [dated mark-all confirmation](screenshots/mark-confirmation.png) |

### Fixes exercised by the review

Home previews now open the selected article in its source feed, with the title, date and compact summary following the preserved overview renderer. Article footers show Add tags when empty and Edit tags with individual navigation links when populated. Empty unread feedback clears only the current stream's unread filter; empty searches retain their query. The original message box is content-sized rather than stretched to the reading viewport. The final gate explicitly rejects that previous layout defect.

Desktop Subscribe and visible-article tag editing use native-control anchors. Subscribe becomes a centered dialog on a narrow or coarse-pointer surface. Responsive editor replacement now initializes the native selection and caret before queued focus handling or reconciliation can observe the replacement. The one-time Loaded callback only focuses the current, attached editor and is removed during cleanup. The rendered resize check records caret **38** before the breakpoint and retains it after replacement; typing `#append` produces the full original URL plus that suffix, with caret **45** in the [phone snapshot](evidence/11-mobile-subscribe.json). This specifically covers the defect where a queued initialization reset the caret and inserted new text at the beginning. Active text composition also prevents a presentation remount in the implementation; physical input-method composition during rotation has not been verified.

The separate subscription-tree cursor scans visible occurrences without opening a stream or marking its articles read. Shift+X can collapse the cursor's containing folder, and Shift+O opens the selected occurrence. Pure traversal tests cover additional boundary and occurrence rules. Shortcut help now presents the original four groups, adapts them to a scrolling phone layout and keeps dismissal reachable. Mark-all confirmation uses the whole current stream's unread count above 50, carries the chosen age cutoff through confirmation and stores Don't ask me again only when confirmed. In the rendered fixture, 60 unread articles include only 30 older than the cutoff; confirmation changes those 30, and the next keyboard action completes without a second prompt.

Touch activation waits for a valid release; scrolling, capture loss, movement outside the tap allowance and multiple contacts cancel pending actions. Recycled controls resolve the currently displayed article ID instead of keeping a captured ID from an earlier row. The full row remains tappable outside its visible text. Subscription movement updates the shared feed order and folder membership atomically, with an insertion indicator and undo.

Native menus, dialogs and the narrow subscription drawer participate in browser history through a transient guard. Back dismisses the top layer and retains a guard while an underlying layer remains. Closing the final overlay consumes it; a new stream replaces it without an empty Back stop. The nested-menu check asserts menu disappearance, retained drawer and unchanged route rather than incorrectly requiring a permanent history-index decrease.

Both hosts validate a library generation before applying pending feed results. Full backup replacement and its undo invalidate earlier requests even if IDs/URLs are reused. Busy counts are generation-specific. Concurrent native subscriptions retain their results; only the latest may navigate while its starting location remains current. IndexedDB independently guards committed storage generations against stale writers. The ordinary localStorage fallback has no equivalent transactional multi-tab guarantee.

The final browser-only adapter reads the focused native `TextBox`'s `TextInputOptions` on `GotFocus`, before the framework focuses its shared browser input. It updates keyboard/return/capitalization hints without polling, field-name inference, refocusing, or changing input type, text, selection or composition. Autocorrection, spellcheck and suggestions remain off for plain TextBoxes; the framework's exact-edit textarea is untouched. Search exposes **Search/Search** and Subscribe exposes **URL/Go**. The final rendered gate checks the actual DOM attributes, and focused JavaScript tests cover preservation of text, selection and composition state while the shared input changes hints.

### Reload, offline integrity and limits

Reload checks wait for a changed `performance.timeOrigin`, with bounded readiness and protocol-error checks. The offline scenario requires Reader's activated worker, `navigator.onLine === false`, a new document after reload, a matching build identifier and all **10,000 saved articles**. Runtime-error assertions are part of the gate.

The worker verifies every cached asset's final decoded size and SHA-256 against `{ path, sha256, size }`. Two concurrent downloads bypass stale HTTP-cache entries; failure cancels and waits for outstanding work before removing only a newly created failed generation. Older caches survive until successful activation, and there is no forced `skipWaiting`. Focused tests cover final-byte manifests, canonical cache keys, activation ordering and same-length tampering rollback.

During independent live verification, older duplicate Reader clients initially kept the prior worker active. Closing the old Reader clients and opening a new tab allowed the final `23a85d01c9f9e4a8` build to activate normally. Its build identifier, Home layout, anchored Subscribe popup, focused URL/Go input and completed Back dismissal were observed in Chrome. This is a manual lifecycle observation; **an automated upgrade while an old client remains open is not covered**.

**Physical iOS/Safari keyboards and hardware gesture variation have not been checked.** Chromium's dispatched touch events and synthetic visual-viewport changes establish the host/native contract exercised here; they do not reproduce every mobile browser's keyboard, zoom or accessibility behavior. Native share/clipboard/file/print availability follows browser permissions and platform support; the test suite does not operate every OS sharing sheet or printer. External publisher images are not all cached. The narrow UI is a modern adaptation of final desktop Reader, not a verified replica of Google's separate historical mobile client. Historical pixel parity and retired Google services remain outside these verification claims.

### Earlier verification history

- [45042481284](https://github.com/FerroUI/Reader/actions/runs/45042481284), revision `d6906685190049bac9693174fc20736cfd5f4d05`, build `b7fd96dc173b6471`: the preceding **17-check** release passed with 26 domain/state, four motion, two native-operation and 18 JavaScript tests. Its rendered scenarios totaled 24,847ms; the first-paint step was 2,087ms and the local native check took 2.19 seconds. It recorded the same 28-container/939-visual stress snapshot and 26-resource offline manifest. Independent Chrome observation confirmed native Home, URL/Go hints and Back dismissal after old clients released the preceding `f26d4636d1bb37b3` worker. Its [complete verification record](https://github.com/FerroUI/Reader/blob/c8162dc3831f0a1100b033380526c41bf28e0492/docs/verification.md) preserves the earlier per-scenario durations and evidence references.
- [45055241079](https://github.com/FerroUI/Reader/actions/runs/45055241079), revision `c8162dc3831f0a1100b033380526c41bf28e0492`: the first expanded review gate exposed the caret reset during desktop-to-phone editor replacement. Screenshot review also found the stretched empty-message border. The final release above corrects both and passes all 23 scenarios, including the new empty-height assertion.
- [45041529376](https://github.com/FerroUI/Reader/actions/runs/45041529376), revision `c3779ff`: the preceding **17-check** release passed after correcting the recycled-row article identity defect exposed by `40f2f02`. The later `d690668` release added verified native keyboard hints and visual corrections.
- [45034186372](https://github.com/FerroUI/Reader/actions/runs/45034186372), revision `5a4985b4`: the earlier strict **11-check** gate established verified new-document offline reload with 10,000 articles.
- [45031751686](https://github.com/FerroUI/Reader/actions/runs/45031751686), revision `20167c92`, and [45033246424](https://github.com/FerroUI/Reader/actions/runs/45033246424), revision `b3477eb`: early screenshots informed Home typography/columns, expanded spacing, checkbox/footer icons and narrow-header fixes. Their reload assertions could inspect the old document and had an unbounded worker wait. Later strict gates supersede that reload/offline evidence.

### Reproduce

```bash
cargo test --locked -p reader-domain -p reader-state
rustc --edition=2021 --test crates/reader-ui/src/gestures/motion.rs -o target/reader-motion-tests
target/reader-motion-tests
rustc --edition=2021 --test crates/reader-ui/src/sidebar/tree.rs -o target/reader-tree-tests
target/reader-tree-tests
rustc --edition=2021 --test crates/reader-desktop/src/operations.rs -o target/reader-operation-tests
target/reader-operation-tests
node --test scripts/*.test.mjs
cargo check --locked -p reader-desktop
bash scripts/build-browser.sh
node scripts/ui-smoke.mjs target/browser/reader-browser target/ui-smoke
```

Install Chrome/Chromium and set `CHROME` if automatic discovery needs help. The harness uses a fresh isolated profile and writes PNGs, native-control JSON and success/failure evidence. It does not use personal browser state. `[verify-ui]` in a pushed commit or `workflow_dispatch` with `verify_ui: true` enables the rendered gate; its failure blocks deployment, and its evidence artifact is retained for seven days. Ordinary source pushes retain the lean behavior-tests/build/deploy path; documentation-only pushes are excluded by the workflow's path filter.

This repository preserves the artifact's exact final [results JSON](verification-results.json), twenty screenshots and fourteen selected native-control snapshots independently of its seven-day retention period. The [release metadata](evidence/release.json) ties the preserved evidence to its application revision, build identifier and workflow run.
