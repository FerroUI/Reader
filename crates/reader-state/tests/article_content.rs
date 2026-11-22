use ferroui_reactive::ObservableExt;
use ferroui_reactiveui::IReactiveNotifyPropertyChanged;
use reader_domain::{AppState, FeedConnection, View};
use reader_state::{AppViewModel, HostRequest, HostServices};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

const XML: &str = "<rss><channel><title>Feed</title><item><guid>post</guid><title>Article</title><link>https://example.com/post#section</link><description>Feed summary</description></item></channel></rss>";

#[derive(Clone)]
struct Request {
    id: u64,
    request_id: u64,
    generation: u64,
    url: String,
}
fn request(requests: &RefCell<Vec<HostRequest>>) -> Request {
    requests
        .borrow()
        .iter()
        .rev()
        .find_map(|request| match request {
            HostRequest::FetchArticle {
                id,
                request_id,
                generation,
                url,
                ..
            } => Some(Request {
                id: *id,
                request_id: *request_id,
                generation: *generation,
                url: url.clone(),
            }),
            _ => None,
        })
        .unwrap()
}
fn complete(vm: &AppViewModel, r: &Request) -> Result<bool, String> {
    vm.receive_article(
        r.id,
        r.request_id,
        r.generation,
        &r.url,
        "<p>Complete article body.</p>",
        "https://example.com/post/",
    )
}
fn harness() -> (Rc<AppViewModel>, Rc<RefCell<Vec<HostRequest>>>, u64) {
    let vm = AppViewModel::new(AppState::new(), 100, Rc::new(|_| Ok(())));
    vm.receive_feed("https://example.com/feed", XML).unwrap();
    let id = vm.state().articles[0].id;
    let events = Rc::new(RefCell::new(Vec::new()));
    let capture = events.clone();
    vm.set_services(HostServices {
        request: Rc::new(move |r| capture.borrow_mut().push(r)),
    });
    (vm, events, id)
}

#[test]
fn article_loading_is_explicit_deduplicated_and_uses_the_saved_connection_policy() {
    let (vm, requests, id) = harness();
    vm.navigate(View::All);
    vm.select(Some(id));
    assert!(
        requests
            .borrow()
            .iter()
            .all(|r| !matches!(r, HostRequest::FetchArticle { .. })),
        "ordinary reading never downloads the publisher page"
    );
    let mut settings = vm.state().settings.clone();
    settings.feed_connection = FeedConnection::Custom;
    settings.feed_proxy = "https://trusted.example/?url={url}".into();
    vm.set_settings(settings);
    assert!(vm.load_full_article(id, false));
    assert!(vm.article_loading(id));
    assert!(!vm.load_full_article(id, false));
    let request = requests
        .borrow()
        .iter()
        .find(|r| matches!(r, HostRequest::FetchArticle { .. }))
        .unwrap()
        .clone();
    let json = serde_json::to_value(request).unwrap();
    assert_eq!(json["kind"], "fetchArticle");
    assert_eq!(json["connection"], "Custom");
    assert_eq!(json["proxy"], "https://trusted.example/?url={url}");
    assert_eq!(json["url"], "https://example.com/post#section");
    assert_eq!(
        requests
            .borrow()
            .iter()
            .filter(|r| matches!(r, HostRequest::FetchArticle { .. }))
            .count(),
        1
    );
}

#[test]
fn saved_content_opens_offline_and_results_do_not_navigate_or_change_annotations() {
    let (vm, requests, id) = harness();
    vm.toggle_star(id);
    vm.set_note(id, "My note".into());
    vm.set_tags(id, vec!["Saved".into()]);
    vm.navigate(View::Starred);
    assert!(vm.load_full_article(id, false));
    let pending = request(&requests);
    vm.navigate(View::Home);
    assert!(complete(&vm, &pending).unwrap());
    assert!(!vm.article_loading(id));
    assert_eq!(vm.view(), View::Home);
    assert_eq!(vm.selected(), None);
    let article = vm.article(id).unwrap();
    assert!(!article.read);
    assert!(article.starred);
    assert_eq!(article.note, "My note");
    assert_eq!(article.tags, ["Saved"]);
    assert_eq!(article.content, "Feed summary");
    assert_eq!(article.reading_content(), "<p>Complete article body.</p>");
    vm.show_feed_content(id);
    assert_eq!(vm.article(id).unwrap().reading_content(), "Feed summary");
    requests.borrow_mut().clear();
    vm.show_full_content(id);
    assert!(!vm.article_loading(id));
    assert!(
        requests
            .borrow()
            .iter()
            .all(|r| !matches!(r, HostRequest::FetchArticle { .. }))
    );
    vm.print_article(id);
    assert!(requests.borrow().iter().any(|request| matches!(request,
        HostRequest::Print { html, base_url, .. }
            if html.contains("Complete article") && base_url == "https://example.com/post/"
    )));
    let restored = AppState::from_json(&vm.export_json().unwrap()).unwrap();
    let offline = AppViewModel::new(restored, 300, Rc::new(|_| Ok(())));
    assert!(!offline.load_full_article(id, false));
    assert!(offline.article_error(id).is_none());
    assert!(
        offline
            .article(id)
            .unwrap()
            .reading_content()
            .contains("Complete article")
    );
}

#[test]
fn cancellation_and_retry_reject_superseded_successes_and_errors() {
    let (vm, requests, id) = harness();
    vm.load_full_article(id, false);
    let old = request(&requests);
    vm.show_feed_content(id);
    assert!(!vm.article_loading(id));
    assert!(requests.borrow().iter().any(
        |r| matches!(r,HostRequest::CancelArticle{request_id} if *request_id==old.request_id)
    ));
    vm.load_full_article(id, false);
    let next = request(&requests);
    assert_ne!(old.request_id, next.request_id);
    assert!(!complete(&vm, &old).unwrap());
    assert!(!vm.article_error_response(id, old.request_id, old.generation, &old.url, "late error"));
    assert!(vm.article_loading(id));
    assert!(
        vm.receive_article(
            id,
            next.request_id,
            next.generation,
            &next.url,
            "",
            &next.url
        )
        .is_err()
    );
    assert!(!vm.article_loading(id));
    assert!(vm.article_error(id).unwrap().contains("readable"));
    assert_eq!(vm.article(id).unwrap().reading_content(), "Feed summary");
    vm.load_full_article(id, false);
    let retry = request(&requests);
    assert!(complete(&vm, &retry).unwrap());
    assert!(!complete(&vm, &retry).unwrap());
    assert!(vm.article_error(id).is_none());
    vm.load_full_article(id, true);
    let reload = request(&requests);
    assert!(vm.article_error_response(
        id,
        reload.request_id,
        reload.generation,
        &reload.url,
        "Publisher unavailable"
    ));
    assert!(
        vm.article(id).unwrap().has_full_content(),
        "failed refresh retains the cached full article"
    );
}

#[test]
fn library_replacement_and_snapshot_undo_cancel_even_identical_article_identifiers() {
    let (vm, requests, id) = harness();
    let backup = vm.export_json().unwrap();
    vm.load_full_article(id, false);
    let old = request(&requests);
    vm.import_json(&backup).unwrap();
    assert!(!complete(&vm, &old).unwrap());
    assert!(!vm.article_loading(id));
    vm.load_full_article(id, false);
    let imported = request(&requests);
    vm.undo();
    assert_ne!(imported.generation, vm.library_generation());
    assert!(!complete(&vm, &imported).unwrap());
    assert!(!vm.article_loading(id));
    assert!(!vm.article(id).unwrap().has_full_content());
}

#[test]
fn unsubscribe_undo_and_changed_publisher_links_do_not_accept_old_results() {
    let (vm, requests, id) = harness();
    let feed_id = vm.article(id).unwrap().feed_id;
    vm.load_full_article(id, false);
    let removed = request(&requests);
    vm.unsubscribe(feed_id);
    vm.undo();
    assert!(vm.article(id).is_some());
    assert!(!complete(&vm, &removed).unwrap());
    vm.load_full_article(id, false);
    let changed = request(&requests);
    vm.receive_feed(
        "https://example.com/feed",
        &XML.replace("post#section", "replacement"),
    )
    .unwrap();
    assert!(!vm.article_loading(id));
    assert!(!complete(&vm, &changed).unwrap());
    assert_eq!(
        vm.article(id).unwrap().url,
        "https://example.com/replacement"
    );
    vm.load_full_article(id, false);
    let failed = request(&requests);
    vm.article_error_response(
        id,
        failed.request_id,
        failed.generation,
        &failed.url,
        "Old URL failed",
    );
    vm.receive_feed("https://example.com/feed", XML).unwrap();
    assert!(
        vm.article_error(id).is_none(),
        "a changed URL cannot inherit the previous URL's error"
    );
}

#[test]
fn invalid_hosts_and_links_fail_without_issuing_a_request() {
    let (vm, requests, id) = harness();
    assert!(!vm.load_full_article(u64::MAX, false));
    let state = AppState::from_json(&vm.export_json().unwrap()).unwrap();
    let no_host = AppViewModel::new(state, 100, Rc::new(|_| Ok(())));
    assert!(!no_host.load_full_article(id, false));
    assert!(no_host.article_error(id).is_some());
    vm.receive_feed(
        "https://example.com/feed",
        &XML.replace("https://example.com/post#section", "javascript:bad()"),
    )
    .unwrap();
    assert!(!vm.load_full_article(id, false));
    assert!(vm.article_error(id).unwrap().contains("valid HTTP"));
    assert!(
        requests
            .borrow()
            .iter()
            .all(|r| !matches!(r, HostRequest::FetchArticle { .. }))
    );
}

#[test]
fn reentrant_backup_restore_discards_an_already_queued_article_completion() {
    let (vm, requests, id) = harness();
    let backup = vm.export_json().unwrap();
    vm.load_full_article(id, false);
    let pending = request(&requests);
    let once = Cell::new(false);
    let weak = Rc::downgrade(&vm);
    let subscription = vm.changed().subscribe_next(move |_| {
        if once.replace(true) {
            return;
        }
        let vm = weak.upgrade().unwrap();
        assert!(complete(&vm, &pending).unwrap());
        vm.import_json(&backup).unwrap();
    });
    vm.toggle_star(id);
    subscription.dispose();
    assert!(!vm.article(id).unwrap().has_full_content());
    assert!(!vm.article_loading(id));
}
