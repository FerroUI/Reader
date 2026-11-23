use reader_domain::{Action, AppState, FullArticle, View, ViewMode};
use reader_state::AppViewModel;
use std::{cell::Cell, rc::Rc};

const URL: &str = "https://example.com/feed";
const ORIGINAL: &str = "<rss><channel><title>Original feed</title><item><guid>one</guid><title>Original</title><link>https://example.com/one</link><description>Old body</description></item><item><guid>older</guid><title>Older</title><description>Archived issue</description></item></channel></rss>";
const UPDATED: &str = "<rss><channel><title>Publisher title</title><item><guid>one</guid><title>Updated</title><link>https://example.com/one</link><description>New body</description></item><item><guid>new</guid><title>New article</title><description>New body</description></item></channel></rss>";

fn vm() -> Rc<AppViewModel> {
    AppViewModel::new(AppState::new(), 100, Rc::new(|_| Ok(())))
}
fn article(vm: &AppViewModel, guid: &str) -> u64 {
    vm.state()
        .articles
        .iter()
        .find(|article| article.guid == guid)
        .unwrap()
        .id
}
fn assert_roundtrip(vm: &AppViewModel) -> AppState {
    let restored = AppState::from_json(&vm.export_json().unwrap()).unwrap();
    assert_eq!(restored.feeds, vm.state().feeds);
    assert_eq!(restored.articles, vm.state().articles);
    for feed in &restored.feeds {
        assert_eq!(restored.feed(feed.id), Some(feed));
    }
    for article in &restored.articles {
        assert_eq!(restored.article(article.id), Some(article));
    }
    restored
}

#[test]
fn resubscribe_then_undo_reconciles_identities_and_older_commands() {
    let vm = vm();
    let old_feed = vm.receive_feed(URL, ORIGINAL).unwrap();
    let old_article = article(&vm, "one");
    let older = article(&vm, "older");
    vm.rename_feed(old_feed, "My custom title".into());
    vm.set_read(old_article, true);
    vm.unsubscribe(old_feed);
    assert!(
        vm.state().feed(old_feed).is_none(),
        "read-only items do not keep an archive"
    );
    vm.set_now(200);
    let current_feed = vm.receive_feed(URL, UPDATED).unwrap();
    let current_article = article(&vm, "one");
    let newest = article(&vm, "new");
    assert_ne!(current_feed, old_feed);
    assert_ne!(current_article, old_article);
    let current_full = FullArticle::new(
        "<p>Latest publisher copy</p>".into(),
        "https://example.com/one",
        "https://example.com/one",
        200,
    )
    .unwrap();
    vm.dispatch(Action::CacheFullArticle {
        id: current_article,
        content: current_full.clone(),
    });
    vm.undo();
    assert_eq!(vm.feeds().len(), 1);
    assert_eq!(
        vm.feeds()[0].id,
        current_feed,
        "live subscription identity stays stable"
    );
    assert_eq!(vm.feeds()[0].title, "My custom title");
    assert_eq!(vm.feeds()[0].fetched_at, 200);
    assert!(vm.article(old_article).is_none());
    let retained = vm.article(current_article).unwrap();
    assert_eq!(retained.title, "Updated");
    assert_eq!(retained.content, "New body");
    assert!(retained.read);
    assert_eq!(retained.read_at, Some(100));
    assert_eq!(retained.full_content.as_deref(), Some(&current_full));
    assert_eq!(vm.article(older).unwrap().feed_id, current_feed);
    assert!(vm.article(newest).is_some());
    assert_eq!(vm.total_count(&View::All), 3);
    assert_eq!(vm.count(&View::All), 2);
    assert_roundtrip(&vm);
    // Earlier entries referenced the removed identities. They must remain useful.
    vm.undo();
    assert!(!vm.article(current_article).unwrap().read);
    assert_eq!(vm.count(&View::All), 3);
    vm.undo();
    assert_eq!(vm.feed(current_feed).unwrap().title, "Original feed");
    assert!(!vm.can_undo());
    assert_roundtrip(&vm);
}

#[test]
fn archive_undo_restores_missing_items_and_memberships_without_duplicates() {
    let vm = vm();
    let id = vm.receive_feed(URL, ORIGINAL).unwrap();
    let protected = article(&vm, "one");
    let ordinary = article(&vm, "older");
    vm.toggle_star(protected);
    vm.set_tags(protected, vec!["Saved".into()]);
    vm.set_note(protected, "My saved note".into());
    vm.set_feed_folders(id, vec!["Reading".into()]);
    vm.unsubscribe(id);
    assert!(vm.state().feed(id).unwrap().archived);
    assert!(vm.feed(id).is_none());
    assert!(vm.article(ordinary).is_none());
    assert!(vm.article(protected).is_some());
    assert_eq!(vm.receive_feed(URL, UPDATED).unwrap(), id);
    let newest = article(&vm, "new");
    vm.undo();
    assert!(!vm.state().feed(id).unwrap().archived);
    assert!(vm.feed(id).unwrap().in_folder("Reading"));
    assert_eq!(vm.state().articles.len(), 3);
    assert_eq!(
        vm.state()
            .articles
            .iter()
            .filter(|article| article.guid == "one")
            .count(),
        1
    );
    let retained = vm.article(protected).unwrap();
    assert_eq!(retained.title, "Updated");
    assert!(retained.starred);
    assert_eq!(retained.tags, ["Saved"]);
    assert_eq!(retained.note, "My saved note");
    assert!(vm.article(ordinary).is_some());
    assert!(vm.article(newest).is_some());
    assert_eq!(vm.total_count(&View::Folder("Reading".into())), 3);
    assert_roundtrip(&vm);
}

#[test]
fn resubscription_rebinds_saved_start_page_and_stream_preferences() {
    let seed = vm();
    let old = seed.receive_feed(URL, ORIGINAL).unwrap();
    let mut state = assert_roundtrip(&seed);
    state.settings.start_page = View::Feed(old);
    let display = reader_domain::StreamSettings {
        view_mode: ViewMode::List,
        unread_only: true,
        ..Default::default()
    };
    state
        .stream_settings
        .insert(View::Feed(old).storage_key(), display);
    let vm = AppViewModel::new(state, 100, Rc::new(|_| Ok(())));
    vm.unsubscribe(old);
    let current = vm.receive_feed(URL, UPDATED).unwrap();
    assert_ne!(old, current);
    vm.undo();
    assert_eq!(vm.state().settings.start_page, View::Feed(current));
    assert!(!vm
        .state()
        .stream_settings
        .contains_key(&View::Feed(old).storage_key()));
    assert_eq!(
        vm.state()
            .stream_settings
            .get(&View::Feed(current).storage_key()),
        Some(&display)
    );
    let offline = AppViewModel::new(assert_roundtrip(&vm), 300, Rc::new(|_| Ok(())));
    assert_eq!(offline.view(), View::Feed(current));
    assert_eq!(offline.state().settings.view_mode, ViewMode::List);
    assert!(offline.state().settings.unread_only);
}

#[test]
fn bulk_unsubscribe_is_one_undo_preserving_document_order_and_background_content() {
    let vm = vm();
    let a = vm.receive_feed(URL, ORIGINAL).unwrap();
    let b = vm
        .receive_feed("https://other.example/feed", ORIGINAL)
        .unwrap();
    let c = vm
        .receive_feed("https://third.example/feed", ORIGINAL)
        .unwrap();
    let mut initial = AppState::from_json(&vm.export_json().unwrap()).unwrap();
    initial.articles.rotate_left(3);
    initial.settings.start_page = View::Feed(b);
    initial.normalize().unwrap();
    let before = initial.clone();
    let vm = AppViewModel::new(initial, 100, Rc::new(|_| Ok(())));
    vm.dispatch(Action::RemoveFeeds(vec![b, a, b]));
    assert_eq!(vm.state().settings.start_page, View::Home);
    assert_eq!(
        vm.feeds().iter().map(|feed| feed.id).collect::<Vec<_>>(),
        [c]
    );
    vm.receive_feed("https://third.example/feed", UPDATED)
        .unwrap();
    vm.undo();
    assert!(!vm.can_undo(), "all selected feeds restore in one undo");
    assert_eq!(
        vm.feeds().iter().map(|feed| feed.id).collect::<Vec<_>>(),
        [a, b, c]
    );
    assert_eq!(vm.state().settings.start_page, View::Feed(b));
    let restored = assert_roundtrip(&vm);
    let original_ids: Vec<_> = before.articles.iter().map(|article| article.id).collect();
    let restored_ids: Vec<_> = restored
        .articles
        .iter()
        .filter(|article| original_ids.contains(&article.id))
        .map(|article| article.id)
        .collect();
    assert_eq!(restored_ids, original_ids);
    assert!(restored
        .articles
        .iter()
        .any(|article| article.feed_id == c && article.guid == "new"));
}

#[test]
fn folder_rename_and_delete_undo_restore_stream_preferences_and_start_page() {
    let mut state = AppState::new();
    state
        .apply(Action::CreateFolder("Reading".into()), 0)
        .unwrap();
    state.settings.start_page = View::Folder("Reading".into());
    let stream = reader_domain::StreamSettings {
        view_mode: ViewMode::List,
        ..Default::default()
    };
    state
        .stream_settings
        .insert("folder:Reading".into(), stream);
    let vm = AppViewModel::new(state, 100, Rc::new(|_| Ok(())));
    vm.rename_folder("Reading".into(), "  Saved  ".into());
    assert_eq!(vm.state().settings.start_page, View::Folder("Saved".into()));
    vm.undo();
    assert_eq!(vm.view(), View::Folder("Reading".into()));
    assert_eq!(
        vm.state().settings.start_page,
        View::Folder("Reading".into())
    );
    assert_eq!(
        vm.state().stream_settings.get("folder:Reading"),
        Some(&stream)
    );
    assert!(!vm.state().stream_settings.contains_key("folder:Saved"));
    vm.delete_folder("Reading".into());
    assert_eq!(vm.state().settings.start_page, View::Home);
    vm.undo();
    assert_eq!(
        vm.state().settings.start_page,
        View::Folder("Reading".into())
    );
    assert_eq!(
        vm.state().stream_settings.get("folder:Reading"),
        Some(&stream)
    );
    assert_roundtrip(&vm);
}

#[test]
fn undo_folder_rename_preserves_reading_selection_but_respects_later_navigation() {
    let vm = vm();
    let feed = vm.receive_feed(URL, ORIGINAL).unwrap();
    vm.set_feed_folders(feed, vec!["Reading".into()]);
    vm.navigate(View::Folder("Reading".into()));
    let selected = article(&vm, "one");
    vm.select(Some(selected));
    let visible = vm.visible_ids();
    vm.rename_folder("Reading".into(), "Saved".into());
    assert_eq!(vm.view(), View::Folder("Saved".into()));
    vm.undo();
    assert_eq!(vm.view(), View::Folder("Reading".into()));
    assert_eq!(vm.selected(), Some(selected));
    assert_eq!(vm.cursor(), Some(selected));
    assert_eq!(vm.visible_ids(), visible);
    vm.rename_folder("Reading".into(), "Saved".into());
    vm.navigate(View::Starred);
    vm.undo();
    assert_eq!(
        vm.view(),
        View::Starred,
        "undo cannot take the user away from a later destination"
    );
    assert!(vm.folders().contains(&"Reading".into()));
    assert_roundtrip(&vm);
}

#[test]
fn undo_removing_the_current_folder_or_tag_rebuilds_the_all_items_projection() {
    let vm = vm();
    vm.receive_feed(URL, ORIGINAL).unwrap();
    vm.create_folder("Temporary".into());
    vm.navigate(View::Folder("Temporary".into()));
    assert!(vm.visible_ids().is_empty());
    vm.undo();
    assert_eq!(vm.view(), View::All);
    assert_eq!(vm.visible_ids().len(), 2);
    let id = article(&vm, "one");
    vm.set_tags(id, vec!["Temporary".into()]);
    vm.navigate(View::Tag("Temporary".into()));
    assert_eq!(vm.visible_ids(), [id]);
    vm.undo();
    assert_eq!(vm.view(), View::All);
    assert_eq!(
        vm.visible_ids().len(),
        2,
        "the fallback must include unchanged articles outside the previous tag"
    );
    assert_roundtrip(&vm);
}

#[test]
fn undo_save_failure_is_not_hidden_by_success_feedback() {
    let fail = Rc::new(Cell::new(false));
    let failure = fail.clone();
    let vm = AppViewModel::new(
        AppState::new(),
        100,
        Rc::new(move |_| {
            if failure.get() {
                Err("disk is full".into())
            } else {
                Ok(())
            }
        }),
    );
    vm.receive_feed(URL, ORIGINAL).unwrap();
    vm.toggle_star(article(&vm, "one"));
    fail.set(true);
    vm.undo();
    assert!(vm.status().contains("Could not save your library"));
    assert!(vm.status().contains("disk is full"));
    assert!(vm.flush().is_err());
    assert_roundtrip(&vm);
}
