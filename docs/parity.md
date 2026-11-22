# Reader fidelity and behavior contract

Reader targets the final desktop Google Reader experience. [reference.md](reference.md) records the original dimensions, colors, shortcuts and source URLs. This document defines the behavior to verify and the boundaries of a static browser application. A completed build alone does not certify every row or pixel; release claims must reflect observed behavior.

## Architecture contract

| Layer | Responsibility |
|---|---|
| `reader-domain` | Feed/article/subscription models, RSS and Atom parsing, safe article representation, OPML and portable documents |
| `reader-state` | MVVM state, commands, reactive projections, filtering, sorting, selection and read/star/tag transitions |
| `reader-controls` | Material-based Reader resources, reusable templated controls and state visuals |
| `reader-ui` | Small RSX view modules, navigation, reading layouts, dialogs and command bindings |
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
| Refresh | Bounded concurrent work; no duplicate articles; cancellation or stale-result rejection after navigation |
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

The lean Pages workflow runs domain/state tests, builds one production browser module using the pinned FerroUI toolchain and deploys that output from `main`. Framework dependencies and browser tools are cached; it does not launch a browser screenshot suite on every commit.

Before claiming a release has visual and interaction parity, check the built app in a real browser at desktop and narrow widths: add/import feeds, navigate, open articles, star/tag/mark unread, search, change density/mode, reload, export and exercise keyboard commands. Check actual screenshots against the reference and record any unsupported original-service features explicitly. A first cold WebAssembly build is heavier than a cached rebuild; the CI workflow keeps this compilation out of repeated setup work.
