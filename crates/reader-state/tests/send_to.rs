use reader_domain::{AppState, Article, Feed, SendToDestination};
use reader_state::{AppViewModel, HostRequest, HostServices};
use std::{cell::RefCell, rc::Rc};

fn model() -> (Rc<AppViewModel>, Rc<RefCell<Vec<HostRequest>>>) {
    let mut state = AppState::new();
    state.feeds = vec![Feed {
        id: 1,
        title: "Original source".into(),
        url: "https://news.example/feed".into(),
        ..Default::default()
    }];
    state.articles = vec![Article {
        id: 2,
        feed_id: 1,
        guid: "one".into(),
        title: "Article & title".into(),
        url: "https://news.example/a#part".into(),
        ..Default::default()
    }];
    let vm = AppViewModel::new(state, 1000, Rc::new(|_| Ok(())));
    let requests = Rc::new(RefCell::new(Vec::new()));
    let sink = requests.clone();
    vm.set_services(HostServices {
        request: Rc::new(move |request| sink.borrow_mut().push(request)),
    });
    (vm, requests)
}
fn save(vm: &AppViewModel) -> SendToDestination {
    vm.save_send_to(
        None,
        "Notebook",
        "https://save.example/?url=${url}&title=${title}&source=${source}",
        true,
    )
    .unwrap();
    vm.state().settings.send_to[0].clone()
}
fn opened(requests: &RefCell<Vec<HostRequest>>) -> Vec<String> {
    requests
        .borrow()
        .iter()
        .filter_map(|request| {
            if let HostRequest::OpenUrl { url } = request {
                Some(url.clone())
            } else {
                None
            }
        })
        .collect()
}

#[test]
fn configuring_a_destination_is_persistent_and_does_not_launch_it() {
    let (vm, requests) = model();
    let destination = save(&vm);
    assert!(opened(&requests).is_empty());
    assert!(!vm.article(2).unwrap().read);
    let restored = AppState::from_json(&vm.export_json().unwrap()).unwrap();
    assert_eq!(restored.settings.send_to, vec![destination.clone()]);
    let saved = vm.state().settings.clone();
    assert!(vm
        .save_send_to(Some(destination.id), "Notebook", "javascript:${url}", true)
        .is_err());
    assert_eq!(vm.state().settings, saved);
}

#[test]
fn sending_resolves_current_source_title_without_read_or_share_side_effects() {
    let (vm, requests) = model();
    let destination = save(&vm);
    vm.rename_feed(1, "Renamed & source".into());
    vm.send_article_to(
        2,
        "https://news.example/a#part",
        vm.library_generation(),
        &destination,
    )
    .unwrap();
    let urls = opened(&requests);
    assert_eq!(urls.len(), 1);
    assert_eq!(urls[0],"https://save.example/?url=https%3A%2F%2Fnews.example%2Fa%23part&title=Article%20%26%20title&source=Renamed%20%26%20source");
    assert!(!vm.article(2).unwrap().read);
    assert!(requests
        .borrow()
        .iter()
        .all(|request| !matches!(request, HostRequest::Share { .. })));
}

#[test]
fn stale_menu_references_cannot_send_replaced_articles_or_destinations() {
    let (vm, requests) = model();
    let destination = save(&vm);
    let generation = vm.library_generation();
    assert!(vm
        .send_article_to(999, "https://news.example/a#part", generation, &destination)
        .is_err());
    assert!(vm
        .send_article_to(2, "https://news.example/other", generation, &destination)
        .is_err());
    vm.set_send_to_enabled(destination.id, false).unwrap();
    assert!(vm
        .send_article_to(2, "https://news.example/a#part", generation, &destination)
        .is_err());
    vm.set_send_to_enabled(destination.id, true).unwrap();
    vm.save_send_to(
        Some(destination.id),
        "Notebook",
        "https://other.example/?url=${url}",
        true,
    )
    .unwrap();
    assert!(vm
        .send_article_to(2, "https://news.example/a#part", generation, &destination)
        .is_err());
    vm.import_json(&vm.export_json().unwrap()).unwrap();
    assert!(vm
        .send_article_to(2, "https://news.example/a#part", generation, &destination)
        .is_err());
    assert!(opened(&requests).is_empty());
}

#[test]
fn removing_a_destination_can_be_undone_and_never_launches_it() {
    let (vm, requests) = model();
    let destination = save(&vm);
    vm.remove_send_to(destination.id);
    assert!(vm.state().settings.send_to.is_empty());
    vm.undo();
    assert_eq!(vm.state().settings.send_to, vec![destination]);
    assert!(opened(&requests).is_empty());
}
