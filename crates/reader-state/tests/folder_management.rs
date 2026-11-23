use reader_domain::{Action, AppState, FeedConnection, SortOrder, View, ViewMode};
use reader_state::{AppViewModel, HostRequest, HostServices};
use std::{cell::RefCell, rc::Rc};

fn vm_with_saved_items() -> Rc<AppViewModel> {
    let mut state = AppState::sample(1000);
    let first = state.articles[0].id;
    let second = state.articles[6].id;
    state.apply(Action::ToggleStar(first), 1000).unwrap();
    state
        .apply(
            Action::SetNote {
                id: second,
                note: "Keep this thought".into(),
            },
            1000,
        )
        .unwrap();
    AppViewModel::new(state, 1000, Rc::new(|_| Ok(())))
}

#[test]
fn bulk_unsubscribe_is_one_undo_and_saved_articles_remain_available_outside_active_streams() {
    let vm = vm_with_saved_items();
    let removed = vm.feeds()[0].id;
    let other = vm.feeds()[1].id;
    vm.navigate(View::Feed(removed));
    let before = vm.state().clone();
    let saved = before
        .articles
        .iter()
        .find(|article| article.feed_id == removed && article.starred)
        .unwrap()
        .id;
    let active_total = before
        .articles
        .iter()
        .filter(|article| ![removed, other].contains(&article.feed_id))
        .count();
    let before_stars = vm.total_count(&View::Starred);
    assert!(vm.unsubscribe_many(vec![removed, other, removed]));
    assert!(vm.feed(removed).is_none() && vm.feed(other).is_none());
    assert!(vm.state().feed(removed).unwrap().archived);
    assert!(vm.state().feed(other).is_none());
    assert_eq!(vm.view(), View::All);
    assert_eq!(vm.total_count(&View::All), active_total);
    assert_eq!(vm.visible_len(), active_total);
    assert_eq!(vm.total_count(&View::Starred), before_stars);
    assert!(vm.status().contains("2 subscriptions removed"));
    assert!(vm.can_undo());
    vm.navigate(View::Starred);
    assert!(vm.visible_ids().contains(&saved));
    vm.navigate(View::Feed(removed));
    assert_eq!(vm.title(), before.feed(removed).unwrap().title);
    assert_eq!(vm.visible_len(), 2);
    AppState::from_json(&vm.export_json().unwrap()).unwrap();
    vm.undo();
    assert_eq!(*vm.state(), before);
    assert!(!vm.can_undo());
}

#[test]
fn stale_bulk_selection_changes_no_subscriptions_and_creates_no_undo_entry() {
    let vm = vm_with_saved_items();
    let first = vm.feeds()[0].id;
    let before = vm.export_json().unwrap();
    assert!(!vm.unsubscribe_many(vec![first, 999_999]));
    assert_eq!(vm.export_json().unwrap(), before);
    assert!(!vm.can_undo());
    assert!(vm.status().contains("Review the selection"));
}

#[test]
fn archiving_a_current_all_item_clears_its_cursor_without_losing_the_starred_copy() {
    let vm = vm_with_saved_items();
    vm.navigate(View::All);
    let item = vm.state().articles[0].clone();
    vm.select(Some(item.id));
    vm.unsubscribe(item.feed_id);
    assert_eq!(vm.selected(), None);
    assert_eq!(vm.cursor(), None);
    assert!(!vm.visible_ids().contains(&item.id));
    assert!(vm.article(item.id).unwrap().starred);
    let unread = vm.count(&View::All);
    vm.set_read(item.id, false);
    assert_eq!(
        vm.count(&View::All),
        unread,
        "an archived item never contributes to active unread counts"
    );
    vm.navigate(View::Starred);
    vm.select(Some(item.id));
    assert_eq!(vm.selected(), Some(item.id));
    assert_eq!(
        vm.count(&View::All),
        unread,
        "reading a saved copy updates the archived aggregate only"
    );
}

#[test]
fn context_refresh_targets_active_folder_members_without_navigating_or_double_requests() {
    let vm = vm_with_saved_items();
    let archived = vm.feeds()[0].id;
    vm.unsubscribe(archived);
    vm.navigate(View::Starred);
    let current = vm.visible_ids()[0];
    vm.select(Some(current));
    let expected: Vec<_> = vm
        .feeds()
        .into_iter()
        .filter(|feed| feed.in_folder("Technology"))
        .collect();
    let requests = Rc::new(RefCell::new(Vec::new()));
    let output = requests.clone();
    vm.set_services(HostServices {
        request: Rc::new(move |request| output.borrow_mut().push(request)),
    });
    assert!(vm.refresh_view(View::Folder("Technology".into())));
    assert!(!vm.refresh_view(View::Folder("Technology".into())));
    assert_eq!(vm.view(), View::Starred);
    assert_eq!(vm.selected(), Some(current));
    let requests = requests.borrow();
    assert_eq!(requests.len(), 1);
    match &requests[0] {
        HostRequest::Refresh {
            feeds, connection, ..
        } => {
            assert_eq!(feeds, &expected);
            assert!(!feeds.iter().any(|feed| feed.id == archived));
            assert_eq!(*connection, FeedConnection::Automatic);
        }
        _ => panic!("expected a scoped refresh"),
    }
}

#[test]
fn archived_startup_sources_are_skipped_but_explicit_subscription_reuses_saved_identity() {
    let vm = vm_with_saved_items();
    let source = vm.feeds()[0].clone();
    let article = vm.state().articles[0].clone();
    vm.unsubscribe(source.id);
    let requests = Rc::new(RefCell::new(Vec::new()));
    let output = requests.clone();
    vm.set_services(HostServices {
        request: Rc::new(move |request| output.borrow_mut().push(request)),
    });
    assert!(!vm.refresh_view(View::Feed(source.id)));
    assert!(requests.borrow().is_empty());
    assert!(vm.refresh_initial_feeds());
    assert!(
        matches!(&requests.borrow()[0], HostRequest::Refresh { feeds, .. } if feeds.iter().all(|feed| feed.id != source.id))
    );
    vm.finish_refresh();
    requests.borrow_mut().clear();
    vm.subscribe(source.url.clone());
    assert!(
        matches!(&requests.borrow()[0], HostRequest::Subscribe { url, .. } if url == &source.url)
    );
    vm.receive_feed(
        &source.url,
        "<rss><channel><title>Live source</title></channel></rss>",
    )
    .unwrap();
    assert_eq!(vm.feed(source.id).unwrap().id, source.id);
    assert_eq!(vm.article(article.id).unwrap(), article);
    assert!(vm.total_count(&View::All) > 0);
}

#[test]
fn folder_rename_uses_trimmed_route_and_preserves_reading_selection_and_stream_preferences() {
    let vm = vm_with_saved_items();
    vm.navigate(View::Folder("Technology".into()));
    vm.set_view_mode(ViewMode::List);
    vm.set_sort(SortOrder::Oldest);
    let selected = vm.visible_ids()[0];
    vm.select(Some(selected));
    let count = vm.total_count(&vm.view());
    let articles = vm.state().articles.clone();
    assert!(vm.rename_folder("Technology".into(), "  Morning reading  ".into()));
    assert_eq!(vm.view(), View::Folder("Morning reading".into()));
    assert_eq!(vm.selected(), Some(selected));
    assert_eq!(vm.cursor(), Some(selected));
    assert_eq!(vm.total_count(&vm.view()), count);
    assert_eq!(vm.state().settings.view_mode, ViewMode::List);
    assert_eq!(vm.state().settings.sort, SortOrder::Oldest);
    assert!(!vm.state().stream_settings.contains_key("folder:Technology"));
    assert!(vm
        .state()
        .stream_settings
        .contains_key("folder:Morning reading"));
    assert_eq!(vm.state().articles, articles);
    let before = vm.export_json().unwrap();
    assert!(!vm.rename_folder("Morning reading".into(), "Science".into()));
    assert_eq!(vm.export_json().unwrap(), before);
    assert_eq!(vm.view(), View::Folder("Morning reading".into()));
}

#[test]
fn deleting_a_folder_keeps_sources_and_articles_and_undo_recovers_its_preferences() {
    let vm = vm_with_saved_items();
    vm.navigate(View::Folder("Technology".into()));
    vm.set_view_mode(ViewMode::List);
    let mut settings = vm.state().settings.clone();
    settings.start_page = vm.view();
    vm.set_settings(settings);
    let before = vm.state().clone();
    assert!(vm.delete_folder("Technology".into()));
    assert_eq!(vm.view(), View::All);
    assert_eq!(vm.state().articles, before.articles);
    assert_eq!(vm.feeds().len(), before.feeds.len());
    assert!(!vm.folders().contains(&"Technology".into()));
    assert!(!vm.state().stream_settings.contains_key("folder:Technology"));
    assert_eq!(vm.state().settings.start_page, View::Home);
    vm.undo();
    assert_eq!(vm.state().articles, before.articles);
    assert_eq!(vm.state().feeds, before.feeds);
    assert_eq!(vm.state().stream_settings, before.stream_settings);
    assert_eq!(vm.state().settings.start_page, before.settings.start_page);
}

#[test]
fn opml_reactivation_is_undoable_without_losing_archived_annotations() {
    let vm = vm_with_saved_items();
    let source = vm.feeds()[0].clone();
    vm.unsubscribe(source.id);
    let archived = vm.state().clone();
    let opml = format!("<opml><body><outline text=\"Imported folder\"><outline text=\"Imported name\" xmlUrl=\"{}\"/></outline></body></opml>", source.url);
    assert_eq!(vm.import_opml(&opml).unwrap(), 1);
    assert!(vm.feed(source.id).unwrap().in_folder("Imported folder"));
    assert_eq!(vm.state().articles, archived.articles);
    vm.undo();
    assert!(vm.feed(source.id).is_none());
    assert!(vm.state().feed(source.id).unwrap().archived);
    assert_eq!(vm.state().articles, archived.articles);
    AppState::from_json(&vm.export_json().unwrap()).unwrap();
}

#[test]
fn successful_subscription_changes_do_not_mask_a_storage_failure() {
    let state = AppState::sample(1000);
    let vm = AppViewModel::new(state, 1000, Rc::new(|_| Err("disk full".into())));
    let id = vm.feeds()[0].id;
    assert!(vm.unsubscribe_many(vec![id]));
    assert!(vm
        .status()
        .contains("Could not save your library: disk full"));
    assert!(vm.delete_folder("Technology".into()));
    assert!(vm
        .status()
        .contains("Could not save your library: disk full"));
    assert!(vm.flush().is_err());
}
