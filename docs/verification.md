# Verification record

## Current article-reading and scrolling release — 22 November 2026

Publisher articles now open inside the native reading pane through **Read full article**, with selectable text, inline links, structured tables, horizontally scrolling code and responsive images. The feed body remains available, and successful downloads are saved with the library. The stream now reuses FerroUI's TreeDataGrid presenter for variable-height article realization. See [article-reading.md](article-reading.md) for behavior, architecture and connection limits.

| Evidence | Verified result |
| --- | --- |
| Application revision | [`2463a5af5d4cce7ee7c1f268b40a47c8c1e7cc99`](https://github.com/FerroUI/Reader/commit/2463a5af5d4cce7ee7c1f268b40a47c8c1e7cc99) |
| Workflow | [45080772694](https://github.com/FerroUI/Reader/actions/runs/45080772694): build job **137388235115** and Pages deploy job **137390272579** succeeded |
| Build identifier | `f3ffc9bb9161ea03` |
| Rust and JavaScript | **94 Rust tests** and **54 JavaScript tests** passed; the final local native desktop check also passed |
| Rendered browser | **33 scenarios passed**; names and durations are preserved in [article-verification-results.json](article-verification-results.json) |
| Offline application | **37 verified resources**, including the extraction worker and its matching WASM; a new-document offline reload retained the installed build and **10,000 saved articles** |
| Release metadata | [article-release.json](evidence/article-release.json) records the exact revision, jobs, counts, snapshot metrics, assertion tolerances and artifact hashes |

The recorded browser scenario durations total **60,129ms**. These include harness readiness, interactions, assertions and captures; they are not device-independent performance measurements. The measurements above come from the final CI artifact and deployment. The manual live-site observations below are identified separately from these controlled fixtures.

### Native article reading

The controlled publisher fixture exercises an explicit download, local Rust extraction and native rendering. It verifies that ordinary feed reading does not request the website; feed/full-content switching reuses the saved copy. Cancellation and an extraction error retain the last readable article. A new document restores the saved copy, and it reopens with the network disabled without another publisher request. Recorded requests contain neither cookies nor authorization headers.

Native text selection and copying pass with the selected text **“Publisher body marker. ”**. Pointer and keyboard inline-link activation, horizontally scrolling code, table content and publisher-relative URLs are exercised. Publisher scripts do not execute, and the fixture rejects DOM article/iframe substitution for the native reading surface. At viewport widths **390 → 320 → 390px**, the image follows **320 → 250 → 320px**, within the asserted **1px** tolerance, without requiring a breakpoint change.

| Preserved evidence | Content |
| --- | --- |
| [Desktop article](screenshots/14-article-native.png) / [control tree](evidence/14-article-native.json) | Saved full article, native text selection, structured table, code, nested list, quote and inline link |
| [Inline error](screenshots/14-article-error.png) / [control tree](evidence/14-article-error.json) | Failed extraction preserves the saved readable copy |
| [Phone article](screenshots/14-article-mobile.png) / [control tree](evidence/14-article-mobile.json) | Native table and code contained in a 390px viewport |
| [320px image layout](screenshots/14-article-mobile-small-image.png) / [control tree](evidence/14-article-mobile-small-image.json) | Image and article content respond within the phone breakpoint |
| [Article assertions](evidence/14-article-verification.json) | Selection/copy result, persistence, cancellation, error preservation, responsive widths and exact fixture requests |

### Large, nonuniform streams

The scrolling fixture loads **1,200 expanded articles** with short items, long paragraphs, wide code and tall images. Its first far seek targets an item immediately after **2,400px of declared images** whose unmeasured estimate is much smaller. Further backward/forward seeks and a rapid wheel burst must realize the requested item in the actual viewport, keep finite scroll geometry and retain fewer than **80 article containers / 9,000 native visuals** at each checked point.

| Final snapshot | Library size | Article containers | Native visuals | Scroll extent |
| --- | ---: | ---: | ---: | ---: |
| [Mixed-height stream](screenshots/15-mixed-height-stream.png) / [tree](evidence/15-mixed-height-stream.json) | 1,200 | **2** | **618** | 1,020,255px |
| [After delayed image decode](screenshots/15-image-scroll-anchor.png) / [tree](evidence/15-image-scroll-anchor.json) | 1,200 | **1** | **520** | 1,026,233px |
| [Phone long article](screenshots/15-phone-long-article.png) / [tree](evidence/15-phone-long-article.json) | 1,200 | **1** | **289** | 1,615,139px |
| [10,000-item list tree](evidence/article-10000-list.json) | 10,000 | **56** | **1,471** | 280,000px |

These are retained control-tree snapshot counts, not peak-memory measurements or universal bounds. Unvisited article heights remain estimates until measured. The 10,000-item capture is stored separately from the older release's `09-virtualized.json`.

Read/star changes must preserve the article position, scroll offset and measured extent within **1.5px**, while the retained template displays the changed star action. Keyboard scanning brings the next item into view without marking it read; selecting the following item does mark it read. A delayed image must increase the actual article height by **more than 400px**, while the visible paragraph moves **no more than 3px**. Desktop-to-phone reflow must preserve the same paragraph's viewport-relative position within **3px**. These values are passing assertion tolerances, not recorded exact displacements. A trusted touch pan beginning on native rich text advances the viewport without changing read state; a subsequent accepted tap marks only its own article read.

Virtualization is **at article level**: body blocks inside a realized article still participate in layout. The body projection bounds pathological complexity, but this release does not implement block virtualization. Saved article text is available offline; images use a separate memory cache and may require a connection after reload. The controlled fixture does not establish public-provider availability, physical iOS behavior or complete historical Google Reader parity.

### Independent observations on the published application

The published [Reader application](https://ferroui.com/Reader/#home) initially served the previous offline build `ad7a2095a4e2e65c`. A normal reload followed by closing and reopening the Reader client activated **`f3ffc9bb9161ea03`**, preserving all **eight existing subscriptions and 167 unread articles**. No storage reset, worker unregister or forced activation was used.

On the Rust feed, **Read full article** downloaded **“Enabling the parallel frontend on nightly”** and displayed **Full article · saved**. Native headings, paragraphs, links and a table rendered, and scrolling reached the publisher's Nalgebra benchmark image. On BBC News, **“Explosion on Russian ship at Libya port caused by suspected drone attack, officials say”** expanded from a one-sentence feed summary into a saved article with multiple paragraphs. These manual observations establish successful reading from two actual publishers; they do not identify whether each request used direct access or a proxy.

Returning to the Rust feed and performing an ordinary reload retained its saved full article and native controls. Normal reading scroll marked that article read, leaving **166 total unread and 9 Rust unread**, as expected. The final Rust reading tab was left available for review. No application extraction error was observed. These are observations of this library and session, not guarantees of publisher availability or item counts.

The initial [45078408836](https://github.com/FerroUI/Reader/actions/runs/45078408836) gate caught a regression where clicking native article text did not select the article and mark it read; it **did not deploy**. The text control handled the press before the ancestor handler. The fix restores the handled mouse path and guards touch release. The final gate passes that article-selection assertion, along with text selection, copying, links and all 33 browser scenarios. Earlier records below retain their own revision identifiers, screenshots and measurements.

## Previous real-feed startup release — revision 59fee66

Fresh libraries now start with six real subscriptions: **Hacker News, Ars Technica and The Verge** in Technology; **Rust Blog and GitHub Blog** in Development; and **BBC News** in News. Production starts with **zero articles**, saves the subscriptions and fetches their publishers on first use. It does not manufacture stories. The feed addresses and connection behavior are documented in [feed-connection.md](feed-connection.md).

| Evidence | Observed result |
| --- | --- |
| Workflow | [45071747165](https://github.com/FerroUI/Reader/actions/runs/45071747165): checks, browser build and Pages deployment succeeded |
| Application revision | [`59fee6683af9c34a3816f04b2ae53b8c2a58a53d`](https://github.com/FerroUI/Reader/commit/59fee6683af9c34a3816f04b2ae53b8c2a58a53d); initial starter implementation in [`a15b7799ce81438edf0f96b60b718621884cd863`](https://github.com/FerroUI/Reader/commit/a15b7799ce81438edf0f96b60b718621884cd863) |
| Build identifier | `ad7a2095a4e2e65c`, also observed in the deployed page's DOM |
| Rust checks | **46 domain/state**, **5 tree**, **4 motion** and **2 native-operation** tests passed; the native desktop host also compiled locally |
| JavaScript checks | **41 passed** |
| Rendered checks | **26 passed**, with recorded scenario durations totaling **41,690ms**; exact names and durations preserved in [starter-verification-results.json](starter-verification-results.json) |
| Offline | New-document offline reload passed with **31 verified resources** |
| Release metadata | [starter-release.json](evidence/starter-release.json) records the revision, build, workflow, counts and live observations |

### Production startup and preservation

The article generator in `reader-domain` is gated behind the `test-support` feature and is excluded from normal production builds. The browser `?demo=1` and desktop `--demo` entry points have been removed. Source-level tests verify six initial subscriptions, three folders, zero initial articles, persistence before network completion and non-overlapping fetch batches. Retry coverage verifies that completed feeds are skipped and a removed default is not reintroduced.

The migration tests separately cover former sample libraries, preserving real articles, annotations, customized subscriptions and settings while removing only recognized illustrative content. They also cover previously removed feeds, an intentionally empty saved library, explicit backup restoration, changed content resembling sample identifiers and identifier exhaustion. These Rust checks establish the migration cases; the rendered checks below establish the browser startup and persistence behavior.

### What the starter browser fixture proves

The isolated Chromium scenario intercepts the **six actual default feed URLs** and holds their responses until Reader renders its saved subscription metadata with **zero articles**. It then supplies controlled RSS documents containing **two test articles per feed**, yielding **12 deterministic test articles** parsed through the production feed pipeline. These articles exist only in the test fixture; they are not embedded production stories or evidence of public-provider availability.

All six URLs are requested exactly once. After downloading, the scenario removes one default, renames another, changes density and adds a star, note and tag. A new-document reload preserves those edits and does not restore the removed subscription. A separate scenario saves an intentionally empty library and verifies that reopening keeps it empty. Their recorded durations are **1,520ms** and **1,278ms**, respectively.

| Preserved evidence | Content |
| --- | --- |
| [Initial subscription metadata](evidence/13-starter-loading.json) | Six real URLs, `sample: false`, no completed fetches and zero articles |
| [Loading screenshot](screenshots/13-starter-loading.png) / [control tree](evidence/13-starter-loading-view.json) | Initial native Home render before fixture responses are released; copied from this release artifact's `01-home.png` and `01-home.json` |
| [Fetched-library screenshot](screenshots/13-starter-ready.png) / [control tree](evidence/13-starter-ready.json) | The 12 controlled articles after automatic fetch and Rust parsing |
| [Empty-library screenshot](screenshots/13-starter-empty.png) / [control tree](evidence/13-starter-empty.json) | An intentionally empty saved library after reopening |
| [Startup assertions](evidence/13-starter-verification.json) | Exact requested URLs, retained subscription identifiers, removed-default preservation, annotations and empty-library preservation |

The preceding [45070869096](https://github.com/FerroUI/Reader/actions/runs/45070869096) gate stopped deployment when its center click landed in the blank space beside the date in a short Home preview and failed to navigate. The fix gives the preview an explicit transparent hit surface. The same assertion passed unchanged in the final 26-check gate; the failed run did not deploy.

### Independent observations on the published application

Chrome loaded build `ad7a2095a4e2e65c` after a normal close and reopen allowed the offline update to activate, without clearing storage or forcing service-worker activation. The existing mixed library retained all **eight subscriptions**, including its previously added BBC News and official Hacker News feeds. Its six former sample subscriptions each fetched ten real publisher articles; Home displayed real Rust, NASA, Smashing Magazine and Mozilla headlines, with the sample-library footer absent. The observed total was **167 unread articles**, including the retained BBC count of 97 and Hacker News count of 10; these are observations, not provider guarantees.

Explore displayed all six current default suggestions, with Read actions for the existing Hacker News, Rust and BBC subscriptions and Subscribe actions for Ars Technica, The Verge and GitHub Blog. Choosing Hacker News's native Read control opened its existing ten-item feed, confirming subscription reuse. These live observations establish real publisher fetching and preservation in an existing library; the controlled browser fixture separately verifies the complete fresh-library bootstrap. Public-provider availability and item counts remain external dependencies.

## Previous feed-connection release — revision eba78c5

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
