# Google Reader reference specification

The visual reference is the final desktop Google Reader client from 2011–2013, using its light appearance. Research was checked on 22 November 2026. The Material theme supplies FerroUI's underlying control behavior; Reader resources and templates supply the historical appearance.

## Primary reference

[Mihai Parparita's preserved Reader](https://readerisdead.com/reader/view/) contains the final public client. In his [first-person account of its preservation](https://blog.persistent.info/2013/07/using-google-readers-reanimated-corpse.html), the former Reader engineer explains that he saved its public HTML, CSS, JavaScript and images immediately before shutdown and supplied a replacement read-only API. His [2018 time-capsule article](https://medium.com/@mihai/google-reader-a-time-capsule-from-5-years-ago-df5176aa08b6) describes the publicly hosted version. The [preservation tooling](https://github.com/mihaip/readerisdead) is available separately.

Reader's implementation in this repository is original Rust using FerroUI. The preserved client is reference material; its obfuscated JavaScript, CSS implementation and article archive are not application dependencies.

The following publicly served reference resources provide exact values:

- [Main stylesheet](https://readerisdead.com/reader/ui/2139832829-en-scroll.css).
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

The default density is Comfortable. The separate top search header is 60 pixels high. The sidebar collapse strip is 8 pixels wide. Sidebar entries use 21-pixel line heights; primary section headings use 30 pixels. Collapsed source labels occupy approximately 11 em, followed by the title/snippet and a right-aligned date. Expanded article titles and bodies have a maximum width of 650 pixels.

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

In list view, unread titles are bold and read titles regular. Selection and read state are independent. In expanded view the currently selected article retains blue links even after becoming read. Article footers expose star, keep unread, email, tags and send-to actions with approximately 16 pixels between controls.

## Navigation and menus

The final client provides Home, All items, Starred items, Trends and Browse for stuff; an Explore section containing recommendations; and a Subscriptions tree. Unread count display is capped at `1000+`. Feed and folder selection participate in browser history. These behaviors are described in the [official search release](https://googlereader.blogspot.com/2007_09_01_archive.html).

The reading toolbar contains refresh, an All items/New items filter, a split Mark all as read button, feed or folder settings, previous/next controls and expanded/list view switches. A gear menu selects Comfortable, Cozy or Compact and opens Reader settings/help.

The Mark all as read dropdown offers all items, items older than a day, older than a week and older than two weeks. The [official feature announcement](https://googlereader.blogspot.com/2009/08/flurry-of-features-for-feed-readers.html) explains the age-based choices and configurable send-to destinations.

Feed settings include newest/oldest/magic sorting, rename, unsubscribe, translation, details/statistics and folder membership. Folder settings include rename/delete and unsubscribe from all. Subscription-section settings include alphabetical/manual ordering, all/updated visibility, favicon/default icon preference, add and manage subscriptions.

## Keyboard map

This table transcribes behavior from the preserved final keyboard-help resource. A space-separated sequence means successive keys, not a simultaneous chord.

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
