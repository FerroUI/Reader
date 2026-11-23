use reader_domain::{Action, AppState, Article, Feed, SortOrder, StreamSettings, View, ViewMode};

fn library() -> AppState {
    let mut state = AppState::new();
    state.feeds = (1..=3)
        .map(|id| Feed {
            id,
            title: format!("Source {id}"),
            url: format!("https://example.com/{id}/feed"),
            site_url: format!("https://example.com/{id}/"),
            folder: "Daily".into(),
            folders: vec!["Daily".into(), "Shared".into()],
            fetched_at: 100,
            ..Feed::default()
        })
        .collect();
    state.articles = (11..=15)
        .map(|id| Article {
            id,
            feed_id: 1,
            guid: format!("item-{id}"),
            title: format!("Story {id}"),
            url: format!("https://example.com/1/{id}"),
            content: "<p>Saved source body</p>".into(),
            starred: id == 11,
            tags: if id == 12 {
                vec!["Keep".into()]
            } else {
                Vec::new()
            },
            note: match id {
                13 => "A private note",
                14 => "  \n ",
                _ => "",
            }
            .into(),
            read: id == 15,
            ..Article::default()
        })
        .collect();
    state.articles.push(Article {
        id: 21,
        feed_id: 2,
        guid: "other".into(),
        title: "Untouched source".into(),
        url: "https://example.com/2/21".into(),
        starred: true,
        ..Article::default()
    });
    state.folders = vec!["Daily".into(), "Shared".into()];
    state.normalize().unwrap();
    state
}

#[test]
fn bulk_unsubscribe_archives_annotated_items_and_removes_only_the_confirmed_sources() {
    let mut state = library();
    let before = state.clone();
    state.settings.start_page = View::Feed(1);
    let changes = state
        .apply(Action::RemoveFeeds(vec![1, 3, 1]), 200)
        .unwrap();
    assert_eq!(changes.removed, [14, 15]);
    assert!(changes.changed && changes.feeds && changes.settings);
    let archived = state.feed(1).unwrap();
    assert!(archived.archived && archived.folder_names().is_empty());
    assert_eq!(archived.title, before.feed(1).unwrap().title);
    assert!(state.feed(3).is_none());
    assert_eq!(state.feed(2), before.feed(2));
    assert_eq!(state.article(21), before.article(21));
    for id in 11..=13 {
        assert_eq!(state.article(id), before.article(id));
    }
    assert_eq!(state.settings.start_page, View::Home);
    assert_eq!(
        AppState::from_json(&state.to_json().unwrap()).unwrap(),
        state
    );
    assert!(
        !state.apply(Action::RemoveFeed(1), 200).unwrap().changed,
        "removing an already archived source cannot erase saved annotations"
    );
}

#[test]
fn stale_bulk_unsubscribe_rejects_the_entire_selection() {
    let mut state = library();
    let before = state.clone();
    assert!(state
        .apply(Action::RemoveFeeds(vec![1, 999, 2]), 200)
        .is_err());
    assert_eq!(state, before);
    assert!(
        !state
            .apply(Action::RemoveFeeds(Vec::new()), 200)
            .unwrap()
            .changed
    );
    state.apply(Action::RemoveFeed(1), 200).unwrap();
    let archived = state.clone();
    assert!(state.apply(Action::RemoveFeeds(vec![2, 1]), 200).is_err());
    assert_eq!(state, archived);
}

#[test]
fn archived_sources_leave_subscription_streams_and_opml_but_keep_saved_search_and_source_views() {
    let mut state = library();
    state.apply(Action::RemoveFeed(1), 200).unwrap();
    let star = state.article(11).unwrap();
    let tag = state.article(12).unwrap();
    for view in [
        View::All,
        View::Home,
        View::Trends,
        View::Folder("Daily".into()),
    ] {
        assert!(!reader_domain::matches_view(star, &state, &view));
    }
    assert!(reader_domain::matches_view(star, &state, &View::Starred));
    assert!(reader_domain::matches_view(
        tag,
        &state,
        &View::Tag("Keep".into())
    ));
    assert!(reader_domain::matches_view(star, &state, &View::Feed(1)));
    assert!(reader_domain::matches_view(
        star,
        &state,
        &View::Search("feed:\"Source 1\"".into())
    ));
    assert!(reader_domain::matches_view(
        state.article(13).unwrap(),
        &state,
        &View::Search("private".into())
    ));
    assert!(!reader_domain::search_matches(star, &state, "folder:Daily"));
    let exported = reader_domain::parse_opml(&state.export_opml()).unwrap();
    assert_eq!(exported.len(), 2);
    assert!(exported
        .iter()
        .all(|entry| entry.url != state.feed(1).unwrap().url));
    let trends = state.trends(200);
    assert_eq!(
        (trends.subscriptions, trends.total, trends.starred),
        (2, 1, 1)
    );
}

#[test]
fn subscribing_to_an_archived_url_reuses_source_and_article_identity_with_annotations() {
    let mut state = library();
    state
        .apply(
            Action::RenameFeed {
                id: 1,
                title: "My source name".into(),
            },
            100,
        )
        .unwrap();
    state.apply(Action::RemoveFeed(1), 200).unwrap();
    let source = state.feed(1).unwrap().clone();
    let parsed = reader_domain::parse_feed(
        "<rss><channel><title>Publisher title</title><item><guid>item-11</guid><title>Updated publisher text</title><link>https://example.com/1/11</link><description>New body</description></item><item><guid>new</guid><title>Newly fetched</title><link>https://example.com/1/new</link></item></channel></rss>",
        &source.url, 300,
    ).unwrap();
    state
        .apply(
            Action::MergeFeed {
                url: source.url,
                feed: parsed,
            },
            300,
        )
        .unwrap();
    let active = state.feed(1).unwrap();
    assert!(!active.archived);
    assert_eq!(active.title, "My source name");
    assert!(active.folder_names().is_empty());
    assert_eq!(state.article(11).unwrap().title, "Updated publisher text");
    assert!(state.article(11).unwrap().starred);
    assert_eq!(state.article(12).unwrap().tags, ["Keep"]);
    assert_eq!(state.article(13).unwrap().note, "A private note");
    assert_eq!(
        state
            .articles
            .iter()
            .filter(|article| article.feed_id == 1)
            .count(),
        4
    );
    assert_eq!(state.feeds.iter().filter(|feed| feed.id == 1).count(), 1);
    assert_eq!(
        AppState::from_json(&state.to_json().unwrap()).unwrap(),
        state
    );
}

#[test]
fn archived_sources_reject_stale_subscription_edits_and_feed_error_mutations() {
    let mut state = library();
    state.apply(Action::RemoveFeed(1), 200).unwrap();
    let before = state.clone();
    assert!(state
        .apply(
            Action::SetFeedFolders {
                id: 1,
                folders: vec!["New".into()]
            },
            200
        )
        .is_err());
    assert!(state
        .apply(
            Action::RenameFeed {
                id: 1,
                title: "Stale title".into()
            },
            200
        )
        .is_err());
    assert!(state
        .apply(
            Action::PositionFeed {
                id: 1,
                target: 2,
                before: true,
                folder: None
            },
            200
        )
        .is_err());
    assert!(
        !state
            .apply(
                Action::FeedError {
                    url: before.feed(1).unwrap().url.clone(),
                    message: "Late failure".into()
                },
                200
            )
            .unwrap()
            .changed
    );
    assert_eq!(state, before);
}

#[test]
fn folder_validation_is_atomic_and_rename_moves_memberships_start_page_and_stream_settings() {
    let mut state = library();
    let display = StreamSettings {
        view_mode: ViewMode::List,
        sort: SortOrder::Oldest,
        unread_only: true,
    };
    state.stream_settings.insert("folder:Daily".into(), display);
    state.settings.start_page = View::Folder("Daily".into());
    let before = state.clone();
    for action in [
        Action::CreateFolder("  ".into()),
        Action::CreateFolder("Daily".into()),
        Action::CreateFolder("Bad\nname".into()),
        Action::RenameFolder {
            from: "Missing".into(),
            to: "New".into(),
        },
        Action::RenameFolder {
            from: "Daily".into(),
            to: "Shared".into(),
        },
        Action::DeleteFolder("Missing".into()),
    ] {
        assert!(state.apply(action, 200).is_err());
        assert_eq!(state, before);
    }
    assert!(
        !state
            .apply(
                Action::RenameFolder {
                    from: "Daily".into(),
                    to: " Daily ".into()
                },
                200
            )
            .unwrap()
            .changed
    );
    state
        .apply(
            Action::RenameFolder {
                from: "Daily".into(),
                to: "  Morning  ".into(),
            },
            200,
        )
        .unwrap();
    assert_eq!(state.settings.start_page, View::Folder("Morning".into()));
    assert_eq!(state.stream_settings.get("folder:Morning"), Some(&display));
    assert!(!state.stream_settings.contains_key("folder:Daily"));
    assert!(state
        .feeds
        .iter()
        .all(|feed| feed.folder_names() == ["Morning", "Shared"]));
    assert_eq!(state.articles, before.articles);
    state
        .apply(Action::DeleteFolder("Morning".into()), 200)
        .unwrap();
    assert_eq!(state.settings.start_page, View::Home);
    assert!(!state.stream_settings.contains_key("folder:Morning"));
    assert_eq!(state.feeds.len(), before.feeds.len());
    assert!(state
        .feeds
        .iter()
        .all(|feed| feed.folder_names() == ["Shared"]));
    assert_eq!(state.articles, before.articles);
    assert_eq!(
        AppState::from_json(&state.to_json().unwrap()).unwrap(),
        state
    );
}

#[test]
fn older_backups_without_archive_field_remain_active_subscriptions() {
    let state = library();
    let json = state.to_json().unwrap();
    assert!(!json.contains("archived"));
    let restored = AppState::from_json(&json).unwrap();
    assert!(restored.feeds.iter().all(|feed| !feed.archived));
    assert_eq!(restored, state);
}
