# Google Reader reference specification

The visual reference is the final desktop Google Reader client from 2011–2013, using its light appearance. Research was checked on 22–23 November 2026. The Material theme supplies FerroUI's underlying control behavior; Reader resources and templates supply the historical appearance.

## Primary reference

[Mihai Parparita's preserved Reader](https://readerisdead.com/reader/view/) contains the final public client. In his [first-person account of its preservation](https://blog.persistent.info/2013/07/using-google-readers-reanimated-corpse.html), the former Reader engineer explains that he saved its public HTML, CSS, JavaScript and images immediately before shutdown and supplied a replacement read-only API. His [2018 time-capsule article](https://medium.com/@mihai/google-reader-a-time-capsule-from-5-years-ago-df5176aa08b6) describes the publicly hosted version. The [preservation tooling](https://github.com/mihaip/readerisdead) is available separately.

Reader's implementation in this repository is original Rust using FerroUI. The preserved client is reference material; its obfuscated JavaScript, CSS implementation and article archive are not application dependencies.

The following publicly served reference resources provide exact values:

- [Main stylesheet at the preserved source revision](https://github.com/mihaip/readerisdead/blob/0e35cf26e88f27e0a07432182757c1ce230f6936/zombie_reader/static/2139832829-en-scroll.css), also [served by the archive](https://readerisdead.com/reader/ui/2139832829-en-scroll.css).
- [Preserved final client behavior](https://github.com/mihaip/readerisdead/blob/0e35cf26e88f27e0a07432182757c1ce230f6936/zombie_reader/static/3564210627-en-scroll.js).
- [Comfortable density](https://readerisdead.com/reader/ui/2159580021-en-scroll_large_density.css).
- [Cozy density](https://readerisdead.com/reader/ui/1771541332-en-scroll_medium_density.css).
- [Compact density](https://readerisdead.com/reader/ui/2617741944-en-scroll_small_density.css).
- [Final keyboard-help content](https://readerisdead.com/reader/ui/2662170544-en-keyboard-help-banner?hl=en).
- [Printable keyboard help](https://readerisdead.com/reader/ui/4269787364-en-keyboard-help-tearoff?hl=en).

The preservation has an alternate top header without the Google+ notification bar. Parparita confirms that this fallback was supported by the original client. It is a valid reference configuration, not a new header invented for this application.

## Layout measurements

Values below are read from the preserved density stylesheets, in CSS pixels. Rendering should compare equivalent display scale, viewport size, density and article content.

| Property | Comfortable | Cozy | Compact |
|---|---:|---:|---:|
| Sidebar width | 264 | 248 | 236 |
| Main toolbar height | 57 | 57 | 43 |
| Sidebar title/Subscribe left inset | 44 | 28 | 16 |
| Main article region left inset | 5 | 5 | 5 |
| Main article region right inset | 33 | 21 | 10 |
| Collapsed article row height | 37 | 31 | 27 |
| Expanded article inner padding | 20 | 10 | 5 |
| Expanded article top/bottom margin | 20 / 40 | 10 / 20 | 5 / 10 |

The default density is Comfortable. The separate top search header is 60 pixels high. The sidebar collapse strip is 8 pixels wide. Sidebar entries use 21-pixel line heights; primary section headings use 30 pixels. Collapsed source labels occupy approximately 11 em, followed by the title/snippet and a right-aligned date. Expanded article titles and bodies normally have a maximum width of 650 pixels; the original body-width exception when the sidebar is hidden is recorded below.

The margin row transcribes the CSS declarations on `#entries.cards .entry`. Adjacent vertical block margins collapse in the reference: the native layout must reproduce their resulting gap rather than add both CSS values together. The density-specific headline heights exclude the containing entry's bottom separator. For an expanded list entry, the headline's content height becomes 36/30/26 pixels and gains a 1-pixel bottom rule, retaining its previous headline footprint. These are the `#entries.list .entry .collapsed` and `#entries.list #current-entry.expanded .collapsed` rules in each density stylesheet.

## Typography, colors and states

| Role | Reference |
|---|---|
| Typeface | Arial, sans-serif |
| Root/body typography | `html` and `body` each use `90%`, about 12.96 px with default browser settings |
| Sidebar | 13 px |
| Stream title | 14 px, bold, 20 px line height |
| Expanded article title | 140% of body size, approximately 18 px |
| Base foreground/background | `#000000` / `#ffffff` |
| Separators | `#ebebeb`; top search separator `#e5e5e5` |
| Article links and unread expanded titles | `#1155cc` |
| Read expanded title | `#444444` |
| Read expanded body | `#222222` |
| Metadata | `#666666` |
| Unread/read collapsed row | `#ffffff` / `#eeeeee` |
| Selected collapsed row | `#ffffcc`, 1 px blue left edge `#4d90f0` |
| Selected sidebar text | `#d14836` |
| Sidebar selection edge | 3 px `#dd4b39` |
| Sidebar hover | `#eeeeee` |
| Article action footer | `#fafafa`, top border `#ebebeb` |

Selected navigation uses a slim red edge and red text. Toolbar buttons are compact rectangles with a slight radius. There are no large rounded navigation pills or elevated dashboard cards in the reference.

Standard buttons use a `#f5f5f5` to `#f1f1f1` gradient, `#dcdcdc` border and `#444444` text. Subscribe uses uppercase white text with a `#dd4b39` to `#d14836` gradient, minimum width 80 pixels and height 27 pixels. Search uses a `#4d90fe` to `#4787ed` gradient and `#3079ed` border. The fallback header has a 250-pixel search field and a 130-pixel scope selector.

In list view, unread titles are bold and read titles regular. Selection and read state are independent. In expanded view the currently selected article retains blue links even after becoming read. Article footers expose star, the configured read-state checkbox, email, tags and enabled send-to destinations with approximately 16 pixels between controls.

### Retained list headline and flush expansion

The [saved client](https://github.com/mihaip/readerisdead/blob/0e35cf26e88f27e0a07432182757c1ce230f6936/zombie_reader/static/3564210627-en-scroll.js) creates the compact headline in `MH.Ef`. `MH.expand` appends the `tH` article container and `iw` action footer below it; `MH.collapse` removes those appended sections. Expansion keeps the headline and its star, source, title/snippet, date and original-link target. The larger title in `tH` repeats the headline title intentionally. Its star placeholder remains empty in list mode; `xH.pm` fills that placeholder for card mode.

The following rules come from the [preserved main stylesheet](https://github.com/mihaip/readerisdead/blob/0e35cf26e88f27e0a07432182757c1ce230f6936/zombie_reader/static/2139832829-en-scroll.css):

| Detail | Original rule and meaning |
|---|---|
| Flush list entries | `#entries.list .entry` has zero margin/padding and a 1 px `#ccc` bottom separator. Expanding a list item does not introduce card margins. |
| Headline appearance | `.read .collapsed` is `#eee`; the current collapsed headline is `#ffc`. `#current-entry.expanded .collapsed` returns to white, or `#eee` when read, and adds a `#ebebeb` bottom separator. |
| Headline typography | Source/date use the inherited approximately 13 px size. `MH.Ef` separates title and snippet with the literal ` - `. Title and snippet share a single line. |
| Expanded body inset | `#entries.list .entry .entry-container` has a 1 px left edge, `.5em` top padding and `1em` left padding. The duplicated full date is hidden by its `.entry-date` rule. |
| List footer | `.entry-actions` has a `#ebebeb` top rule, 1 px left edge and padding `6px 0 6px 7px`. The current entry's body/footer left edges become `#4d90f0`. |
| Card selection | `.card` retains its 1 px `#ddd` frame; `#current-entry .card` changes only the left edge to `#4d90f0`. The reference also has a subtle `0 0 4px #e3e5eb` shadow. |

The native implementation follows this retained-headline structure in `article/headline.rs` and `article.rs`, with card density geometry in `article/layout.rs`. Touch target expansion remains a deliberate narrow/coarse-pointer adaptation.

The list footer has no explicit height or line-height in the source. Its inherited Arial size is approximately 13 px with `line-height: normal`; the inline action rules add 1 px vertical paint padding without increasing that line box. The separate `.card-common .card-actions` wrapper has an explicit 30 px height plus its top border. Reader's desktop list actions use 17 px native controls, accommodating a 15 px text line and two focus-border pixels, producing a 30 px footer with the original vertical padding. This is a native font-metric approximation; phone actions retain 44 px targets.

### Body and hyperlink palette

The [main stylesheet](https://github.com/mihaip/readerisdead/blob/0e35cf26e88f27e0a07432182757c1ce230f6936/zombie_reader/static/2139832829-en-scroll.css) defines body link color under `.entry-container .entry-body a`, its `.read` override and the more-specific `#current-entry` override:

| Article state | Body | Body links | Source link |
|---|---|---|---|
| Unread, current or inactive | `#000000` | `#1155cc` | `#1155cc` |
| Read and current | `#000000` | `#1155cc` | `#1155cc` |
| Read and inactive | `#222222` | `#444444` | `#666666` |

Title and source rules remove underlines explicitly; body links retain the normal anchor underline. In the native implementation, paragraph foreground changes update the existing `Hyperlink` objects, including links in table cells. Appearance changes must preserve native paragraph identity, link interaction and the user's text selection; rebuilding the rich document solely to recolor it is not part of the behavior contract.

## Navigation and menus

The final client provides Home, All items, Starred items, Trends and Browse for stuff; an Explore section containing recommendations; and a Subscriptions tree. Unread count display is capped at `1000+`. Feed and folder selection participate in browser history. These behaviors are described in the [official search release](https://googlereader.blogspot.com/2007_09_01_archive.html).

The reading toolbar contains refresh, an All items/New items filter, a split Mark all as read button, feed or folder settings, previous/next controls and expanded/list view switches. A gear menu selects Comfortable, Cozy or Compact and opens Reader settings/help.

The Mark all as read dropdown offers all items, items older than a day, older than a week and older than two weeks. The [official feature announcement](https://googlereader.blogspot.com/2009/08/flurry-of-features-for-feed-readers.html) explains the age-based choices and configurable send-to destinations.

Feed settings include newest/oldest/magic sorting, rename, unsubscribe, translation, details/statistics and folder membership. Folder settings include rename/delete and unsubscribe from all. Subscription-section settings include alphabetical/manual ordering, all/updated visibility, favicon/default icon preference, add and manage subscriptions.

The preserved generic menu-button implementation (`Ou`, using `Ku(..., 5)`) anchors the menu at the control's bottom-start edge. Native toolbar dropdowns now supply an explicit local placement rectangle so that FerroUI preserves this alignment and constrains the popup at viewport edges. Up/Down opening and native radio semantics preserve keyboard selection and prevent choosing an already-selected preference from clearing its checkmark.

### Subscription tree ordering

The [official September 2008 announcement](https://googlereader.blogspot.com/2008/09/friends-everywhere-and-other-friendly.html) introduced a choice between alphabetical subscription ordering and drag-and-drop ordering. The [saved final client's](https://github.com/mihaip/readerisdead/blob/0e35cf26e88f27e0a07432182757c1ce230f6936/zombie_reader/static/3564210627-en-scroll.js) `lv.zp` presents **Sort alphabetically** and **Sort by drag and drop** as exclusive choices; `lv.KA` saves the preference and changes the tree's mode. It is a persistent projection choice, not a command that discards manual positions.

| Contract | Preserved-client evidence |
|---|---|
| Independent manual positions | `gA`/`hA` read ordering per parent stream, `jA` saves its list and `kA` inserts a node relative to an anchor. A feed's occurrence inside one folder need not share another folder's position. |
| Alphabetical projection | `Nv.lq` builds folder groups first, followed by unfiled feeds and then standalone tags. `fG.Hd` applies the saved per-parent order only in manual mode. |
| Cross-folder movement | `Nv.Ar` creates the membership delta `sd:[destination]`, `Rd:[sourceParent]`: add the destination label and remove only the dragged occurrence's source label. Other labels survive. |
| Root movement | `Nv.Ar` adds a destination label for root-to-folder and removes the source label for folder-to-root. A feed with another label therefore remains in that other folder. Folder nodes do not nest. |
| Alphabetical-mode dragging | `Nv.Ar` gates relative placement through the manual-mode flag while still allowing cross-parent membership changes. Switching modes retains the saved manual lists. |

The native tree follows these occurrence and mode contracts. **Move up**, **Move down** and a folder's one-shot **Sort subscriptions alphabetically** provide explicit menu access to manual operations. These are usability adaptations, as are touch targets. Existing Reader backups without order metadata retain their previous unfiled-first/document-order projection until the user changes it; alphabetical mode uses the original folder-first grouping. JSON backup preserves the mode and manual lists. OPML exports the current projection, but its subscription/membership import does not reconstruct the independent ordering.

### Read-state checkbox

The [saved client's](https://github.com/mihaip/readerisdead/blob/0e35cf26e88f27e0a07432182757c1ce230f6936/zombie_reader/static/3564210627-en-scroll.js) `aw` chooses `it` when automatic read tracking is enabled and `ht` otherwise. `ct` and `bt` define their respective labels and state transitions; `Gp` names the kept-unread state and `Fp` the read state.

| Preference | Constant checkbox label | Checked meaning | Checking / unchecking |
|---|---|---|---|
| Automatic read tracking | Keep unread | Explicitly kept unread, not merely an unread article | Checking adds kept-unread and removes read; unchecking adds read and removes kept-unread. |
| Manual read tracking | Mark as read | Article is read | Checking adds read and removes kept-unread; unchecking adds kept-unread and removes read. |

The current native footer uses these two checkbox contracts. Its label does not switch between Keep unread and Mark as read solely because the article's read flag changes.

### Configured Send to destinations

The [official August 2009 announcement](https://googlereader.blogspot.com/2009/08/flurry-of-features-for-feed-readers.html) introduced configurable destinations. The [saved client's](https://github.com/mihaip/readerisdead/blob/0e35cf26e88f27e0a07432182757c1ce230f6936/zombie_reader/static/3564210627-en-scroll.js) `nE` renders the Send to label with a dropdown arrow. `IG` hides that footer control when there are no enabled destinations; `MG` and `FG` combine selected built-in services with configured custom links. `ix.fA` and `zx` connect Shift+T to this menu.

The `nE` toggle uses `Cu` with the control's bottom-left anchor (`ql = 1`) aligned to the menu's top-left (`tl = 0`). Reader supplies an explicit native placement rectangle for footer and keyboard opening, preserving that alignment and allowing the popup positioner to constrain it to a narrow viewport.

`LG` prepares template values and `KG` substitutes them. The current app implements three original GET-template values:

| Placeholder | Meaning |
|---|---|
| `${url}` | Original article address. |
| `${title}` | Article title, with the original HTML text decoded. |
| `${source}` | Subscription display name, including the user's rename; this is the title produced by `hw`, not the source site's URL. |

Original GET destinations URL-encode substituted values with `encodeURIComponent`; the current resolver applies that encoding to each value and expands templates once. Reader settings allow creating, editing, enabling, disabling and removing custom destinations. Enabled destinations appear in the article menu and Shift+T. With none enabled, the footer action is absent and Shift+T opens configuration, an explicit usability adaptation.

Current destinations require HTTPS and a fixed host, and use URL navigation after an explicit selection. The original service catalog, service icons, POST forms, URL shortener, and additional body/Google-identifier/escaped-title placeholders are not implemented. Exact historical settings-page layout and wording are not established by the preserved client templates used in this review.

## Secondary surfaces and interaction details

The following contracts were checked against the preserved source in the additional 22 November 2026 review. They describe the historical reference; they are not claims that every surface has already been reproduced. CSS selectors and JavaScript function names identify the exact evidence in the [stylesheet](https://readerisdead.com/reader/ui/2139832829-en-scroll.css) and [saved client](https://github.com/mihaip/readerisdead/blob/0e35cf26e88f27e0a07432182757c1ce230f6936/zombie_reader/static/3564210627-en-scroll.js).

| Surface | Original geometry and appearance | Evidence |
|---|---|---|
| Dropdown | White, 1 px translucent black border, 6 px vertical outer padding, shadow `0 2px 4px rgba(0,0,0,.2)` | `.goog-menu` |
| Menu item | 13 px text; 6 px vertical padding; hover `#eee`; disabled text `#ccc` | `.goog-menuitem`, hover/disabled rules |
| Menu separator | 1 px `#ebebeb` rule, 6 px space above and below | `.goog-menuseparator` |
| Standard dialog | White, 1 px border, 42 px horizontal / 30 px vertical padding, shadow `0 4px 16px rgba(0,0,0,.2)` | `.modal-dialog` |
| Dialog heading/body | Heading 16 px normal, 24 px line height, 16 px bottom gap; body line height 1.4; 16 px gap before buttons | `.modal-dialog-title`, content/buttons rules |
| Dialog backdrop | White at **50% opacity at runtime**; the stylesheet's 75% is overridden | `xy` prototype `mj=0.5`, `Ey`, dialog render; `.modal-dialog-bg` |
| Empty stream | Centered heading inside a white box; 2 px `#ccc` border; 5 px radius; margin `1em`; padding `1.5em 2em 3em` | `#no-entries-msg`, `kE.Xt` |
| Subscribe popup | Anchored below Subscribe; width 300 px; `#f7f7f7`; 1 px `#d0d0d0`; form inset 8 px | `#quick-add-bubble-holder`, `#quick-add-form`, `Iv.kb` |
| Tag editor | Anchored at the article action; width 260 px plus 8 px padding; `#f7f7f7`; 1 px `#d0d0d0` | `.tags-edit`, `oE.cA`, `VG` |

An empty unread stream offers **View all items** by changing the current stream's filter. It preserves the feed, folder or tag rather than jumping to All items. The message distinguishes an empty reading list, no starred articles, a named stream with no articles, and a stream with no unread articles. A search with no results identifies its query. These distinctions come from `as` and `kE.Xt`; the obsolete Google Blog Search link is not a useful modern recovery action.

Opening a Home article resolves its containing feed stream and then selects that article (`zv.fx`). The [preservation README](https://github.com/mihaip/readerisdead#zombie_reader) explicitly says that its Home page was repurposed for archive statistics. Archive-specific Home contents are therefore not evidence for the original product's content.

Mark-all uses the current stream's unread count. The client asks for confirmation only above 50 unread items when confirmation is enabled; the dialog includes the affected stream, count or age condition, and a preference to stop asking (`Ts`, `kE.xs`, `kE.iy`). The older-than choices use that same confirmation path. The age cutoff remains part of the pending operation through confirmation.

The Subscribe and tag editors support focused input, Enter to save, Escape to close, and dismissal outside their popup. An article without tags has an add action; an article with tags exposes an edit action followed by individually navigable tag names (`OG`, `PG`). Preserve these relationships when adapting the controls to narrow screens; historical small close glyphs do not define a suitable touch target.

## Keyboard map

This table transcribes behavior from the preserved final keyboard-help resource. A space-separated sequence means successive keys, not a simultaneous chord. It is the historical map, not a list of every currently implemented destination: the app has no Popular items destination (`g p`), and its in-app help omits that destination. Shift+T opens configured Send to destinations; host sharing remains a separate adaptation on Shift+S. Current coverage and adaptations are recorded in [parity.md](parity.md#implemented-coverage-and-remaining-differences).

| Shortcut | Action |
|---|---|
| `j` / `k` | Next / previous article |
| Space / Shift+Space | Next / previous article or page |
| `n` / `p` | Scan down/up without opening in list view |
| Shift+`n` / Shift+`p` | Next / previous navigation subscription |
| Shift+`x` | Expand/collapse the selected folder |
| Shift+`o` | Open the selected subscription or folder |
| `r` | Refresh |
| `f` | Toggle full-screen reading |
| `u` | Hide/show the sidebar |
| `1` / `2` | Expanded / list view |
| `/` | Focus search |
| `a` | Subscribe |
| `=` / `-` | Increase / decrease reading magnification |
| `g h` | Home |
| `g a` | All items |
| `g s` | Starred items |
| `g u` | Open subscription selector |
| `g t` | Open tag selector |
| `g Shift+t` | Trends |
| `g d` | Discovery |
| `g e` | Explore |
| `g p` | Popular items |
| `s` | Toggle star |
| `t` | Edit article tags |
| `e` | Email article |
| Shift+`s` | Share article |
| `v` | Open original |
| `o` / Enter | Expand/collapse the selected article in list view |
| `m` | Toggle read/unread |
| Shift+`a` | Mark the current stream read |
| Shift+`t` | Open send-to menu |
| `?` | Keyboard help |

Shortcut handlers must respect text input, composition, open dialogs and platform modifier keys. A partial `g` sequence expires and unrelated keys cancel it.

### Space and Shift+Space

The [saved client's](https://github.com/mihaip/readerisdead/blob/0e35cf26e88f27e0a07432182757c1ce230f6936/zombie_reader/static/3564210627-en-scroll.js) `xE` binds Space/Shift+Space to `XG.iu`, while Page Down/Page Up bind directly to `XG.Le`. These are distinct commands:

| Situation | Space | Shift+Space |
|---|---|---|
| Stream has no current article | Open the first article. | Leave the cursor unchanged. |
| Current article's relevant edge is outside the viewport | Page forward until its bottom is visible. | Page backward until its top is visible. |
| Relevant edge is visible | Open the next article. | Open the previous article; do not wrap before the first. |
| Last article's bottom is visible | `kE.ju` advances to the next unread subscription-tree occurrence through `iG`, without wrapping. | Apply the ordinary previous-article/top rule. |
| Home or another non-stream page | Page the active page. | Page the active page backward. |

`XG.Le` moves by the viewport height minus 50 pixels, preserving a reading overlap. The native implementation uses measured or indexed variable-height article geometry, keeps list scans distinct from opening, and uses the subscription cursor's unread traversal at the stream boundary. Page Up/Page Down continue to perform viewport paging.

### Verified magnification and hidden-sidebar behavior

The original `wE` binds magnification changes in ten-point steps; `XG.Wp` clamps them to 100–150%. The [stylesheet's](https://github.com/mihaip/readerisdead/blob/0e35cf26e88f27e0a07432182757c1ce230f6936/zombie_reader/static/2139832829-en-scroll.css) `.magnification110` through `.magnification150` rules target `#current-entry` and `.cursor`, not all article bodies. The native `+`/`=`/`-` commands now keep a transient 100–150% scale for the active article. The saved global base text size remains a separate preference. Scaling updates retained native text properties rather than rebuilding the document; density-defined headline heights remain fixed.

The same stylesheet's `.lhn-hidden #entries .entry-body` rule raises the body's maximum width to 100% when the sidebar is hidden. The native body now follows that width policy when the desktop sidebar is hidden or focus mode is active, while titles keep their own width constraint. The phone layout continues to use its responsive pane width.

### Subscription cursor and help overlay

The [saved client](https://github.com/mihaip/readerisdead/blob/0e35cf26e88f27e0a07432182757c1ce230f6936/zombie_reader/static/3564210627-en-scroll.js) separates the subscription keyboard cursor from the opened stream (`RF`, `ZF`, `iG`, `VF`). Shift+N/P traverses the Subscriptions tree in visible depth-first order, including folders, feed occurrences and standalone tags. It excludes Home, Starred and Explore. Collapsed descendants are skipped; the updated-only setting skips entries with no unread items. The cursor is gray `#ddd`; the open stream keeps its red selection treatment.

Traversal does not wrap. Without a cursor, it starts relative to the opened tree occurrence. With neither cursor nor opened occurrence, Shift+N chooses the first eligible entry and Shift+P does nothing. Shift+O opens the cursor entry and retains it; opening an already open entry refreshes it. Shift+X toggles the cursor's folder or, from a child feed, the nearest ancestor with children. Rebuilding the visible tree preserves the same occurrence or its closest surviving ancestor. Shift+J/K are aliases of Shift+N/P in `wE`.

The [keyboard-help template](https://readerisdead.com/reader/ui/2662170544-en-keyboard-help-banner?hl=en) has four groups arranged as a two-by-two table: Navigation/Application, then Jumping/Acting on items. Its [CSS](https://readerisdead.com/reader/ui/2139832829-en-scroll.css) places the banner at 5% from the top and left, width 90%, with a black 80%-opaque background, 10 px radius, white text and `#dd0` group headings. Keys are bold and right-aligned; descriptions are normal weight. It includes separate-window and close links and scrolls when the viewport is too short (`VC`). The app retains the four groups, close actions and scrolling, but does not provide the original separate-window sheet. Narrow layouts stack the groups while preserving the supported command meanings and reachable close action.

## Historical scope and additional primary sources

- [2006 redesign](https://googlereader.blogspot.com/2006/09/something-looks-different.html): unread counts, mark all read, folder navigation and expanded reading.
- [2006 bug fixes](https://googlereader.blogspot.com/2006/10/bug-swatting.html): reset scroll position between feeds, prevent asynchronous feed content mixing, and add-folder behavior.
- [Trends introduction](https://googlereader.blogspot.com/2007/01/i-like-big-charts-and-i-cannot-lie.html): reading activity, subscription statistics and identifying inactive/rarely read feeds.
- [Personalized ranking](https://googlereader.blogspot.com/2009/10/reading-gets-personal-with-popular.html): recommendations and magic sorting depend on personal and aggregate server activity.
- [October 2011 redesign](https://googlereader.blogspot.com/2011/10/new-in-reader-fresh-design-and-google.html?m=1): native following, shared items and comments were retired in favor of Google+.
- [2011 migration notice](https://googlereader.blogspot.com/2011/10/upcoming-changes-to-reader-new-look-new.html): export and send-to remain available after the social redesign.
- [Official Reader archive](https://googlereader.blogspot.com/): July 2013 shutdown and data-removal notices.

Earlier screenshots showing People you follow, Your stuff, native comments or blue rounded section headers describe a different product revision. Those elements are not the default visual contract for this final-era reconstruction.

## Screenshot aids

- [Old/new design comparison](https://miro.medium.com/v2/resize%3Afit%3A1400/1%2A9giz_CNO0km-PKxjtNECog.jpeg).
- [Comfortable/Cozy/Compact gear menu](https://cdn.tutsplus.com/cdn-cgi/image/width%3D680/webdesign/uploads/legacy/articles/056_google_ui/images/comfortable-cosy-compact.png).
- [Final-era shutdown screenshot](https://openrss.org/media/google-reader-view-shutdown.png).

These illustrate the layout. The preserved stylesheets above are the authoritative measurement source. Font rasterization, article content, device scale and historical shared Google headers must be held constant before evaluating pixel differences.

## Original mobile Reader and touch evidence

Google maintained distinct mobile interfaces. Its [May 2006 announcement](https://googlereader.blogspot.com/2006/05/you-can-now-use-google-reader-from.html) introduced a lightweight mobile reader. The [May 2008 iPhone announcement](https://googlereader.blogspot.com/2008/05/brand-new-google-reader-for-iphone.html) introduced the richer `/reader/i/` interface: a tap expands a headline in the list, and star, share and keep-unread actions update in place. The [original 2008 screenshot](https://blogger.googleusercontent.com/img/b/R29vZ2xl/AVvXsEghYy_lsVyOgwD7g33Blgz7QmhkFdr2SYxbSnFFT3uY-fgpg-qTaB9FNqHv7stRuJ9MEPEI1MdW0qfyDVWvlr1cTQ1fFfbOCYW3XGT3Be9MhLEXI2akI1D0q8tfBGQkNRowMlM/s400/mobilescroll-small.png) is linked directly from that announcement.

The [January 2009 update](https://googlereader.blogspot.com/2009/01/what-we-did-on-our-winter-break.html) documents a more compact rich-mobile header for iPhone, Android and other browsers. The [January 2010 mobile release](https://googlereader.blogspot.com/2010/01/new-year-new-mobile-features-and-more.html) adds tagging, liking and oldest/newest sorting; places less frequent article actions behind a bottom **More** link; and replaces the secondary toolbar with an options dropdown. Its [article-action screenshot](https://blogger.googleusercontent.com/img/b/R29vZ2xl/AVvXsEiIj7bII1Nw-YmCUGMr3m0jP0zqSngrkSaPo0_9u9VtpWZa1J6Gp31FiuFMsW1m-bcFxpGnKgwfJh24JlJAtVvcsLBXrTGviH70pl4KZjVEYJNcWz_IxWUVTX6eI_KwzTlO1yG0/s400/photo%2B%283%29.jpg) and [header/options screenshot](https://blogger.googleusercontent.com/img/b/R29vZ2xl/AVvXsEhh8FAoJAnSqd38u2AXOK9jbDXtE8vFVUMRbM2SpWChRuA2GLnMUAXAAhVxf1WtToNaXc36HFIJFrQpLTq5U5iVmcyBPRIQCv87odh471NEZeE5K4AJN5HEO8ULsm_f88c5zpXt/s400/photo%2B%281%29.jpg) provide first-party visual evidence for that release, not an assertion about every final-2013 mobile pixel.

These primary sources support taps, inline expansion and explicit action controls. They do not establish left/right article swipe commands or pull-to-refresh as original Google Reader web behavior. Swipe-to-star/read gestures described in contemporary **Reeder** reviews concern a separate third-party iPhone application. Do not import those gestures under a claim of historical Google Reader parity. Neither `/reader/i/` nor `/reader/m/` could be retrieved from the preserved desktop host during this review; a complete runnable final mobile reference has not been verified.

The [official Android app announcement](https://googlereader.blogspot.com/2010/11/android-google-reader-app-is-here.html?m=1) separately documents long-press context menus for folders/subscriptions, volume-key navigation and sharing through other Android apps. This implementation exposes the existing subscription actions after a stationary held release; moving after the hold starts a cancellable native drag. The browser implementation does not claim the separate Android app's widgets, account synchronization or hardware-volume-key integration.

This project's narrow interface remains an adaptation of the final desktop design: a subscription drawer, responsive article/action layout, larger touch targets for a coarse primary pointer and keyboard-sized dialogs. One-finger scrolling uses FerroUI's native scrolling pipeline; browser pinch magnification remains available. Safe-area insets, visual-viewport keyboard handling and browser Back dismissal of transient overlays are modern host integration, not features attributed to the 2008 screenshots. [WebKit's first-party safe-area guidance](https://webkit.org/blog/7929/designing-websites-for-iphone-x/) explains why a page using `viewport-fit=cover` must inset interactive content around device cutouts and the home indicator.
