# Verification record

The final review release passed **all 17 rendered-browser checks**, including native touch interaction, nested browser-history dismissal, subscription reordering, native input hints, keyboard-sized dialogs and a new-document offline reload with **10,000 saved articles**. Its build and deployment succeeded, and the deployed build was independently observed in Chrome. These results verify the recorded implementation and scenarios; they do not establish 100% historical visual or service parity. See [parity.md](parity.md) for boundaries and [reference.md](reference.md) for the original sources.

## Reviewed release — 22 November 2026

| Evidence | Observed result |
| --- | --- |
| Workflow | [45042481284](https://github.com/FerroUI/Reader/actions/runs/45042481284): checks, build and deploy succeeded |
| Application revision | [`d6906685190049bac9693174fc20736cfd5f4d05`](https://github.com/FerroUI/Reader/commit/d6906685190049bac9693174fc20736cfd5f4d05) |
| Application build identifier | `b7fd96dc173b6471` |
| Rendered checks | **17 passed**; exact output preserved in [verification-results.json](verification-results.json) |
| Offline manifest | **26 resources**, verified by decoded size and SHA-256 during installation |
| Independent live check | Chrome loaded [ferroui.com/Reader/#home](https://ferroui.com/Reader/#home); its DOM build identifier matched `b7fd96dc173b6471`, with native Home controls visible |
| Live input/navigation | Subscribe exposed URL/Go keyboard hints with capitalization/correction disabled; browser Back closed the dialog and retained Home |
| Persistent evidence | [Screenshots](screenshots/) and [native-control snapshots](evidence/) copied from this release's CI artifact |

## Checks and measured scope

| Check family | Verified evidence |
| --- | --- |
| Domain and view model | **26 tests passed in final CI**: feed identity/merge, folders and order, OPML, validation/search, read state, undo, reactive reentrancy, backup restoration and generation/busy-state transitions |
| Native touch state | **4 motion tests passed in final CI**: tap excursion, cancellation, multiple contacts and capture loss |
| Native network bookkeeping | **2 operation tests passed in final CI**: concurrent subscriptions and replacement-generation isolation |
| JavaScript host/offline behavior | **18 tests passed in final CI**, including input-hint preservation and offline manifest/tampering checks |
| Full native host compilation | Final local check passed in **2.19 seconds**; this is compilation time, not native application startup time |
| Earlier native I/O/parser coverage | Six production native I/O tests and five article-parser/image-header tests passed in the earlier verification record; these are retained evidence, not claims that this browser run repeated them |
| Rendered browser | **17 passed in final CI**, including actual DOM input-hint assertions while the native Search and Subscribe fields had focus |

The following durations are the final release's recorded scenario times.

| Scenario | What it checks | Observed step time |
| --- | --- | ---: |
| Native first paint | Loaded FerroUI controls and a nonempty rendered surface | 2,087 ms |
| Stream navigation/list shortcut | All items and the native list presentation | 1,154 ms |
| Reading actions | Scan without marking read, open, star, tag and Keep unread | 1,241 ms |
| Search typing | Search/Search hints, text entry and navigation without accidental article commands | 708 ms |
| Browser modifiers/composition guard | Alt/modified shortcuts and synthetic composition-key events preserve annotations | 467 ms |
| Feed subscription | URL/Go hints, browser transport, real local RSS document and Rust parsing | 854 ms |
| Preferences | Native settings menu and Compact density | 1,424 ms |
| IndexedDB reload | Saved feeds, annotations and density survive a verified new document | 762 ms |
| Conflicting writer | A stale writer reports conflict and preserves the newer committed generation | 127 ms |
| Phone touch reading | Actual touch scrolling, cancellation, multiple contacts, independent star and inline expansion | 4,284 ms |
| Browser Back | Drawer, dialog and reading menu dismiss before the reading location changes | 1,018 ms |
| Held subscription interactions | Cancelled hold, held menu, nested Back, persisted reorder and folder movement | 4,507 ms |
| Small/landscape dialogs | Dialog, input, close, Add and Cancel remain visible and hit-testable | 1,037 ms |
| Keyboard-sized viewport | Native viewport follows synthetic geometry while retaining dialog text and focus | 913 ms |
| Read tracking | User scrolling marks the reading line; layout/programmatic movement do not; Keep unread remains explicit | 1,409 ms |
| Virtualization | A 10,000-item library retains a bounded realized native tree | 1,049 ms |
| Offline reload | Activated Reader worker, disabled network, changed document and matching build/library | 1,806 ms |

The scenario times sum to **24,847 ms**. They include harness readiness, assertions and capture work. The **2,087 ms first-paint step** is a headless CI observation, not production download latency, a memory measurement or a device-independent startup guarantee.

### Viewports and control counts

Desktop captures use **1363 × 936**. Touch checks use **390 × 844**, small portrait dialogs **320 × 568**, and landscape dialogs **844 × 390**. The keyboard scenario overrides the public visual-viewport values to **390 × 380**, with **offsetTop = 30** and scale 1. It verifies both the DOM host rectangle and the native dialog bounds, text and focus, then restores the viewport. This is a synthetic contract test, not a physical phone keyboard test.

The final [10,000-article stress snapshot](evidence/09-virtualized.json) contains **28 realized article containers and 939 native visual-tree elements** after scrolling. These are observed snapshot counts, not a universal bound for arbitrary article bodies or a measurement of peak memory.

Final screenshot inspection confirmed the blue focus rectangle, fully visible desktop **All items** filter, readable mobile menu labels, reachable portrait/landscape dialog actions, and blue two-pixel subscription insertion line. The touch test also activates an article through the blank area of its date column, exercising the full row target rather than only its text glyphs.

| Evidence | Final release captures |
| --- | --- |
| Desktop appearance | [Home](screenshots/home.png), [expanded reading](screenshots/expanded.png) |
| Phone reading/navigation | [Narrow layout](screenshots/narrow.png), [touch reading](screenshots/mobile-reading.png), [navigation drawer](screenshots/mobile-navigation.png) |
| Constrained dialogs | [Small portrait](screenshots/mobile-small.png), [landscape](screenshots/mobile-landscape.png), [synthetic keyboard viewport](screenshots/mobile-keyboard.png) |
| Menus and subscription interaction | [Insertion indicator](screenshots/mobile-insertion.png), [reading actions menu](screenshots/mobile-menu.png) |

## Fixes exercised by the review

Touch activation waits for a valid release; scrolling, capture loss, movement outside the tap allowance and multiple contacts cancel pending actions. Recycled controls resolve the currently displayed article ID instead of keeping a captured ID from an earlier row. The full row remains tappable outside its visible text. Subscription movement updates the shared feed order and folder membership atomically, with an insertion indicator and undo.

Native menus, dialogs and the narrow subscription drawer participate in browser history through a transient guard. Back dismisses the top layer and retains a guard while an underlying layer remains. Closing the final overlay consumes it; a new stream replaces it without an empty Back stop. The nested-menu check asserts menu disappearance, retained drawer and unchanged route rather than incorrectly requiring a permanent history-index decrease.

Both hosts validate a library generation before applying pending feed results. Full backup replacement and its undo invalidate earlier requests even if IDs/URLs are reused. Busy counts are generation-specific. Concurrent native subscriptions retain their results; only the latest may navigate while its starting location remains current. IndexedDB independently guards committed storage generations against stale writers. The ordinary localStorage fallback has no equivalent transactional multi-tab guarantee.

The final browser-only adapter reads the focused native `TextBox`'s `TextInputOptions` on `GotFocus`, before the framework focuses its shared browser input. It updates keyboard/return/capitalization hints without polling, field-name inference, refocusing, or changing input type, text, selection or composition. Autocorrection, spellcheck and suggestions remain off for plain TextBoxes; the framework's exact-edit textarea is untouched. Search exposes **Search/Search** and Subscribe exposes **URL/Go**. The final rendered gate checks the actual DOM attributes, and focused JavaScript tests cover preservation of text, selection and composition state while the shared input changes hints.

## Reload, offline integrity and limits

Reload checks wait for a changed `performance.timeOrigin`, with bounded readiness and protocol-error checks. The offline scenario requires Reader's activated worker, `navigator.onLine === false`, a new document after reload, a matching build identifier and all **10,000 saved articles**. Runtime-error assertions are part of the gate.

The worker verifies every cached asset's final decoded size and SHA-256 against `{ path, sha256, size }`. Two concurrent downloads bypass stale HTTP-cache entries; failure cancels and waits for outstanding work before removing only a newly created failed generation. Older caches survive until successful activation, and there is no forced `skipWaiting`. Focused tests cover final-byte manifests, canonical cache keys, activation ordering and same-length tampering rollback.

During independent live verification, an older duplicate Reader client initially retained the earlier `f26d4636d1bb37b3` worker. Closing all old Reader clients and opening a new tab allowed the final `b7fd96dc173b6471` build to activate normally. This is a manual lifecycle observation; **an automated upgrade while an old client remains open is not covered**.

**Physical iOS/Safari keyboards and hardware gesture variation have not been checked.** Chromium's dispatched touch events and synthetic visual-viewport changes establish the host/native contract exercised here; they do not reproduce every mobile browser's keyboard, zoom or accessibility behavior. Native share/clipboard/file/print availability follows browser permissions and platform support; the test suite does not operate every OS sharing sheet or printer. External publisher images are not all cached. The narrow UI is a modern adaptation of final desktop Reader, not a verified replica of Google's separate historical mobile client. Historical pixel parity and retired Google services remain outside these verification claims.

## Earlier verification history

- [45041529376](https://github.com/FerroUI/Reader/actions/runs/45041529376), revision `c3779ff`: the preceding **17-check** release passed after correcting the recycled-row article identity defect exposed by `40f2f02`. The final release above adds and verifies native keyboard hints and the final visual corrections.
- [45034186372](https://github.com/FerroUI/Reader/actions/runs/45034186372), revision `5a4985b4`: the earlier strict **11-check** gate established verified new-document offline reload with 10,000 articles.
- [45031751686](https://github.com/FerroUI/Reader/actions/runs/45031751686), revision `20167c92`, and [45033246424](https://github.com/FerroUI/Reader/actions/runs/45033246424), revision `b3477eb`: early screenshots informed Home typography/columns, expanded spacing, checkbox/footer icons and narrow-header fixes. Their reload assertions could inspect the old document and had an unbounded worker wait. Later strict gates supersede that reload/offline evidence.

## Reproduce

```bash
cargo test --locked -p reader-domain -p reader-state
rustc --edition=2021 --test crates/reader-ui/src/gestures/motion.rs -o target/reader-motion-tests
target/reader-motion-tests
rustc --edition=2021 --test crates/reader-desktop/src/operations.rs -o target/reader-operation-tests
target/reader-operation-tests
node --test scripts/*.test.mjs
cargo check --locked -p reader-desktop
bash scripts/build-browser.sh
node scripts/ui-smoke.mjs target/browser/reader-browser target/ui-smoke
```

Install Chrome/Chromium and set `CHROME` if automatic discovery needs help. The harness uses a fresh isolated profile and writes PNGs, native-control JSON and success/failure evidence. It does not use personal browser state. `[verify-ui]` in a pushed commit or `workflow_dispatch` with `verify_ui: true` enables the rendered gate; its failure blocks deployment, and its evidence artifact is retained for seven days. Ordinary source pushes retain the lean behavior-tests/build/deploy path; documentation-only pushes are excluded by the workflow's path filter.

This repository preserves the artifact's exact final [results JSON](verification-results.json), ten screenshots and five selected native-control snapshots independently of its seven-day retention period.
