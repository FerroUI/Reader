use reader_domain::{parse_feed, plain_text, safe_url, Action, AppState};

const RSS: &str = r#"<?xml version="1.0"?><rss version="2.0" xmlns:content="http://purl.org/rss/1.0/modules/content/"><channel><title>Example &amp; News</title><link>https://example.com</link><item><guid>first</guid><title>One &amp; two</title><link>/one</link><pubDate>Wed, 05 Nov 2025 12:00:00 +0000</pubDate><content:encoded><![CDATA[<p>Hello <strong>world</strong>.</p><script>bad()</script>]]></content:encoded></item></channel></rss>"#;

#[test]
fn rss_refresh_preserves_reader_state_and_deduplicates_guids() {
    let mut state = AppState::new();
    let parsed = parse_feed(RSS, "https://example.com/feed", 1000).unwrap();
    assert_eq!(parsed.title, "Example & News");
    assert_eq!(parsed.articles[0].url, "https://example.com/one");
    assert_eq!(parsed.articles[0].summary, "Hello world.");
    state
        .apply(
            Action::MergeFeed {
                url: "https://example.com/feed".into(),
                feed: parsed.clone(),
            },
            1000,
        )
        .unwrap();
    let id = state.articles[0].id;
    state
        .apply(Action::SetRead { id, read: true }, 1500)
        .unwrap();
    state.apply(Action::ToggleStar(id), 1500).unwrap();
    state
        .apply(
            Action::SetTags {
                id,
                tags: vec![" saved ".into(), "saved".into()],
            },
            1500,
        )
        .unwrap();
    let mut refreshed = parsed;
    refreshed.articles[0].title = "Updated title".into();
    refreshed.articles.push(refreshed.articles[0].clone());
    state
        .apply(
            Action::MergeFeed {
                url: "https://example.com/feed".into(),
                feed: refreshed,
            },
            2000,
        )
        .unwrap();
    assert_eq!(state.articles.len(), 1);
    let article = state.article(id).unwrap();
    assert!(article.read && article.starred);
    assert_eq!(article.read_at, Some(1500));
    assert_eq!(article.tags, vec!["saved"]);
    assert_eq!(article.title, "Updated title");
}

#[test]
fn atom_xhtml_relative_links_and_dates_are_supported() {
    let atom = r#"<feed xmlns="http://www.w3.org/2005/Atom"><title>Atom</title><link href="https://example.com/"/><entry><id>tag:example,1</id><title>Entry</title><link rel="alternate" href="post"/><updated>2025-01-02T03:04:05Z</updated><author><name>Ada</name></author><content type="xhtml"><div xmlns="http://www.w3.org/1999/xhtml"><p>Real <b>markup</b></p></div></content></entry></feed>"#;
    let parsed = parse_feed(atom, "https://example.com/feed", 0).unwrap();
    assert_eq!(parsed.articles[0].author, "Ada");
    assert_eq!(parsed.articles[0].published, 1735787045);
    assert_eq!(parsed.articles[0].url, "https://example.com/post");
    assert!(parsed.articles[0].content.contains("<b>markup</b>"));
}

#[test]
fn json_feed_and_untrusted_url_boundaries() {
    let json = r#"{"version":"https://jsonfeed.org/version/1.1","title":"JSON","items":[{"id":"1","title":"Item","url":"javascript:alert(1)","content_text":"<not markup>"}]}"#;
    let parsed = parse_feed(json, "https://example.com/feed", 42).unwrap();
    assert!(parsed.articles[0].url.is_empty());
    assert_eq!(parsed.articles[0].content, "&lt;not markup&gt;");
    assert!(safe_url("data:text/html,bad", None).is_none());
    assert!(safe_url("", Some("https://example.com")).is_none());
    assert_eq!(
        plain_text("<style>hidden</style>A &#x1F680; &amp; B"),
        "A 🚀 & B"
    );
    assert!(parse_feed(
        "<html><title>Not a feed</title></html>",
        "https://example.com/",
        0
    )
    .is_err());
}
