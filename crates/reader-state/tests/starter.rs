use reader_domain::{Action, AppState, FeedConnection, View};
use reader_state::{AppViewModel, HostRequest, HostServices};
use std::{cell::RefCell, rc::Rc};

fn recording(state: AppState) -> (Rc<AppViewModel>, Rc<RefCell<Vec<HostRequest>>>) {
    let requests = Rc::new(RefCell::new(Vec::new()));
    let output = requests.clone();
    let vm = AppViewModel::new(state, 2000, Rc::new(|_| Ok(())));
    vm.set_services(HostServices {
        request: Rc::new(move |request| output.borrow_mut().push(request)),
    });
    (vm, requests)
}

#[test]
fn initial_download_batches_only_pending_feeds_and_keeps_the_current_location() {
    let mut state = AppState::with_default_feeds();
    state.feeds[0].fetched_at = 1000;
    state.feeds[1].error = Some("Previously offline".into());
    state.settings.start_page = View::Feed(state.feeds[0].id);
    state.settings.feed_connection = FeedConnection::Custom;
    state.settings.feed_proxy = "https://trusted.example/?url={url}".into();
    let location = state.settings.start_page.clone();
    let expected: Vec<_> = state.feeds[1..].to_vec();
    let (vm, requests) = recording(state);
    assert!(vm.refresh_initial_feeds());
    assert!(
        !vm.refresh_initial_feeds(),
        "startup and online events cannot overlap refreshes"
    );
    assert!(vm.is_refreshing());
    assert_eq!(vm.view(), location);
    let requests = requests.borrow();
    assert_eq!(requests.len(), 1);
    match &requests[0] {
        HostRequest::Refresh {
            feeds,
            proxy,
            connection,
            ..
        } => {
            assert_eq!(*feeds, expected);
            assert_eq!(proxy, "https://trusted.example/?url={url}");
            assert_eq!(*connection, FeedConnection::Custom);
        }
        _ => panic!("expected one batch refresh"),
    }
}

#[test]
fn empty_completed_and_uninitialized_hosts_do_not_start_initial_downloads() {
    let (empty, requests) = recording(AppState::new());
    assert!(!empty.refresh_initial_feeds());
    empty.refresh();
    assert!(!empty.is_refreshing());
    assert!(requests.borrow().is_empty());
    let mut complete = AppState::with_default_feeds();
    for feed in &mut complete.feeds {
        feed.fetched_at = 1000;
    }
    let (complete, requests) = recording(complete);
    assert!(!complete.refresh_initial_feeds());
    assert!(requests.borrow().is_empty());
    let vm = AppViewModel::new(AppState::with_default_feeds(), 2000, Rc::new(|_| Ok(())));
    assert!(!vm.refresh_initial_feeds());
    assert!(!vm.is_refreshing());
}

#[test]
fn startup_persists_subscriptions_before_a_network_result_can_arrive() {
    let events = Rc::new(RefCell::new(Vec::new()));
    let saved = Rc::new(RefCell::new(String::new()));
    let saves = saved.clone();
    let writes = events.clone();
    let vm = AppViewModel::new(
        AppState::with_default_feeds(),
        2000,
        Rc::new(move |json| {
            writes.borrow_mut().push("save");
            *saves.borrow_mut() = json.into();
            Ok(())
        }),
    );
    let requests = events.clone();
    vm.set_services(HostServices {
        request: Rc::new(move |request| {
            if matches!(request, HostRequest::Refresh { .. }) {
                requests.borrow_mut().push("refresh");
            }
        }),
    });
    vm.persist_initial_state().unwrap();
    assert!(vm.refresh_initial_feeds());
    assert_eq!(*events.borrow(), ["save", "refresh"]);
    let persisted = AppState::from_json(&saved.borrow()).unwrap();
    assert_eq!(persisted.feeds.len(), 6);
    assert!(persisted.articles.is_empty());
}

#[test]
fn retries_skip_completed_feeds_and_never_reintroduce_removed_defaults() {
    let mut state = AppState::with_default_feeds();
    let removed = state.feeds[0].id;
    state.apply(Action::RemoveFeed(removed), 1000).unwrap();
    let completed_url = state.feeds[0].url.clone();
    let (vm, requests) = recording(state);
    assert!(vm.refresh_initial_feeds());
    vm.receive_feed(
        &completed_url,
        "<rss><channel><title>Empty publisher feed</title></channel></rss>",
    )
    .unwrap();
    vm.finish_refresh();
    requests.borrow_mut().clear();
    assert!(vm.refresh_initial_feeds());
    let requests = requests.borrow();
    let feeds = match requests.last().unwrap() {
        HostRequest::Refresh { feeds, .. } => feeds,
        _ => panic!(),
    };
    assert_eq!(feeds.len(), 4);
    assert!(feeds
        .iter()
        .all(|feed| feed.id != removed && feed.url != completed_url));
    assert_eq!(vm.state().feeds.len(), 5);
    assert!(vm.state().feed(removed).is_none());
}
