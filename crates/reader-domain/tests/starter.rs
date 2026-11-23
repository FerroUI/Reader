use reader_domain::{Action, AppState, FeedConnection, DEFAULT_FEEDS};

#[test]
fn fresh_library_has_only_real_subscription_metadata_and_no_articles() {
    let state = AppState::with_default_feeds();
    assert_eq!(state.feeds.len(), 6);
    assert!(state.articles.is_empty());
    assert_eq!(state.folders, ["Development", "News", "Technology"]);
    for (feed, source) in state.feeds.iter().zip(DEFAULT_FEEDS) {
        assert_eq!(feed.url, source.url);
        assert_eq!(feed.title, source.title);
        assert_eq!(feed.folder, source.folder);
        assert_eq!(feed.fetched_at, 0);
        assert!(!feed.sample);
        assert!(feed.error.is_none());
        assert_eq!(state.feed(feed.id), Some(feed));
        assert_eq!(
            reader_domain::subscription_url(&feed.url),
            Some(feed.url.clone())
        );
    }
    assert_eq!(
        AppState::from_json(&state.to_json().unwrap()).unwrap(),
        state
    );
}

#[test]
fn intact_legacy_demo_moves_to_defaults_once_and_keeps_settings_and_retained_ids() {
    let mut state = AppState::sample(1_800_000_000);
    let rust_id = state.feeds[0].id;
    let hn_id = state.feeds[5].id;
    let next = state.next_id;
    state.settings.font_size = 19.0;
    state.settings.feed_connection = FeedConnection::Direct;
    state.settings.feed_proxy = "https://trusted.example/?url={url}".into();
    let settings = state.settings.clone();
    assert!(state.migrate_legacy_samples());
    assert!(
        state.articles.is_empty(),
        "the originally starred/tagged demo row is also retired"
    );
    assert_eq!(state.settings, settings);
    assert_eq!(
        state
            .feeds
            .iter()
            .map(|feed| feed.url.as_str())
            .collect::<Vec<_>>(),
        DEFAULT_FEEDS
            .iter()
            .map(|feed| feed.url)
            .collect::<Vec<_>>()
    );
    assert_eq!(state.feeds[0].id, hn_id);
    assert_eq!(state.feeds[3].id, rust_id);
    assert!(state
        .feeds
        .iter()
        .filter(|feed| feed.id != hn_id && feed.id != rust_id)
        .all(|feed| feed.id >= next));
    assert!(state.feeds.iter().all(|feed| !feed.sample));
    let migrated = state.clone();
    assert!(!state.migrate_legacy_samples());
    assert_eq!(state, migrated);
    assert_eq!(
        AppState::from_json(&state.to_json().unwrap()).unwrap(),
        state
    );
}

#[test]
fn removed_demo_subscription_is_not_restored_or_replaced_by_another_default() {
    let mut state = AppState::sample(1000);
    state
        .apply(Action::RemoveFeed(state.feeds[2].id), 1000)
        .unwrap();
    let urls: Vec<_> = state.feeds.iter().map(|feed| feed.url.clone()).collect();
    assert!(state.migrate_legacy_samples());
    assert_eq!(
        state
            .feeds
            .iter()
            .map(|feed| feed.url.clone())
            .collect::<Vec<_>>(),
        urls
    );
    assert!(state.articles.is_empty());
    assert!(!state.migrate_legacy_samples());
}

#[test]
fn archived_demo_rows_keep_user_annotations_but_retire_shipped_stars_and_tags() {
    let mut state = AppState::sample(1000);
    let saved_id = state.articles[0].id;
    let shipped_id = state.articles[2].id;
    state
        .apply(
            Action::SetNote {
                id: saved_id,
                note: "My own saved note".into(),
            },
            1000,
        )
        .unwrap();
    let user_source = state.articles[0].feed_id;
    let shipped_source = state.articles[2].feed_id;
    state
        .apply(Action::RemoveFeeds(vec![user_source, shipped_source]), 1000)
        .unwrap();
    assert!(state.feed(user_source).unwrap().archived);
    assert!(state.feed(shipped_source).unwrap().archived);
    assert!(state.migrate_legacy_samples());
    assert_eq!(state.article(saved_id).unwrap().note, "My own saved note");
    assert!(state.article(shipped_id).is_none());
    assert!(state.feed(user_source).unwrap().archived);
    assert!(state.feed(shipped_source).unwrap().archived);
    assert!(!state.migrate_legacy_samples());
    assert_eq!(
        AppState::from_json(&state.to_json().unwrap()).unwrap(),
        state
    );
}

#[test]
fn customized_library_retains_real_records_and_user_annotations_on_demo_rows() {
    let mut state = AppState::sample(1000);
    state.articles[0].note = "My own note".into();
    state.articles[1].starred = true;
    state.articles[3].tags = vec!["Saved by me".into()];
    state.feeds[0].title = "My Rust subscription".into();
    state
        .apply(
            Action::AddSubscription {
                title: "My site".into(),
                url: "https://mine.example/rss".into(),
                site_url: "https://mine.example/".into(),
                folder: "Personal".into(),
            },
            1000,
        )
        .unwrap();
    let feed_id = state.feeds.last().unwrap().id;
    let mut real = state.articles[0].clone();
    real.id = state.next_id;
    state.next_id += 1;
    real.feed_id = feed_id;
    real.guid = "publisher-guid".into();
    real.title = "Publisher article".into();
    real.read = true;
    real.starred = true;
    state.articles.push(real.clone());
    state.normalize().unwrap();
    let real = state.article(real.id).unwrap().clone();
    let annotated = state.articles[..4]
        .iter()
        .filter(|a| a.guid != "reader-sample-2")
        .cloned()
        .collect::<Vec<_>>();
    let feeds = state.feeds.clone();
    assert!(state.migrate_legacy_samples());
    assert_eq!(
        state.feeds.iter().map(|f| f.id).collect::<Vec<_>>(),
        feeds.iter().map(|f| f.id).collect::<Vec<_>>()
    );
    assert_eq!(state.feeds[0].title, "My Rust subscription");
    assert_eq!(state.articles.len(), 4);
    for article in annotated {
        assert_eq!(state.article(article.id), Some(&article));
    }
    assert_eq!(state.article(real.id), Some(&real));
    assert!(!state.migrate_legacy_samples());
}

#[test]
fn a_sample_like_guid_cannot_delete_modified_content_or_another_publishers_data() {
    let mut state = AppState::sample(1000);
    state.articles[0].title = "Imported content with a coinciding GUID".into();
    state.articles[1].author = "Publisher author".into();
    state.articles[2].url = "https://example.com/real-article".into();
    state.normalize().unwrap();
    let preserved = state.articles[..3].to_vec();
    assert!(state.migrate_legacy_samples());
    assert_eq!(state.articles, preserved);
}

#[test]
fn earlier_refresh_leftovers_are_removed_without_deleting_real_unstarred_articles() {
    let mut state = AppState::sample(1000);
    let source = state.feeds[2].clone();
    state.feeds[2].sample = false;
    let mut real = state.articles[2].clone();
    real.id = state.next_id;
    state.next_id += 1;
    real.guid = "actual-publisher-item".into();
    real.title = "Real article".into();
    real.starred = false;
    real.tags.clear();
    state.articles.push(real.clone());
    state.normalize().unwrap();
    let real = state.article(real.id).unwrap().clone();
    assert!(state.migrate_legacy_samples());
    assert_eq!(state.articles, [real.clone()]);
    state
        .feeds
        .iter_mut()
        .find(|feed| feed.id == source.id)
        .unwrap()
        .sample = true;
    let parsed = reader_domain::parse_feed(
        "<rss><channel><title>Publisher</title></channel></rss>",
        &source.url,
        2000,
    )
    .unwrap();
    state
        .apply(
            Action::MergeFeed {
                url: source.url,
                feed: parsed,
            },
            2000,
        )
        .unwrap();
    assert_eq!(
        state.article(real.id),
        Some(&real),
        "refresh must not purge real rows because of a stale sample flag"
    );
}

#[test]
fn empty_libraries_and_user_removed_defaults_stay_empty_or_removed() {
    let mut empty = AppState::new();
    assert!(!empty.migrate_legacy_samples());
    assert!(empty.feeds.is_empty());
    let mut state = AppState::with_default_feeds();
    let removed = state.feeds[0].id;
    state.apply(Action::RemoveFeed(removed), 1000).unwrap();
    assert!(!state.migrate_legacy_samples());
    assert_eq!(state.feeds.len(), 5);
    assert!(state.feed(removed).is_none());
    let old = AppState::sample(1000);
    assert_eq!(
        AppState::from_json(&old.to_json().unwrap()).unwrap(),
        old,
        "explicit parsing/restoration does not seed or migrate a saved document implicitly"
    );
}

#[test]
fn legacy_migration_does_not_overflow_a_document_near_identifier_exhaustion() {
    let mut state = AppState::sample(1000);
    state.next_id = u64::MAX - 2;
    state.normalize().unwrap();
    let urls: Vec<_> = state.feeds.iter().map(|feed| feed.url.clone()).collect();
    assert!(state.migrate_legacy_samples());
    assert!(state.articles.is_empty());
    assert_eq!(
        state
            .feeds
            .iter()
            .map(|feed| feed.url.clone())
            .collect::<Vec<_>>(),
        urls
    );
    assert_eq!(state.next_id, u64::MAX - 2);
}
