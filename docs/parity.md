# Reader fidelity and behavior contract

Reader targets the final desktop Google Reader experience. [reference.md](reference.md) records the original dimensions, colors, shortcuts and source URLs. This document defines the behavior to verify and the boundaries of a static browser application. A completed build alone does not certify every row or pixel; release claims must reflect observed behavior.

## Implemented coverage and remaining differences

The following describes the checked-in implementation. It is a source-level feature inventory, not certification of every interaction on every browser or operating system.

| Area | Present implementation | Remaining difference |
| --- | --- | --- |
| Main interface | Final-era fallback white header with a combined Google Reader wordmark, red Subscribe button, subscription tree, reading toolbar, stream title and all three density choices | Font rasterization, vector branding, menus and dialogs need reference-image comparison; 100% pixel parity is not established |
| Streams | Home, All items, Starred, feed, folder, tag, search, Trends and Explore destinations | Home uses a local overview and Explore a four-feed starter catalog; these are not the original server-generated pages |
| Reading | Expanded/list modes, inline row opening, cursor navigation, read/unread, sticky Keep unread, stars, tags and private notes | Historical Google+ actions and original social interactions are not available |
| Article body | Selectable text, simple formatting, links, headings, lists, quotes, code and fetched images | Constrained HTML projection; no publisher CSS engine, embedded media players or complete HTML/table layout |
| Stream controls | All/unread, newest/oldest, previous/next, age-based mark-read, per-stream display preferences | No Google's magic ranking, translation or global subscription statistics |
| Subscriptions | RSS/Atom/JSON Feed, accessible-page discovery, refresh, rename, multiple folder labels, reorder/drag, filter and unsubscribe | No Google feed index, search-by-keyword directory, server crawler or historical archive |
| Folder management | Create/rename/delete folders, assign memberships, collapse/expand, move feeds | The complete original tree/context-menu command surface is not implemented |
| Search | Local text and field operators, quoted phrases and exclusions; scoped local searches | Searches saved articles only, without Google's historical corpus |
| Trends | Local totals, 30-day reading chart and per-feed totals/read percentages | Original hour/day charts, subscriber-based statistics and inactive/obscure subscription analytics are not fully reproduced |
| Import/export | OPML merge/export with multiple memberships; validated complete JSON backup/restore; recent-change undo | Google Takeout social JSON and account recovery are not supported; undo is session-only |
| Preferences | Density, text size, start page, automatic read tracking, bulk-read confirmation, all/updated subscriptions, proxy | No localization, complete historical settings tabs or custom Send to services |
| Keyboard | Core article/stream/share/view keys, timed `g` prefix, subscription/tag jump dialogs, `g Shift+t` Trends, text-size keys and input exclusions | Original folder-tree cursor/toggle semantics are incomplete; subscription traversal opens feeds directly |
| Browser integration | History, downloads/uploads, clipboard, supported native sharing, email drafts, sanitized print document | Availability follows browser permissions and platform capabilities; no account, social or server email delivery |
| Browser persistence | IndexedDB transactions with generation checks; localStorage fallback; recovery handling; offline application snapshot | No cross-device synchronization; fallback storage lacks transactional multi-tab conflict protection |
| Desktop integration | Shared UI, native pickers/clipboard/launcher, asynchronous curl, coalesced atomic file writes | Requires curl; feed decoder currently UTF-8-only; share copies text, print opens a text preview, and one process should use a data directory |
| Narrow screen | Overlay subscriptions, compact toolbar and responsive article widths | A new adaptation of the desktop design; not a reproduction of Google's separate original mobile UI |

The app is an independent local reader built from the historical design. It must not be advertised as an operational replacement for every retired Google backend, nor as a fully validated pixel-perfect replica.

## Architecture contract

| Layer | Responsibility |
|---|---|
| `reader-domain` | Feed/article/subscription models, RSS and Atom parsing, safe article representation, OPML and portable documents |
| `reader-state` | MVVM state, commands, reactive projections, filtering, sorting, selection and read/star/tag transitions |
| `reader-controls` | Material-based Reader resources, reusable templated controls and state visuals |
| `reader-ui` | Small RSX view modules, navigation, reading layouts, dialogs and command bindings |
| `reader-fonts` | Shared font registration, browser font files and optional native embedding |
| `reader-browser` | FerroUI browser lifetime, browser persistence/network/storage integration and static host |
| `reader-desktop` | Native lifetime using the same controls, view models and domain behavior |

UI state belongs in view models. Reusable controls render properties and commands; they do not own feed retrieval or persistence. Large streams must use bounded projections and virtualization where supported. Refresh merges article identities while preserving the user's read, star and tag state. Synchronous paint/input paths must not perform network or storage work.

## Behavior acceptance matrix

| Area | Required behavior and important edge cases |
|---|---|
| Home | Useful overview derived from current subscriptions; links navigate to real streams |
| Streams | All items, individual feed, folder, starred and tag filters; accurate unread counts |
| Selection | Stable article identity through read-state changes, refresh, sorting and collection updates |
| Expanded reading | Readable title, source, author, date and body; original-link action; active article distinction |
| List reading | Unread bold/read regular rows; in-place expansion; selection distinct from expansion |
| Automatic read state | Opening/reading updates counts; Keep unread is not immediately reversed by scroll tracking |
| Star and tags | Immediate, persistent changes; tag views update; a star does not imply read |
| Bulk read | Current stream only; all/day/week/two-week age cutoffs; confirmation when configured |
| Sort and filter | Newest/oldest, all/unread; deterministic stable order for equal dates |
| Search | Titles and article text; case-insensitive input; meaningful empty results; scope respected |
| Subscribe | RSS/Atom URL parsing, duplicate handling, loading/error state and a real resulting feed |
| Refresh | Bounded concurrent work; no duplicate articles; no navigation on background completion; results cannot recreate an unsubscribed source |
| Folders | Create, rename, membership changes, collapse/expand, delete and move/reorder semantics |
| Manage subscriptions | Searchable list, rename, unsubscribe, folder changes and coherent active selection |
| Import/export | OPML preserves folder membership; malformed input is reported without erasing saved state |
| Backup | Portable app document preserves read/star/tag/preferences state; import validates its schema |
| Trends | Statistics computed from recorded events, with useful empty states; no fabricated activity |
| Preferences | Density, reading mode and relevant settings persist across reloads |
| Keyboard | Reference map, input focus exclusions, multi-key timeout and dialog Escape handling |
| Browser navigation | Back/forward restores feed/folder destination; selection does not corrupt stream history |
| Sharing | Visible user action opens a supported sharing destination or copies a link; no silent send |
| Persistence | Reload retains subscriptions and article annotations; storage failure is visible and recoverable |
| Narrow layout | Reading remains usable, controls stay reachable and popups remain inside the viewport |
| Offline | Previously saved local content remains readable; network failures preserve saved data |

## Visual acceptance

Compare the same viewport, device scale, font, density, stream and content against the preserved client. Verify all three density modes and both reading modes. Include unread, read, selected, hover, pressed, focused and disabled states; expanded articles; dropdown menus; dialogs; empty streams; long titles; long source names; scrolling and keyboard focus.

The key layout is a fixed-width subscription tree beside a flexible reading pane, a compact reading toolbar, thin separators and restrained red selection accents. The Material base must not leak unrelated shapes, elevation, spacing or typography into Reader controls. The source measurements are recorded in [reference.md](reference.md#layout-measurements).

The original desktop client had a separate mobile interface, not a modern single responsive screen. The narrow adaptation keeps the desktop information model and reading commands accessible. It should be described as an adaptation unless checked separately against the original mobile client.

## Original service boundaries

Google shut down Reader in July 2013. This application cannot reconnect to old Google Reader accounts or recover data deleted by Google. Importable exports supplied by the user are distinct from account recovery. See the [official Reader archive](https://googlereader.blogspot.com/).

| Original capability | Boundary for this app |
|---|---|
| Google account synchronization | Local browser/native state is not cross-device account sync; a separately configured service is needed for synchronization |
| Google's feed crawler and historical archive | A browser can fetch feeds allowed by the publisher's CORS policy; general crawling/history needs a server |
| Global feed-directory search | A local directory or local subscription search is not Google's former global index |
| Arbitrary website-to-feed discovery | Requires an accessible page/feed or a configured proxy; a URL must not be reported subscribed when retrieval failed |
| Magic ranking | A disclosed local ranking can be useful, but cannot reproduce Google's unpublished global/personal ranking |
| Recommended/popular items | A curated catalog or local recommendations must be identified as such; they are not the retired Google service |
| Subscriber counts and global statistics | Show locally measured values only; historical Google subscriber counts are unavailable |
| Translation | Needs a functioning, explicitly chosen translation service; no fake success state |
| Google+ sharing | Retired service; supported modern destinations and copy-link are separate adaptations |
| Native Reader social network | Following, shared-item comments and likes were removed in the 2011 redesign; they are outside the default final-era interface |
| Server email delivery | `mailto:` opens the user's email application; it does not send mail or prove delivery |
| Old embedded media | Publishers may have removed assets or stopped allowing embedding; preserve article text and original links |

GitHub Pages serves static files and cannot execute a server-side RSS proxy. A configurable proxy must be chosen by the user; feed URLs and content sent through it are visible to its operator. Direct feed requests should be attempted where supported, with clear retrieval errors and without changing existing subscriptions on failure.

## Data and interaction invariants

1. Article identity is stable across refreshes. Updating content must retain read/star/tag annotations.
2. OPML imports merge subscriptions and folder membership; they do not replace the whole library without an explicit replace action.
3. Unsubscribing one source must not remove unrelated sources or mutate another feed's annotations.
4. Marking a filtered stream read must affect only the intended items and age range.
5. Refreshing a stream after navigating away must not replace the active stream with stale content.
6. Rendered publisher HTML is untrusted. Scripts, event handlers and unsafe URL schemes must not become executable application UI.
7. External links and share destinations open through a deliberate user gesture.
8. Persistence errors must not be represented as a successful save or overwrite a known-good document with partial state.
9. Shortcuts never intercept normal typing or text composition.
10. Cache updates must not claim offline availability before all required application assets are stored.

## Build and release verification

The lean Pages workflow runs domain/state and focused JavaScript host tests, builds one production browser module using the pinned FerroUI toolchain and deploys that output from `main`. Framework dependencies and browser tools are cached; it does not launch a browser screenshot suite on every commit.

An explicit `[verify-ui]` commit marker or manual `verify_ui: true` dispatch adds the rendered-browser gate and the `reader-ui-verification` evidence artifact. The final eleven-check pass, new-document offline proof and remaining coverage limits are recorded in [verification.md](verification.md).

Before claiming a release has visual and interaction parity, check the built app in a real browser at desktop and narrow widths: add/import feeds, navigate, open articles, star/tag/mark unread, search, change density/mode, reload, export and exercise keyboard commands. Check actual screenshots against the reference and record any unsupported original-service features explicitly. A first cold WebAssembly build is heavier than a cached rebuild; the CI workflow keeps this compilation out of repeated setup work.
