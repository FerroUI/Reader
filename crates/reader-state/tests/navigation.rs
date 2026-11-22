use reader_domain::{AppState, SortOrder, View, ViewMode};
use reader_state::AppViewModel;
use std::rc::Rc;

#[test]
fn stream_presentation_is_independent_and_survives_backup_restore() {
    let vm = AppViewModel::new(AppState::sample(1000), 1000, Rc::new(|_| Ok(())));
    let first = vm.feeds()[0].id;
    let second = vm.feeds()[1].id;
    vm.navigate(View::Feed(first));
    vm.set_view_mode(ViewMode::List);
    vm.set_sort(SortOrder::Oldest);
    vm.set_unread_only(true);
    vm.navigate(View::Feed(second));
    assert_eq!(vm.state().settings.view_mode, ViewMode::Expanded);
    assert_eq!(vm.state().settings.sort, SortOrder::Newest);
    assert!(!vm.state().settings.unread_only);
    vm.navigate(View::Feed(first));
    assert_eq!(vm.state().settings.view_mode, ViewMode::List);
    assert_eq!(vm.state().settings.sort, SortOrder::Oldest);
    assert!(vm.state().settings.unread_only);
    let restored = AppViewModel::new(
        AppState::from_json(&vm.export_json().unwrap()).unwrap(),
        1000,
        Rc::new(|_| Ok(())),
    );
    restored.navigate(View::Feed(first));
    assert_eq!(restored.state().settings.view_mode, ViewMode::List);
    assert!(restored.state().settings.unread_only);
}

#[test]
fn scanning_does_not_open_or_mark_articles_read_and_keep_unread_blocks_scroll() {
    let vm = AppViewModel::new(AppState::sample(1000), 1000, Rc::new(|_| Ok(())));
    vm.navigate(View::All);
    let unread = vm.count(&View::All);
    vm.scan_article(1);
    let id = vm.cursor().unwrap();
    assert_eq!(vm.selected(), None);
    assert_eq!(vm.count(&View::All), unread);
    vm.select(Some(id));
    assert_eq!(vm.count(&View::All), unread - 1);
    vm.keep_unread(id);
    vm.select_from_scroll(id);
    assert!(!vm.article(id).unwrap().read);
    assert!(vm.is_kept_unread(id));
    vm.select(Some(id));
    assert!(vm.article(id).unwrap().read);
    assert!(!vm.is_kept_unread(id));
}

#[test]
fn multi_folder_opml_import_preserves_one_subscription_and_undo_restores_memberships() {
    let vm = AppViewModel::new(AppState::new(), 1000, Rc::new(|_| Ok(())));
    let opml = r#"<opml><body><outline text="Science"><outline text="Feed" xmlUrl="https://example.com/feed"/></outline><outline text="Favorites"><outline text="Feed" xmlUrl="https://example.com/feed"/></outline></body></opml>"#;
    assert_eq!(vm.import_opml(opml).unwrap(), 1);
    let feed = vm.feeds()[0].clone();
    assert_eq!(feed.folder_names(), vec!["Favorites", "Science"]);
    let exported = reader_domain::parse_opml(&vm.export_opml()).unwrap();
    assert_eq!(exported.len(), 1);
    assert_eq!(exported[0].folders, vec!["Favorites", "Science"]);
    vm.set_feed_folders(feed.id, vec!["Science".into()]);
    vm.undo();
    assert_eq!(
        vm.feed(feed.id).unwrap().folder_names(),
        vec!["Favorites", "Science"]
    );
    vm.import_opml(&opml.replace("Favorites", "Reading"))
        .unwrap();
    assert!(vm.feed(feed.id).unwrap().in_folder("Reading"));
    vm.undo();
    assert_eq!(
        vm.feed(feed.id).unwrap().folder_names(),
        vec!["Favorites", "Science"]
    );
    vm.delete_folder("Favorites".into());
    assert_eq!(vm.feeds().len(), 1);
    assert_eq!(vm.feed(feed.id).unwrap().folder_names(), vec!["Science"]);
}

#[test]
fn start_page_is_honored_and_background_feed_updates_do_not_navigate() {
    let mut state = AppState::sample(1000);
    state.settings.start_page = View::Starred;
    let vm = AppViewModel::new(state, 1000, Rc::new(|_| Ok(())));
    assert_eq!(vm.view(), View::Starred);
    let xml = "<rss><channel><title>Feed</title><item><guid>one</guid><title>New</title></item></channel></rss>";
    vm.receive_feed("https://example.com/feed", xml).unwrap();
    assert_eq!(vm.view(), View::Starred);
    let mut settings = vm.state().settings.clone();
    settings.start_page = View::Feed(999_999);
    vm.set_settings(settings);
    let restored = AppViewModel::new(
        AppState::from_json(&vm.export_json().unwrap()).unwrap(),
        1000,
        Rc::new(|_| Ok(())),
    );
    assert_eq!(restored.view(), View::Home);
}

#[test]
fn redirected_feed_links_use_effective_url_without_changing_subscription_identity() {
    let vm = AppViewModel::new(AppState::new(), 1000, Rc::new(|_| Ok(())));
    let original = "https://example.com/feed";
    let xml = "<rss><channel><title>Feed</title><link>../</link><item><guid>one</guid><title>One</title><link>posts/one</link></item></channel></rss>";
    let id = vm.receive_feed(original, xml).unwrap();
    let refreshed = vm
        .receive_feed_from(original, xml, "https://cdn.example.org/news/rss.xml")
        .unwrap();
    assert_eq!(refreshed, id);
    assert_eq!(vm.feed(id).unwrap().url, original);
    assert_eq!(vm.state().articles.len(), 1);
    assert_eq!(
        vm.state().articles[0].url,
        "https://cdn.example.org/news/posts/one"
    );
    assert_eq!(
        vm.state().articles[0].content_base,
        "https://cdn.example.org/news/rss.xml"
    );
    assert_eq!(vm.view(), View::Home);
    assert!(vm
        .receive_feed_from(original, xml, "javascript:unsafe")
        .is_err());
    assert_eq!(vm.state().articles.len(), 1);
}
