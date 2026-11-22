# Verification record

This document separates observed results from checks that have been implemented but have not yet completed. Test counts do not establish 100% Google Reader feature or visual parity; [parity.md](parity.md) describes the remaining differences and [reference.md](reference.md) records the original-client sources.

## Observed checks — 22 November 2026

| Check | Observed result | Scope and limits |
| --- | --- | --- |
| Rust domain and view-model tests | **20 passed** | Feed merges, identity and redirect/base handling, OPML/folders, document validation/search, selection/read behavior, per-stream preferences, startup navigation, reactive reentrancy, undo and save-error behavior |
| JavaScript host tests | **6 passed** | URL/proxy validation, route encoding, text decoding, bounded/deduplicated fetches, explicit proxy fallback and redirected refresh identity/base handling |
| Native host compilation | **Passed** | `CARGO_BUILD_JOBS=1 cargo check --locked --offline -p reader-desktop`; initial check completed in 21.06 seconds and the Print API follow-up recheck in 5.17 seconds on Linux, with upstream framework warnings |
| Native I/O tests | **6 passed** | Atomic replacement, malformed-library recovery, coalesced writer shutdown/failure, safe discovery, background wake and real localhost curl redirect/discovery behavior |
| Article parser and image-header tests | **5 passed** | Safe content projection, code/list/table text handling, URL exclusion and bounded image header/dimension handling |
| Rendered-browser interaction gate | **10 passed** | Actual Chromium input, rendering, feed transport, persistence, conflicting writers, narrow navigation and a 10,000-article fixture; evidence detailed below |
| Offline reload gate | **Pending / unrun** | Added after the successful ten-check revision; requires the next requested browser run |
| Visual comparison with preserved Google Reader | **Reference-guided; complete parity not established** | Captured screens are inspected against the historical layout, but no exhaustive pixel comparison or complete original-screen matrix has passed |

The native I/O test target was compiled directly from `crates/reader-desktop/tests/io.rs` with `rustc --test`, linking the previously compiled compatible `reader-domain` library. The same production I/O modules are included by that harness. All six tests passed in 0.03 seconds; curl 8.14.1 was present, so the localhost transport test executed rather than taking its no-curl skip path. An earlier cold `cargo test -p reader-desktop --test io` build was intentionally interrupted to avoid recompiling the full native UI dependency graph; that Cargo command is not represented as a completed test run.

The native recheck includes the updated Print request pattern accepting its content-base field. Counts and timings record the revisions actually exercised rather than a guarantee that every subsequent commit has run the same gates.

## Completed browser verification

| Field | Value |
| --- | --- |
| Workflow | Build and publish, optional rendered-browser gate |
| Run | [45031751686](https://github.com/FerroUI/Reader/actions/runs/45031751686) |
| Tested revision | `20167c92b9775a51e0153702b3f3c5bdc4c13887` |
| Observed workflow status | Build and deploy jobs both completed successfully |
| Browser test result | **10 passed**, as recorded in `results.json` |
| Evidence artifact | `reader-ui-verification`; PNGs, native-control JSON and results inspected from the extracted `target/ui-ci-20167c9` directory |
| Pages deployment outcome | Success for the tested revision; live browser startup and native controls observed at [ferroui.com/Reader](https://ferroui.com/Reader/#home) |

The workflow status was read from GitHub, and the artifact's `results.json` confirms ten successful checks. The deployed application was also opened in a live browser through the Pages redirect. The assertions and screenshots cover the specified paths; they do not establish complete historical visual or service parity.

### Executed browser checks

| Check | Result | CI step duration |
| --- | --- | ---: |
| Native first paint and nonempty rendered surface | Passed | 3,225 ms |
| Stream navigation and list-view shortcut | Passed | 1,516 ms |
| Scan, open, star, tag and Keep unread through actual input | Passed | 972 ms |
| Search editing without triggering reading shortcuts | Passed | 886 ms |
| Local RSS subscription through browser transport and Rust parser | Passed | 920 ms |
| Settings menu and Compact density | Passed | 1,318 ms |
| IndexedDB persistence across reload | Passed | 1,192 ms |
| Independent stale writer preserves the newer saved generation | Passed | 15 ms |
| Narrow layout and navigation drawer | Passed | 1,056 ms |
| Bounded native control tree with 10,000 articles | Passed | 1,138 ms |

These are durations of harness steps in headless CI. In particular, **3,225 ms is the first-paint verification step**, including readiness/capture work in the local test environment. It is not production page-load latency, network download time, a general responsiveness benchmark or a cross-device performance guarantee.

The desktop viewport was **1363 × 936**, and the narrow viewport **390 × 844**. The captured 10,000-article stress view contained **29 realized article containers and 923 native visual-tree elements** after scrolling (`09-virtualized.json`). These are counts in that snapshot, not a peak-memory measurement or a bound on every article layout.

The current script adds an eleventh check: application and saved-library reload with network access disabled. **That offline check did not run in this successful ten-check revision and remains pending the next requested run.**

### Inspected evidence

| Observation | Result |
| --- | --- |
| Final workflow conclusion and tested revision | Success; `20167c92b9775a51e0153702b3f3c5bdc4c13887` |
| `results.json` passed count | 10; no failing check recorded |
| Desktop Home / expanded / list screenshots | `01-home.png`, `02-expanded.png`, `03-list.png`, with matching native-control JSON |
| Search, subscription and settings screenshots | `04-search.png`, `05-feed.png`, `06-settings.png`, `07-compact.png` |
| Narrow layout and drawer screenshots | `08-mobile.png`, `08-mobile-navigation.png`, both 390 × 844 |
| Virtualization evidence | `09-virtualized.png` and JSON: 29 article containers, 923 visual elements, 10,000 records |
| Offline reload evidence | Pending; no `10-offline` evidence exists in this artifact |
| Runtime error tracking | The harness's runtime-error assertions passed for the ten executed paths |
| Remaining observed visual issues in this revision | Footer star text displayed a missing-glyph square; the narrow wordmark was clipped by search; reference calibration identified padding, vertical alignment and Home-layout refinements |
| Published URL and deployment revision | [https://ferroui.com/Reader/](https://ferroui.com/Reader/), deployed from the tested revision; the GitHub Pages alias redirects to it |

### Follow-up visual refinements

After inspecting the first artifact, the UI work addresses the footer glyph with the shared vector icon, narrow logo/search spacing, expanded-title/body padding, first-card/toolbar vertical alignment and Home columns/heading. These changes are separate from the successful evidence above. Record their next tested revision and screenshots after the subsequent run; the first artifact does not prove that later edits have passed.

Branding remains a native recreation, font rasterization differs from the original browser DOM, and Home/Explore/content projection retain the documented adaptations. The ten successful scenarios are not a complete screen-by-screen reproduction of Google Reader.

## Reproduce the checks

For the ordinary fast gates:

```bash
cargo test --locked -p reader-domain -p reader-state
node --test scripts/host.test.mjs
cargo check --locked -p reader-desktop
```

For rendered-browser verification, build the application first, install Chrome or Chromium and run:

```bash
bash scripts/build-browser.sh
node scripts/ui-smoke.mjs target/browser/reader-browser target/ui-smoke
```

The harness automatically serves the built output and creates a fresh isolated Chromium profile, keeping the test library and service-worker cache separate from personal browser state. Set `CHROME` to a specific executable when needed. The output directory contains PNG screenshots and JSON control-tree snapshots, plus `results.json` on success or available `failure.png`/`failure.json` evidence on failure.

In GitHub Actions, include `[verify-ui]` in a pushed commit message or manually dispatch the workflow with `verify_ui: true`. The optional gate uploads the `reader-ui-verification` artifact with seven-day retention, including failure evidence when available. A failing requested gate prevents that run's Pages deployment. Ordinary pushes retain domain/state tests, focused JavaScript tests, the actual FerroUI browser build and deployment.

## Corrections covered by this pass

The Reader persistence adapter is named `reader-storage.js` because the framework also stages a module named `storage.js`. The old filename collided with that platform module and prevented browser startup. The Reader imports now use the distinct name. Startup, IndexedDB reload and conflicting-writer checks passed through that renamed integration in the ten-check browser run.

Feed transport preserves the original subscription identity while resolving links against the final response URL and applicable XML/content base. Native transport performs this parsing on a worker thread. These cases have local domain/host/I/O coverage; the passing browser subscription fixture additionally exercises the compiled network-to-UI path.
