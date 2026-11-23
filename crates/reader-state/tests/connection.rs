use reader_domain::{AppState, FeedConnection, View};
use reader_state::{AppViewModel, HostRequest, HostServices};
use std::{cell::RefCell, rc::Rc};

#[test]
fn subscriptions_and_refresh_send_the_persisted_policy_with_normalized_addresses() {
    let requests = Rc::new(RefCell::new(Vec::new()));
    let received = requests.clone();
    let vm = AppViewModel::new(AppState::sample(1000), 1000, Rc::new(|_| Ok(())));
    vm.set_services(HostServices {
        request: Rc::new(move |request| received.borrow_mut().push(request)),
    });
    let proxy = "https://proxy.example/?url={url}";
    vm.set_feed_connection(FeedConnection::Direct, proxy)
        .unwrap();
    vm.subscribe("news.example.com/rss#top".into());
    let subscription = requests
        .borrow()
        .iter()
        .find_map(|request| match request {
            HostRequest::Subscribe {
                url,
                proxy,
                connection,
            } => Some((url.clone(), proxy.clone(), *connection)),
            _ => None,
        })
        .unwrap();
    assert_eq!(
        subscription,
        (
            "https://news.example.com/rss".into(),
            proxy.into(),
            FeedConnection::Direct
        )
    );
    vm.finish_refresh();
    vm.refresh();
    let refreshed = requests
        .borrow()
        .iter()
        .find_map(|request| match request {
            HostRequest::Refresh {
                feeds,
                proxy,
                connection,
                ..
            } => Some((feeds.len(), proxy.clone(), *connection)),
            _ => None,
        })
        .unwrap();
    assert_eq!(
        refreshed,
        (vm.feeds().len(), proxy.into(), FeedConnection::Direct)
    );
    let serialized = serde_json::to_value(HostRequest::Subscribe {
        url: "https://example.com/".into(),
        proxy: String::new(),
        connection: FeedConnection::Automatic,
    })
    .unwrap();
    assert_eq!(serialized["connection"], "Automatic");
}

#[test]
fn invalid_connection_edits_do_not_replace_saved_settings_or_the_custom_endpoint() {
    let vm = AppViewModel::new(AppState::new(), 1000, Rc::new(|_| Ok(())));
    let proxy = "https://proxy.example/?url={url}";
    vm.set_feed_connection(FeedConnection::Custom, proxy)
        .unwrap();
    let saved = vm.state().settings.clone();
    assert!(vm.set_feed_connection(FeedConnection::Custom, "").is_err());
    assert!(vm
        .set_feed_connection(FeedConnection::Automatic, "https://broken.example/")
        .is_err());
    assert_eq!(vm.state().settings, saved);
    vm.set_feed_connection(FeedConnection::Direct, proxy)
        .unwrap();
    assert_eq!(vm.state().settings.feed_proxy, proxy);
    assert_eq!(vm.state().settings.feed_connection, FeedConnection::Direct);
}

#[test]
fn duplicate_and_credentialed_subscriptions_never_start_another_download() {
    let requests = Rc::new(RefCell::new(Vec::new()));
    let received = requests.clone();
    let vm = AppViewModel::new(AppState::new(), 1000, Rc::new(|_| Ok(())));
    let id = vm
        .receive_feed(
            "https://example.com/rss",
            "<rss><channel><title>Feed</title></channel></rss>",
        )
        .unwrap();
    vm.set_services(HostServices {
        request: Rc::new(move |request| received.borrow_mut().push(request)),
    });
    vm.subscribe("example.com/rss#article".into());
    assert_eq!(vm.view(), View::Feed(id));
    vm.subscribe("https://user:secret@example.com/rss".into());
    assert!(!vm.is_refreshing());
    assert!(requests
        .borrow()
        .iter()
        .all(|request| !matches!(request, HostRequest::Subscribe { .. })));
    assert!(vm.status().contains("without embedded credentials"));
}
