# Verification record

The final strict browser gate **passed all 11 checks**, including a verified new-document offline reload with **10,000 saved articles**. Both build and deployment succeeded. The evidence below belongs to the tested application revision; it does not establish 100% Google Reader visual or service parity. [parity.md](parity.md) records remaining differences and [reference.md](reference.md) records the historical sources.

## Final verified revision — 22 November 2026

| Field | Evidence |
| --- | --- |
| Workflow run | [45034186372](https://github.com/FerroUI/Reader/actions/runs/45034186372) |
| Application revision | `5a4985b4222c37e191b688add60bdba609981f8c` |
| Built application identifier | `f26d4636d1bb37b3` |
| Workflow result | Build and deploy jobs both completed successfully |
| Browser gate | **11 passed**, recorded in the artifact's `results.json` |
| CI behavior tests | **20 Rust domain/state tests and 9 JavaScript tests passed** in the same run |
| Artifact | `reader-ui-verification`; extracted results, PNGs and native-control JSON inspected under `target/ui-ci-5a4985b` |
| Application URL | [https://ferroui.com/Reader/](https://ferroui.com/Reader/); the GitHub Pages alias redirects to it |

Independent live-site verification completed in Cloud Chrome at `https://ferroui.com/Reader/#home`: the final Home layout and native controls were observed, and `data-reader-build` matched **`f26d4636d1bb37b3`** exactly. An older worker initially served its previous page; closing the old application client and opening a new tab allowed the latest version to activate normally. This was a manual observation of current-version activation, not an automated upgrade-coverage result.

The application uses the shared Rust/FerroUI UI in the browser. The final integrity change preserves the Rust UI from the previously inspected visual revision `b3477eb`; the final screenshots retain those corrections. Representative captures are [Home](screenshots/home.png), [expanded reading](screenshots/expanded.png) and [narrow reading](screenshots/narrow.png).

## Observed checks

| Check | Result | Scope |
| --- | --- | --- |
| Rust domain and view-model tests | **20 passed in final CI** | Feed merges, identity and base URLs, OPML/folders, document validation/search, selection/read behavior, stream preferences, startup navigation, reactive reentrancy, undo and save-error behavior |
| JavaScript host and offline-cache tests | **9 passed in final CI** | Six host tests for transport/routing/decoding and three tests for final-byte manifests, canonical cache keys, activation ordering and tamper rollback |
| Native host compilation | **Passed** | Final visual/API recheck completed in 1.58 seconds on Linux with upstream framework warnings |
| Native I/O tests | **6 passed locally** | Atomic replacement, recovery, coalesced writer shutdown/failure, discovery, background wake and real localhost curl redirect/discovery |
| Article parser and image-header tests | **5 passed locally** | Safe content projection, code/list/table text handling, URL exclusion and bounded image header/dimension handling |
| Rendered-browser gate | **11 passed in final CI** | Actual input, rendering, feed transport, settings, fresh-document persistence, conflicting writers, narrow navigation, virtualization and offline reload |
| Historical visual parity | **Reference-guided; complete parity not established** | Screenshots verify the implemented layouts and recorded corrections, without claiming an exhaustive pixel comparison or every original screen |

The native I/O harness was compiled directly from `crates/reader-desktop/tests/io.rs` with `rustc --test`, linking the already compiled compatible domain library. It includes the actual production I/O modules. All six tests passed in 0.03 seconds, with curl 8.14.1 present, so the localhost transport test executed. A prior cold Cargo I/O test build was intentionally interrupted to avoid rebuilding the native UI dependency graph; that interrupted command is not counted as a completed test run.

## Final browser results

| Check | Result | Headless CI step duration |
| --- | --- | ---: |
| Native first paint and nonempty surface | Passed | 1,882 ms |
| Stream navigation and list-view shortcut | Passed | 1,261 ms |
| Scan, open, star, tag and Keep unread | Passed | 886 ms |
| Search typing without triggering reading shortcuts | Passed | 707 ms |
| Local RSS subscription through browser transport and Rust parser | Passed | 920 ms |
| Settings menu and Compact density | Passed | 1,396 ms |
| IndexedDB persistence across a verified new-document reload | Passed | 900 ms |
| Independent stale writer preserves the newer saved generation | Passed | 31 ms |
| Narrow layout and navigation drawer | Passed | 1,155 ms |
| Bounded native control tree for 10,000 articles | Passed | 1,342 ms |
| Installed application and saved library reload without network | Passed | 3,239 ms |

These durations measure harness steps in headless CI, including readiness and capture work. In particular, **1,882 ms is the first-paint verification step**, not production network download time, general page-load latency or a cross-device performance guarantee.

The desktop viewport was **1363 × 936** and the narrow viewport **390 × 844**. The final stress snapshot `09-virtualized.json` records **28 realized article containers and 904 native visual-tree elements** after scrolling a 10,000-article list. These are observed snapshot counts, not peak memory or a universal bound for arbitrary article bodies.

### Reload and offline proof

The final gate corrects the earlier harness's navigation race. It records `performance.timeOrigin` before navigation or reload and waits for a changed value before inspecting the new application. It also checks protocol errors and uses bounded readiness/worker-activation waits.

The offline scenario passed with all of these conditions:

1. A newly loaded document registered Reader's own worker and obtained an activated controller.
2. The browser network was disabled and `navigator.onLine` was confirmed false.
3. Reload produced a **new document**, rather than satisfying assertions against the previous page.
4. The reloaded application retained build identifier `f26d4636d1bb37b3`.
5. The restored library contained all **10,000 saved articles**.

The result and `10-offline.png`/`10-offline.json` now supply the fresh-document evidence missing from the earlier runs. The harness's tracked runtime-error assertions also passed.

### Inspected visual corrections

The captured screens verify the revised Home columns, section heading and feed-title typography; expanded-title/body alignment and card spacing; blue settings checkboxes; vector footer stars; and the narrow wordmark without the previous search-field collision. The calibrated desktop layout places the first expanded card at y = 172 and the Refresh button at y = 77.

The artifact contains Home, expanded/list reading, search, subscription, settings, Compact density, narrow reading/drawer, virtualization and offline screenshots with corresponding native-control snapshots. Branding remains a native recreation, font rasterization differs from the original browser DOM, and Home/Explore/content projection retain the documented adaptations.

## Offline-cache integrity

The generated manifest records `{ path, sha256, size }` for each asset's final bytes, including the stamped index. Installation fetches two assets concurrently using `cache: "reload"` and a build-specific query, checks decoded size and SHA-256, and stores verified responses under canonical URLs. Failure cancels and waits for outstanding jobs before removing only a newly created failed generation. Previous caches survive until successful activation; the worker does not call `skipWaiting`.

Three focused tests execute the actual generated worker handlers with Web Crypto/response streams and simulated network/CacheStorage. They verify final-byte hashes and idempotent preparation, canonical cache keys and activation ordering, and rejection of same-length tampering while preserving the previous cache. The final browser gate separately verifies installation and a real offline reload.

**A browser upgrade while an old client remains open has not been exercised end to end.** The test record also does not cover every browser, device, publisher HTML structure, native operating system or historical Google Reader screen.

## Earlier verification runs

- [45031751686](https://github.com/FerroUI/Reader/actions/runs/45031751686), revision `20167c92`: ten script checks passed and exposed footer-glyph, narrow-header and layout refinements through screenshots.
- [45033246424](https://github.com/FerroUI/Reader/actions/runs/45033246424), revision `b3477eb`: eleven script checks passed, and inspected screenshots verified those visual corrections.

Release review found that both earlier scripts could inspect an old document immediately after requesting reload, and their service-worker readiness promise was not bounded. Their recorded visual/input evidence remains useful, but their reload/offline assertions were qualified. **The final run above supersedes those reload/offline results with explicit new-document, controller, network and build-identity checks.**

## Reproduce the checks

For ordinary fast checks:

```bash
cargo test --locked -p reader-domain -p reader-state
node --test scripts/*.test.mjs
cargo check --locked -p reader-desktop
```

For rendered-browser verification:

```bash
bash scripts/build-browser.sh
node scripts/ui-smoke.mjs target/browser/reader-browser target/ui-smoke
```

Install Chrome or Chromium first; set `CHROME` to its executable if automatic discovery cannot find it. The harness serves the built site and creates a fresh isolated profile, separate from personal browser state. It writes PNG screenshots and JSON control snapshots, plus `results.json` on success or available failure evidence.

In GitHub Actions, include `[verify-ui]` in a pushed commit message or dispatch the workflow with `verify_ui: true`. The optional gate uploads `reader-ui-verification` with seven-day retention and blocks deployment if it fails. Ordinary pushes retain domain/state tests, focused JavaScript tests, the actual FerroUI browser build and deployment.
