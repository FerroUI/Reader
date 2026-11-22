use reader_domain::{Action, AppState, FullArticle, MAX_ARTICLE_HTML_BYTES};
use std::rc::Rc;

fn library() -> AppState {
    let mut state = AppState::new();
    refresh(
        &mut state,
        "Initial title",
        "<p>Feed summary.</p>",
        "https://example.com/post#section",
    );
    state
}

fn refresh(state: &mut AppState, title: &str, body: &str, link: &str) {
    let xml = format!(
        "<rss><channel><title>Feed</title><link>https://example.com/</link><item><guid>post</guid><title>{title}</title><link>{link}</link><description><![CDATA[{body}]]></description></item></channel></rss>"
    );
    let feed = reader_domain::parse_feed(&xml, "https://example.com/feed", 100).unwrap();
    state
        .apply(
            Action::MergeFeed {
                url: "https://example.com/feed".into(),
                feed,
            },
            100,
        )
        .unwrap();
}

fn cache(state: &mut AppState) {
    let article = &state.articles[0];
    let content = FullArticle::new(
        "<p>Full publisher text with a searchable platypus.</p>".into(),
        &article.url,
        "https://cdn.example.com/articles/post/",
        200,
    )
    .unwrap();
    state
        .apply(
            Action::CacheFullArticle {
                id: article.id,
                content,
            },
            200,
        )
        .unwrap();
}

#[test]
fn full_copy_roundtrips_separately_from_feed_content_and_shares_snapshots() {
    let mut state = library();
    let original = state.articles[0].content.clone();
    let id = state.articles[0].id;
    state
        .apply(
            Action::SetNote {
                id,
                note: "My note".into(),
            },
            100,
        )
        .unwrap();
    state.apply(Action::ToggleStar(id), 100).unwrap();
    cache(&mut state);
    let article = state.article(id).unwrap();
    assert_eq!(article.content, original);
    assert!(article.show_full_content && article.has_full_content());
    assert!(article.reading_content().contains("platypus"));
    assert!(article.search_text.contains("platypus"));
    assert_eq!(
        article.reading_base_url(),
        "https://cdn.example.com/articles/post/"
    );
    assert_eq!(
        article.full_content.as_ref().unwrap().source_url,
        "https://example.com/post#section"
    );
    let snapshot = article.clone();
    assert!(Rc::ptr_eq(
        article.full_content.as_ref().unwrap(),
        snapshot.full_content.as_ref().unwrap()
    ));
    assert!(Rc::ptr_eq(&article.search_text, &snapshot.search_text));
    let restored = AppState::from_json(&state.to_json().unwrap()).unwrap();
    assert_eq!(restored, state);
    assert!(restored.article(id).unwrap().starred);
    assert_eq!(restored.article(id).unwrap().note, "My note");
    state
        .apply(Action::ShowFullArticle { id, show: false }, 300)
        .unwrap();
    assert_eq!(state.article(id).unwrap().reading_content(), original);
    assert!(state.article(id).unwrap().has_full_content());
}

#[test]
fn refresh_keeps_downloaded_copy_and_annotations_until_the_article_link_changes() {
    let mut state = library();
    cache(&mut state);
    let id = state.articles[0].id;
    state
        .apply(
            Action::SetTags {
                id,
                tags: vec!["Saved".into()],
            },
            200,
        )
        .unwrap();
    let before = state.article(id).unwrap().full_content.clone().unwrap();
    refresh(
        &mut state,
        "Corrected title",
        "<p>Corrected feed body.</p>",
        "https://example.com/post#section",
    );
    let article = state.article(id).unwrap();
    assert!(Rc::ptr_eq(&before, article.full_content.as_ref().unwrap()));
    assert_eq!(article.title, "Corrected title");
    assert_eq!(article.tags, ["Saved"]);
    assert!(article.show_full_content);
    refresh(
        &mut state,
        "New destination",
        "<p>Replacement summary.</p>",
        "https://example.com/replacement",
    );
    let article = state.article(id).unwrap();
    assert!(!article.has_full_content());
    assert!(!article.show_full_content);
    assert_eq!(article.tags, ["Saved"]);
    assert_eq!(article.reading_content(), "<p>Replacement summary.</p>");
    assert!(!article.search_text.contains("platypus"));
    assert!(
        state
            .apply(
                Action::CacheFullArticle {
                    id,
                    content: (*before).clone()
                },
                400
            )
            .is_err()
    );
}

#[test]
fn older_backups_and_invalid_optional_caches_preserve_the_feed_library() {
    let state = library();
    let source = state.to_json().unwrap();
    assert!(!source.contains("full_content"));
    assert_eq!(AppState::from_json(&source).unwrap(), state);
    for (source_url, base_url, html) in [
        (
            "https://wrong.example/post",
            "https://example.com/post",
            "<p>Wrong source</p>",
        ),
        (
            "https://example.com/post#section",
            "https://user:secret@example.com/post",
            "<p>Credentials</p>",
        ),
        (
            "https://example.com/post#section",
            "https://example.com/post",
            "<script>unsafe()</script>",
        ),
    ] {
        let mut document = serde_json::to_value(&state).unwrap();
        document["articles"][0]["full_content"] = serde_json::json!({"html":html,"source_url":source_url,"base_url":base_url,"fetched_at":200});
        document["articles"][0]["show_full_content"] = true.into();
        let restored = AppState::from_json(&document.to_string()).unwrap();
        assert_eq!(
            restored, state,
            "invalid optional cache is discarded without losing the valid document"
        );
    }
}

#[test]
fn oversized_or_empty_downloads_cannot_replace_saved_content() {
    let mut state = library();
    cache(&mut state);
    let original = state.clone();
    let id = state.articles[0].id;
    for html in [
        String::new(),
        " ".into(),
        "x".repeat(MAX_ARTICLE_HTML_BYTES + 1),
    ] {
        let content = FullArticle {
            html,
            source_url: state.articles[0].url.clone(),
            base_url: "https://example.com/post".into(),
            fetched_at: 400,
        };
        assert!(
            state
                .apply(Action::CacheFullArticle { id, content }, 400)
                .is_err()
        );
        assert_eq!(state, original);
    }
}
