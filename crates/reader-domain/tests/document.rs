use reader_domain::{parse_opml, search_matches, Action, AppState, View};

#[test]
fn opml_round_trip_preserves_folders_and_escapes() {
    let opml = r#"<opml version="2.0"><body><outline text="Design &amp; Code"><outline text="A &amp; B" xmlUrl="https://example.com/feed?a=1&amp;b=2" htmlUrl="https://example.com/"/></outline><outline text="Duplicate" xmlUrl="https://example.com/feed?a=1&amp;b=2"/><outline xmlUrl="javascript:bad"/></body></opml>"#;
    let parsed = parse_opml(opml).unwrap();
    assert_eq!(parsed.len(), 1);
    let mut state = AppState::new();
    for sub in &parsed {
        state
            .apply(
                Action::AddSubscription {
                    title: sub.title.clone(),
                    url: sub.url.clone(),
                    site_url: sub.site_url.clone(),
                    folder: sub.folder.clone(),
                },
                0,
            )
            .unwrap();
    }
    assert_eq!(parse_opml(&state.export_opml()).unwrap(), parsed);
}

#[test]
fn backup_validation_and_search_operators() {
    let state = AppState::sample(1_800_000_000);
    let restored = AppState::from_json(&state.to_json().unwrap()).unwrap();
    assert_eq!(state, restored);
    assert!(AppState::from_json("{}").is_err());
    let mut bad = serde_json::to_value(&state).unwrap();
    bad["articles"][0]["feed_id"] = serde_json::json!(999);
    assert!(AppState::from_json(&bad.to_string()).is_err());
    let starred = state.articles.iter().find(|a| a.starred).unwrap();
    assert!(search_matches(
        starred,
        &state,
        "is:starred tag:inspiration from:smashing"
    ));
    assert!(!search_matches(
        starred,
        &state,
        "is:starred -tag:inspiration"
    ));
    assert!(reader_domain::matches_view(
        starred,
        &state,
        &View::Folder("Design".into())
    ));
}

#[test]
fn delete_folder_preserves_subscriptions_and_article_data() {
    let mut state = AppState::sample(0);
    let articles = state.articles.clone();
    let subscriptions = state.feeds.len();
    state
        .apply(Action::DeleteFolder("Design".into()), 0)
        .unwrap();
    assert_eq!(state.articles, articles);
    assert_eq!(state.feeds.len(), subscriptions);
    assert!(!state.folders().contains(&"Design".into()));
}

#[test]
fn scoped_search_matches_folder_membership_and_feed_identity() {
    let mut state = AppState::sample(1000);
    let article = state.articles[0].clone();
    state
        .apply(
            Action::SetFeedFolders {
                id: article.feed_id,
                folders: vec!["Daily reading".into(), "Technology".into()],
            },
            1000,
        )
        .unwrap();
    assert!(search_matches(
        &article,
        &state,
        &format!("folder:\"Daily reading\" feed-id:{}", article.feed_id)
    ));
    assert!(search_matches(&article, &state, "folder:TECHNOLOGY"));
    assert!(!search_matches(&article, &state, "feed-id:9999"));
    assert!(!search_matches(&article, &state, "-folder:technology"));
}
