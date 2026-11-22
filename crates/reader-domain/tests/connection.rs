use reader_domain::{subscription_url, validate_feed_proxy, AppState, FeedConnection};

#[test]
fn subscribe_accepts_website_addresses_without_reinterpreting_unsupported_schemes() {
    for (input, expected) in [
        (" example.com ", "https://example.com/"),
        (
            "example.com/blog/feed.xml#item",
            "https://example.com/blog/feed.xml",
        ),
        (
            "//example.com/rss?a=1&b=2",
            "https://example.com/rss?a=1&b=2",
        ),
        ("example.com:8443/feed", "https://example.com:8443/feed"),
        (
            "example.com/rss?source=https://other.example/feed",
            "https://example.com/rss?source=https://other.example/feed",
        ),
        ("http://localhost:8080/rss", "http://localhost:8080/rss"),
        ("[::1]:8080/rss", "https://[::1]:8080/rss"),
    ] {
        assert_eq!(
            subscription_url(input).as_deref(),
            Some(expected),
            "{input}"
        );
    }
    for input in [
        "",
        " ",
        "example .com",
        "example.com\n/rss",
        "javascript:alert(1)",
        "file:///etc/passwd",
        "ftp://example.com/feed",
        "data:text/xml,<rss/>",
        "mailto:reader@example.com",
        "http:example.com",
        "https://user:secret@example.com/rss",
        "//user:secret@example.com/rss",
        "example.com:wrong/feed",
        "https://example.com\\@other.com/feed",
    ] {
        assert_eq!(subscription_url(input), None, "{input}");
    }
}

#[test]
fn connection_policy_migrates_old_backups_without_losing_the_custom_endpoint() {
    let proxy = "https://proxy.example/?url={url}";
    let old = format!(r#"{{"feeds":[],"articles":[],"settings":{{"feed_proxy":"{proxy}"}}}}"#);
    let mut state = AppState::from_json(&old).unwrap();
    assert_eq!(state.settings.feed_connection, FeedConnection::Automatic);
    assert_eq!(state.settings.feed_proxy, proxy);
    state.settings.feed_connection = FeedConnection::Direct;
    let restored = AppState::from_json(&state.to_json().unwrap()).unwrap();
    assert_eq!(restored.settings.feed_connection, FeedConnection::Direct);
    assert_eq!(restored.settings.feed_proxy, proxy);
    let fresh = AppState::from_json(r#"{"feeds":[],"articles":[]}"#).unwrap();
    assert_eq!(fresh.settings.feed_connection, FeedConnection::Automatic);
    assert!(fresh.settings.feed_proxy.is_empty());
}

#[test]
fn custom_templates_retain_the_placeholder_and_reject_unsafe_or_unusable_forms() {
    assert_eq!(validate_feed_proxy("  ").unwrap(), "");
    let template = "https://proxy.example/fetch?url={url}&format=raw";
    assert_eq!(
        validate_feed_proxy(&format!(" {template} ")).unwrap(),
        template
    );
    for invalid in [
        "https://proxy.example/fetch",
        "javascript:fetch('{url}')",
        "file:///{url}",
        "https://name:secret@proxy.example/?url={url}",
        "https://proxy.example/#{url}",
        "https://proxy.example/ a?url={url}",
        "https://{url}/fetch",
    ] {
        assert!(validate_feed_proxy(invalid).is_err(), "{invalid}");
    }
}
