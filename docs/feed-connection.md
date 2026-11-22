# Feed connections in the browser

Choose **Subscribe**, enter a feed address and select **Add**. Reader accepts full HTTP/HTTPS URLs and bare website addresses such as `example.com/feed.xml`; bare addresses use HTTPS. Embedded usernames/passwords and unsupported schemes are rejected. When you use Subscribe, the new subscription is committed only after a successful download and feed parse.

## Default subscriptions

A fresh library starts with six real publisher subscriptions, grouped into three folders, and fetches their articles on first use. There are no bundled illustrative stories. The links below are the publishers' own RSS/Atom endpoints:

| Folder | Publisher | Feed address |
| --- | --- | --- |
| Technology | [Hacker News](https://news.ycombinator.com/) | [news.ycombinator.com/rss](https://news.ycombinator.com/rss) |
| Technology | [Ars Technica](https://arstechnica.com/rss-feeds/) | [feeds.arstechnica.com/arstechnica/index](https://feeds.arstechnica.com/arstechnica/index) |
| Technology | [The Verge](https://www.theverge.com/) | [theverge.com/rss/index.xml](https://www.theverge.com/rss/index.xml) |
| Development | [Rust Blog](https://blog.rust-lang.org/) | [blog.rust-lang.org/feed.xml](https://blog.rust-lang.org/feed.xml) |
| Development | [GitHub Blog](https://github.blog/) | [github.blog/feed/](https://github.blog/feed/) |
| News | [BBC News](https://www.bbc.com/news) | [feeds.bbci.co.uk/news/rss.xml](https://feeds.bbci.co.uk/news/rss.xml) |

These are ordinary subscriptions: you can rename them, move them between folders or unsubscribe. First-use setup does not replace an established user library or re-add defaults after you intentionally empty the library. Add more sources through Subscribe, Explore or an OPML import.

The initial fetch uses the same connection policy as a manual refresh. Articles appear as publishers respond; an unavailable source does not cause Reader to manufacture replacement content. If you start offline or a request fails, reconnect and choose **Refresh**. Publisher or public-service caching, availability and item limits still apply.

## Why some publishers need a relay

[GitHub Pages is a static hosting service](https://docs.github.com/en/pages/getting-started-with-github-pages/what-is-github-pages). It hosts Reader's application files, but cannot change a publisher's response headers or run Reader's own feed-fetching server. Browsers require a readable cross-origin response to carry the appropriate [CORS headers](https://developer.mozilla.org/en-US/docs/Web/HTTP/Guides/CORS). Opening a feed in a tab does not establish that JavaScript on another site may read it.

Reader's **Automatic** mode provides a separate service path when direct access fails. It does not disable browser security or replace the retired Google feed crawler.

## Choose a connection

Open **Settings → Reader settings → Feed connection**, choose a mode and select **Save feed connection**. Mode and URL edits remain a draft until saved; invalid custom templates leave the previous settings intact.

| Mode | Request order | Use it when |
| --- | --- | --- |
| **Automatic** — default | Publisher directly, then a saved custom proxy if present; otherwise rss2json for eligible public RSS/Atom URLs | Reading ordinary public feeds without configuring a server |
| **Direct only** | Publisher directly, with no relay | The publisher supports browser access, or you do not want a third-party service to receive the address |
| **Custom proxy** | Publisher directly, then the configured raw-response endpoint | You control or trust a feed proxy, need full raw feed responses, or need HTML/JSON Feed transport without publisher CORS support |

An old library's saved custom proxy is retained and takes precedence in Automatic mode. Clear the custom field and save Automatic to use the built-in service. Choosing Direct only preserves that field but does not use it. These settings affect the browser host; the desktop host continues to use its native direct transport.

Fallback is used for a connection failure or timeout. A readable publisher HTTP error, unsupported encoding or oversized response keeps its actual error instead of being retried through another service.

## Automatic service: rss2json

Reader sends a public RSS/Atom URL to the [rss2json API](https://rss2json.com/docs) without an API key, then normalizes the returned items for its shared Rust feed parser. The publisher URL remains the subscription identity; the API endpoint is never saved as the feed address.

The provider's documented default is **10 items**, and changing the requested count requires an API key. Responses can be cached and are subject to daily/per-minute quotas; anonymous usage is counted by website or IP. Limits can change, so consult the provider's documentation. Reader's built-in mode does not configure a paid account or a higher item allowance. Already saved articles remain in the library when a later response contains fewer items.

The service receives the feed address and returned content. Its [privacy policy](https://rss2json.com/privacy-policy) also describes collection of requested feeds, request counts and technical data such as IP/browser information. Reader does not send its complete library, notes, read state or browser cookies to this API.

Automatic does not forward IP-literal/local addresses or recognizable token-bearing/private links to the public service. This guard cannot infer every secret embedded in an arbitrary URL. For private feeds, choose Direct only or a custom endpoint you trust. Reader does not add authentication headers or browser cookies, and embedded username/password URLs remain unsupported.

**JSON Feed works directly or through a custom raw proxy.** The built-in conversion service is the RSS/Atom fallback; it is not a general raw-content proxy or a guarantee of every publisher's full history.

## Configure a custom raw proxy

Enter a template such as:

```text
https://your-service.example/feed?url={url}
```

Reader replaces every literal `{url}` with the percent-encoded publisher address. The endpoint must:

- Return the raw RSS, Atom, JSON Feed or HTML body, with an appropriate content type; a provider-specific JSON wrapper is not a raw response.
- Permit the deployed Reader origin through CORS, including readable error responses.
- Use HTTPS when Reader is served over HTTPS.
- Return meaningful HTTP statuses and, when applicable, a readable `Retry-After` header.

Custom mode requires a nonempty valid template. The proxy sees feed URLs and content, so configure an endpoint whose operator you trust. Reader does not automatically switch to unrelated proxy providers when that endpoint fails.

A raw proxy's HTTP response URL normally identifies the proxy itself. Reader therefore retains the requested publisher URL as the content base. If a proxied website redirects to a different directory and its links are relative, use its final page/feed address or a page with a correct `<base href>`.

## Feed discovery from a website

When Reader can read a website's HTML directly or through a custom raw proxy, it inspects advertised RSS/Atom/JSON Feed alternate links. It honors `<base href>`, resolves relative addresses, skips invalid or duplicate alternates and tries up to four candidates. Leading comments and HTML response types are recognized.

The public RSS/Atom conversion API does not provide arbitrary website HTML. If a blocked homepage cannot be added, find that publisher's RSS/Atom link and paste the **actual feed URL**, or use a custom raw proxy. Reader does not guess that a failed website request created a subscription.

## Understand failures

| Message or condition | Meaning and next step |
| --- | --- |
| **Offline** | Reconnect before adding or refreshing. Saved article text remains available. |
| **Could not connect** / direct-access explanation | A browser network failure can include CORS, DNS, TLS or connection failure; JavaScript cannot reliably distinguish them. Use Automatic or a trusted custom proxy if appropriate. |
| **HTTP 404** | The readable endpoint reported that the feed was not found. Check its actual RSS/Atom/JSON Feed address. |
| **HTTP 401/403** | The readable endpoint denied access. This is not reported as a successful subscription. |
| **HTTP 429** | The endpoint is rate limited. For relay requests Reader observes `Retry-After`, or a short default cooldown when it is absent. Avoid repeated retries. |
| **Timeout** | Direct access or the feed service did not finish in time. A feed/discovery operation has a bounded deadline; retry later or supply the direct feed URL. |
| **No feed found** / service could not load the feed | Supply the real advertised feed address; a web page or service error is not a feed. |
| **Invalid proxy**, **unsupported encoding**, or **download limit** | Correct the configuration or source. These errors are not relabeled as CORS failures. |

Feed responses are capped at 8 MiB, discovery has a 30-second operation budget, and refresh uses at most four workers. Errors preserve saved subscriptions and annotations. A refresh keeps the original publisher identity and merges returned articles, rather than creating a duplicate subscription under a relay URL.

See [verification.md](verification.md) for the exact tested release and results. The cross-origin browser fixture deliberately uses a publisher without CORS headers and a separate relay; passing that fixture is distinct from availability of any public provider or publisher.
