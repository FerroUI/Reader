# Reader fidelity and behavior contract

Reader targets the final desktop Google Reader experience. [reference.md](reference.md) records the original dimensions, colors, shortcuts and source URLs. This document defines the behavior to verify and the boundaries of a static browser application. A completed build alone does not certify every row or pixel; release claims must reflect observed behavior.

## Implemented coverage and remaining differences

The following describes the checked-in implementation. It is a source-level feature inventory, not certification of every interaction on every browser or operating system.

| Area | Present implementation | Remaining difference |
| --- | --- | --- |
| Main interface | Final-era fallback white header with a combined Google Reader wordmark, red Subscribe button, subscription tree, reading toolbar, stream title and all three density choices; menu and dialog styling follows preserved source measurements | Font rasterization, vector branding and visual states still need complete reference-image comparison. Bundled Liberation fonts do not cover every script: Japanese glyphs remain missing in the Unicode destination fixture, even though its text and URL encoding are preserved. 100% pixel parity is not established |
| Streams | Home, All items, Starred, feed, folder, tag, search, Trends and Explore destinations; Home stories open their source feed; unread-empty recovery preserves the current stream | Home uses a local overview and Explore a six-feed starter catalog; these are not the original server-generated pages |
| Reading | Expanded/list modes with the compact headline retained above flush list expansion; title/snippet share a shaped line; density-specific right insets and a current-item blue left edge; cursor navigation, read/unread, stars, private notes and navigable tags; automatic/manual preferences select the original Keep unread/Mark as read checkbox contract | Historical Google+ actions and original social interactions are not available; the explicit kept-unread tracking lock is session-local |
| Article body | Selectable text, inline emphasis/links, headings, nested lists, quotes, horizontally scrollable code, tables and dimension-aware images; read/current foregrounds update existing native paragraphs and hyperlinks; explicit publisher extraction with feed/full switching, cancellation, retry and offline text cache | Full-article extraction is an extension to the original feed-reading model. No publisher CSS engine, embedded media players, JavaScript execution or access to subscription-only content; media retains original links |
| Reading scale and width | Saved global body text size and sidebar visibility controls | Original magnification targeted the current article/cursor at 100–150% in ten-point steps; the app changes all body text within 11–24 px. Original hidden-sidebar bodies could fill the pane; the app retains its 650 px cap. [Exact source rules](reference.md#verified-magnification-and-hidden-sidebar-differences) |
| Scrolling | Native ListBox backed by TreeDataGrid's presenter with per-article measured offsets, bounded recycling and anchor preservation for nonuniform bodies | Unseen row sizes remain estimates until measured; row projection and sorting are proportional to stream length. See architecture and executed verification for the precise guarantees |
| Stream controls | All/unread, newest/oldest, previous/next, age-based mark-read, per-stream display preferences; configured bulk confirmation above 50 unread items, retained age cutoff and persistent opt-out | No Google's magic ranking, translation or global subscription statistics |
| Subscriptions | RSS/Atom/JSON Feed, bare-address normalization, accessible-page discovery, refresh, rename, multiple folder labels, reorder/drag, filter and unsubscribe; browser Automatic mode adds an rss2json fallback for eligible public RSS/Atom URLs | The public provider may cache or limit items and impose quotas; JSON Feed needs direct access or a custom raw proxy. No Google feed index, search-by-keyword directory, server crawler or historical archive |
| Folder management | Create/rename/delete folders, assign memberships, collapse/expand, move feeds; keyboard folder toggling preserves the cursor's parent occurrence | The complete original tree/context-menu command surface, folder ordering and independent per-folder feed orders are not implemented |
| Search | Local text and field operators, quoted phrases and exclusions; scoped local searches with query-specific empty feedback | Searches saved articles only, without Google's historical corpus |
| Trends | Local totals, 30-day reading chart and per-feed totals/read percentages | Original hour/day charts, subscriber-based statistics and inactive/obscure subscription analytics are not fully reproduced |
| Import/export | OPML merge/export with multiple memberships; validated complete JSON backup/restore; recent-change undo | Google Takeout social JSON and account recovery are not supported; undo is session-only |
| Preferences | Density, text size, start page, automatic read tracking, bulk-read confirmation, all/updated subscriptions; Automatic, Direct only and validated Custom proxy modes; custom Send to destination management | No localization or complete historical settings tabs/layout |
| Send to | Configured HTTPS destinations with encoded `${url}`, `${title}` and `${source}` values; create/edit/remove and enable/disable; footer dropdown and Shift+T use the same destinations; settings persist in backups | Original built-in service catalog, service icons, POST destinations, URL shortening and other legacy placeholders are absent. Shift+T opens configuration when no destination is enabled, a disclosed adaptation |
| Keyboard | Separate subscription-tree cursor with Shift+N/P and Shift+J/K aliases, Shift+O to open, Shift+X to toggle; timed `g` prefix, jump dialogs, `g Shift+t` Trends and input exclusions; Space/Shift+Space page tall articles with 50 px overlap or open neighbors, with forward unread-stream continuation; Shift+T opens Send to | No Popular items destination (`g p`); magnification differs as described above; Shift+S uses supported host sharing instead of the retired Google+ action |
| Dialogs and help | Subscribe and visible-article tag editors anchor to their desktop controls; other dialogs use the original white backdrop and compact heading; four-group shortcut help adapts to narrow viewports | Coarse-pointer/narrow screens and tag editing without a visible article anchor use centered dialogs; the original separate-window shortcut sheet and full historical settings surfaces are not reproduced |
| Browser integration | Stream history, Back-dismissed popups/dialogs/drawer, downloads/uploads, focus-preserving clipboard fallback, supported native sharing, email drafts, sanitized print document; article URL fragments retained | Availability follows browser permissions and platform capabilities; no account, social or server email delivery |
| Browser persistence | IndexedDB transactions with generation checks; localStorage fallback; recovery handling; offline application snapshot | No cross-device synchronization; fallback storage lacks transactional multi-tab conflict protection |
| Desktop integration | Shared UI, native pickers/clipboard/launcher, asynchronous curl, coalesced atomic file writes | Requires curl; feed decoder currently UTF-8-only; share copies text, print opens a text preview, and one process should use a data directory |
| Narrow screen | Overlay subscriptions, compact toolbar, responsive article widths, coarse-pointer targets, safe-area and visual-viewport layout, browser pinch magnification, native Search/Search and URL/Go keyboard hints | An adaptation of the desktop design; not a reproduction of Google's separate original mobile UI. DOM hints and synthetic keyboard viewport handling are tested; physical iOS keyboard presentation is not verified |

The app is an independent local reader built from the historical design. It must not be advertised as an operational replacement for every retired Google backend, nor as a fully validated pixel-perfect replica.

## Architecture contract

| Layer | Responsibility |
|---|---|
| `reader-domain` | Feed/article/subscription models, RSS and Atom parsing, safe article representation, OPML and portable documents |
| `reader-state` | MVVM state, commands, reactive projections, filtering, sorting, selection and read/star/tag transitions |
| `reader-controls` | Material-based Reader resources, reusable templated controls and state visuals |
| `reader-ui` | Small RSX view modules, navigation, reading layouts, dialogs and command bindings |
| `reader-content` | Bounded native content projection and optional HTML5/Readability publisher extraction |
| `reader-extract` | Small Web Worker WASM bridge using the same extractor as desktop |
| `reader-fonts` | Shared font registration, browser font files and optional native embedding |
| `reader-browser` | FerroUI browser lifetime, browser persistence/network/storage integration and static host |
| `reader-desktop` | Native lifetime using the same controls, view models and domain behavior |

UI state belongs in view models. Reusable controls render properties and commands; they do not own feed retrieval or persistence. Large streams must use bounded projections and virtualization where supported. Refresh merges article identities while preserving the user's read, star and tag state. Synchronous paint/input paths must not perform network or storage work.

## Behavior acceptance matrix

| Area | Required behavior and important edge cases |
|---|---|
| Home | Useful overview derived from current subscriptions; clicking the title, date or excerpt opens the article in its source feed |
| Streams | All items, individual feed, folder, starred and tag filters; accurate unread counts |
| Empty streams | Reading-list, starred and named-stream messages distinguish empty from unread-empty; View all items clears only the current filter; search feedback retains the query and scope |
| Selection | Stable article identity through read-state changes, refresh, sorting and collection updates |
| Expanded reading | Readable title, source, author, date and body; original-link action; density-specific geometry; the current card changes its left edge while other frame edges stay gray |
| List reading | Unread bold/read regular titles and a single-line ` - ` snippet; source/date use the reference size; expansion retains the compact headline and appends the full title/body/footer without card margins; the expanded headline uses read/unread color, not collapsed-selection yellow; selection remains distinct from expansion |
| Article palette | Unread/current text is black with blue links; inactive read text is `#222222` with `#444444` links and `#666666` source metadata. Table text follows the same rule. Recoloring retains native paragraph/link identity, selection and hyperlink behavior |
| Read-state checkbox | Automatic mode always says Keep unread and checks the explicit kept-unread state; manual mode always says Mark as read and checks the read flag. Checking/unchecking updates the paired read/kept-unread states; scroll tracking does not immediately reverse a kept-unread action |
| Star and tags | Immediate, persistent changes; a star does not imply read; Add tags becomes Edit tags with individual tag-navigation links and no duplicate label |
| Bulk read | Current stream only; all/day/week/two-week age cutoffs; configured confirmation above 50 unread items in the whole stream, including dated actions; Cancel changes nothing; confirmation preserves the cutoff and can persist Don't ask me again |
| Sort and filter | Newest/oldest, all/unread; deterministic stable order for equal dates |
| Search | Titles and article text; case-insensitive input; meaningful empty results; scope respected |
| Subscribe | RSS/Atom/JSON Feed parsing, normalized website addresses, duplicate handling, loading/error state and a real resulting feed; desktop popup anchored to Subscribe with Enter, Escape and outside dismissal |
| Feed connection | Automatic tries direct access before the saved custom proxy or rss2json; Direct never uses a relay; Custom requires a valid raw-response template. Preserve publisher identities and saved articles, distinguish offline/HTTP/timeout/configuration failures, and never invent a subscription after a failed request |
| Refresh | Bounded concurrent work; no duplicate articles; no navigation on background completion; results cannot recreate an unsubscribed source |
| Folders | Create, rename, membership changes, collapse/expand, delete and move/reorder semantics |
| Manage subscriptions | Searchable list, rename, unsubscribe, folder changes and coherent active selection |
| Import/export | OPML preserves folder membership; malformed input is reported without erasing saved state |
| Backup | Portable app document preserves read/star/tag/preferences state; import validates its schema |
| Trends | Statistics computed from recorded events, with useful empty states; no fabricated activity |
| Preferences | Density, reading mode and relevant settings persist across reloads |
| Keyboard | Subscription cursor moves without opening or marking articles read; visible-tree order, no wrapping, duplicate feed occurrences and collapsed-parent recovery remain coherent; input focus exclusions, multi-key timeout and dialog Escape handling |
| Space navigation | Without a current item, Space opens the first and Shift+Space does nothing. Page within a tall current article until its relevant edge is visible, then open its neighbor. At the final bottom, Space opens the next unread tree occurrence without wrapping. Non-stream pages and Page Up/Page Down retain page scrolling |
| Popups | Desktop anchors follow native layout; form values survive presentation changes; active text composition is not remounted during resize; touch dialogs retain reachable actions |
| Browser navigation | Back/forward restores feed/folder destination; selection does not corrupt stream history |
| Send to | Only enabled saved destinations appear. Resolve `${source}` from the subscription display name, not its URL; encode substitutions once and preserve title/article fragments. Invalid drafts do not replace saved settings; a stale popup cannot send a replaced article or destination. Opening the menu alone performs no external action |
| Sharing | Send to and host sharing remain separate commands. A deliberate destination choice opens its configured address; Shift+S uses supported native sharing or a copy fallback; no silent send |
| Persistence | Reload retains subscriptions and article annotations; storage failure is visible and recoverable |
| Narrow layout | Reading remains usable, controls stay reachable and popups remain inside the viewport |
| Offline | Previously saved local content remains readable; network failures preserve saved data |

## Visual acceptance

Compare the same viewport, device scale, font, density, stream and content against the preserved client. Verify all three density modes and both reading modes. Include unread, read, selected, hover, pressed, focused and disabled states; expanded articles; dropdown menus; dialogs; empty streams; long titles; long source names; scrolling and keyboard focus.

The key layout is a fixed-width subscription tree beside a flexible reading pane, a compact reading toolbar, thin separators and restrained red selection accents. The Material base must not leak unrelated shapes, elevation, spacing or typography into Reader controls. The source measurements are recorded in [reference.md](reference.md#layout-measurements).

The current article refinement uses the [preserved list/card selectors and templates](reference.md#retained-list-headline-and-flush-expansion), [body/link palette](reference.md#body-and-hyperlink-palette), [read-state checkbox contract](reference.md#read-state-checkbox), [Space navigation](reference.md#space-and-shiftspace) and [Send to behavior](reference.md#configured-send-to-destinations). Source-level corrections and newly defined regression scenarios are not a completed comparison against every historical screenshot. The magnification and hidden-sidebar width differences above remain explicit visual gaps.

The original desktop client had a separate mobile interface. The narrow adaptation keeps the desktop information model and reading commands accessible. [The primary mobile sources](reference.md#original-mobile-reader-and-touch-evidence) confirm tap-to-expand and inline actions, and a later More menu; they do not establish left/right article swipe commands. Those gestures must not be attributed to Reader merely because third-party Reeder implemented them. Safe-area, keyboard-viewport and browser Back integration are explicit modern adaptations.

## Original service boundaries

Google shut down Reader in July 2013. This application cannot reconnect to old Google Reader accounts or recover data deleted by Google. Importable exports supplied by the user are distinct from account recovery. See the [official Reader archive](https://googlereader.blogspot.com/).

| Original capability | Boundary for this app |
|---|---|
| Google account synchronization | Local browser/native state is not cross-device account sync; a separately configured service is needed for synchronization |
| Google's feed crawler and historical archive | Direct CORS access, an optional trusted raw proxy and Automatic rss2json conversion fetch current feeds; provider limits apply and this is not Google's crawler or historical archive |
| Global feed-directory search | A local directory or local subscription search is not Google's former global index |
| Arbitrary website-to-feed discovery | Accessible HTML or a custom raw proxy supports alternate links, document bases and relative feeds; blocked HTML may require its actual RSS/Atom address. A URL must not be reported subscribed when retrieval failed |
| Magic ranking | A disclosed local ranking can be useful, but cannot reproduce Google's unpublished global/personal ranking |
| Recommended/popular items | A curated catalog or local recommendations must be identified as such; they are not the retired Google service |
| Subscriber counts and global statistics | Show locally measured values only; historical Google subscriber counts are unavailable |
| Translation | Needs a functioning, explicitly chosen translation service; no fake success state |
| Google+ sharing | Retired service; supported modern destinations and copy-link are separate adaptations |
| Native Reader social network | Following, shared-item comments and likes were removed in the 2011 redesign; they are outside the default final-era interface |
| Server email delivery | `mailto:` opens the user's email application; it does not send mail or prove delivery |
| Old embedded media | Publishers may have removed assets or stopped allowing embedding; preserve article text and original links |

GitHub Pages serves static files and cannot make publishers return CORS headers or execute an application-owned RSS proxy. Browser Automatic mode tries direct access, then an existing configured custom proxy or the named rss2json service for eligible public RSS/Atom feeds. The service receives feed URLs and has caching, item and quota limits. Direct only and custom raw-proxy modes remain available; the native host continues to fetch directly. [Feed connection](feed-connection.md) documents setup, private-feed handling, JSON Feed and discovery boundaries, errors and primary sources.

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

The lean Pages workflow runs domain/state/content, focused JavaScript host, pure gesture, tree-cursor, image/scroll geometry and native request-operation checks, builds the production UI and small extraction worker using the pinned FerroUI toolchain and deploys that output from `main`. Framework dependencies and browser tools are cached; it does not launch a browser screenshot suite on every commit.

An explicit `[verify-ui]` commit marker or manual `verify_ui: true` dispatch adds the rendered-browser gate and the `reader-ui-verification` evidence artifact. The suite includes source-feed Home navigation, tag links, empty-stream recovery, anchored quick add, subscription cursor behavior, shortcut help, dated mark-all confirmation and a publisher/relay fixture with real cross-origin blocking. Article-layout, native palette/selection-retention and variable-height Space-navigation scenarios cover this refinement; Send to template and command tests cover saved destination resolution and stale-menu rejection. Defined scenarios are not a passing-run claim: the executed revision, outcomes, new-document offline proof and remaining coverage limits are recorded separately in [verification.md](verification.md).

Before claiming a release has visual and interaction parity, check the built app in a real browser at desktop and narrow widths: add/import feeds, navigate, open articles, star/tag/mark unread, search, change density/mode, reload, export and exercise keyboard commands. Check actual screenshots against the reference and record any unsupported original-service features explicitly. A first cold WebAssembly build is heavier than a cached rebuild; the CI workflow keeps this compilation out of repeated setup work.
