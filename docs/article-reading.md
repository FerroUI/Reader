# Reading articles inside Reader

Open an item in expanded view or expand its list row. The feed's original body appears immediately. Choose **Read full article** to download the publisher's page, extract its readable content and show it in the same reading pane.

The title and **View original** action still open the publisher, matching the original Reader navigation model. Full-article extraction is an explicit extension for feeds that provide summaries or links, including Hacker News. Merely scrolling or selecting an article does not download its website.

## Reading controls

| Control | Behavior |
| --- | --- |
| Read full article | Downloads and extracts on demand; opens an existing saved copy immediately |
| Cancel | Stops the active browser worker or native curl download; retains feed text and any previous saved copy |
| Feed content | Returns to the original RSS/Atom/JSON Feed body |
| Full article | Reopens the saved publisher copy without a request |
| Reload | Fetches a fresh copy; an error leaves the previous copy available |
| Try again | Retries a failed download after the connection or publisher problem is resolved |

Article errors appear beside these controls and do not remove the subscription, navigate to another stream or alter stars, tags and notes. The saved text participates in search, print and complete JSON backups. Feed refresh preserves it when the article URL remains unchanged.

## Browser connection

Article requests follow the connection preference under **Settings → Reader settings → Feed connection**:

- **Direct only:** fetch the publisher with no cookies or authorization headers. Its response must allow browser access.
- **Custom proxy:** try the publisher, then the configured trusted raw proxy after a network/CORS failure. The proxy must return original HTML and allow the Reader origin. Its `{url}` token receives the encoded article address.
- **Automatic:** try the publisher, then a saved custom proxy if configured. Otherwise use [Jina Reader](https://github.com/jina-ai/reader) for eligible public article URLs. Feed requests continue to use rss2json; these are separate services.

The public article fallback sends the requested article URL to `https://r.jina.ai/`. Its documented `X-Respond-With: html` and `X-Respond-Timing: html` options provide the original HTML response for the local extractor. Reader does not ask the service to execute the publisher's JavaScript. The service may cache responses, impose quotas or be unavailable. It receives the URL and request technical information; Reader does not send the library, notes, annotations or browser cookies.

Local/IP addresses and recognizable private/token-bearing links are not forwarded to that public service. A URL can contain secrets that no heuristic recognizes, so select Direct only or your trusted proxy for private sources. Readable HTTP denials, missing pages, unsupported formats and oversized responses remain errors rather than triggering a different provider. Rate-limit responses honor a bounded cooldown. A proxy response does not expose the final publisher URL reliably; publisher-relative links use the requested address and any valid `<base href>` in the returned document.

The desktop host fetches directly with curl and runs the same Rust extractor in its background pool. It does not need a CORS relay. Desktop documents currently require UTF-8; browser downloads also decode supported declared character sets.

## Content, offline behavior and limits

The HTML5/Readability extractor is [dom_smoothie 0.19.0](https://github.com/niklak/dom_smoothie). Its output is projected into native FerroUI controls. Publisher CSS and scripts are not applied to Reader. Headings, paragraphs, emphasis, links, nested lists, quotes, code, structured tables and images are supported. Media links open the original destination; there is no embedded publisher player or iframe.

Full text is saved in the local library. It remains readable offline after storage succeeds and is included in exported JSON backups. Images have a separate bounded memory cache and may still require a connection after reload; the “saved” label refers to article text. The offline application cache includes the extraction worker and its version-matched WASM module.

Publisher pages are limited to 4 MiB, extracted article HTML to 2 MiB and the library's saved full copies to 24 MiB. Oldest fetched copies are evicted first, preserving the original feed body and every annotation. The renderer also bounds pathological block/span/table/image counts and displays a notice if only a readable prefix fits. A site can legitimately supply only a preview, require JavaScript, request browser verification or reserve content for subscribers. Reader cannot promise full text for every URL; **View original** remains available for those cases.

## Scrolling through unequal article sizes

The reading list uses FerroUI's reusable TreeDataGrid presenter with a measured-height index, native anchoring and bounded recycling. Each visited article contributes its actual height. Unseen articles keep independent estimates, so measuring one long article does not rescale every short article in the library. Keyboard jumps use indexed offsets and the actual reading width. Width/content changes invalidate geometry, while read/star updates retain it.

Image placeholders reserve declared dimensions, and each image completion updates its own control instead of every image in the stream. Parsed body data is cached across annotation and viewport changes. The [verification record](verification.md) distinguishes the automated mixed-height tests, rendered browser evidence and remaining platform limits.
