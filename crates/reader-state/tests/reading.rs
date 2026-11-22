use reader_domain::{AppState, View};
use reader_state::{AppViewModel, HostRequest, HostServices};
use std::{cell::RefCell, rc::Rc};

fn vm() -> Rc<AppViewModel> {
    AppViewModel::new(
        AppState::sample(1_800_000_000),
        1_800_000_000,
        Rc::new(|_| Ok(())),
    )
}

#[test]
fn unread_navigation_retains_open_entry_then_removes_it() {
    let vm = vm();
    vm.navigate(View::All);
    vm.set_unread_only(true);
    let before = vm.count(&View::All);
    let first = vm.visible_ids()[0];
    vm.select(Some(first));
    assert_eq!(vm.count(&View::All), before - 1);
    assert!(vm.visible_ids().contains(&first));
    vm.next_article(1);
    assert_ne!(vm.selected(), Some(first));
    assert!(!vm.visible_ids().contains(&first));
    assert_eq!(vm.count(&View::All), before - 2);
}

#[test]
fn narrow_mutation_undo_preserves_feed_content_updates() {
    let vm = AppViewModel::new(AppState::new(), 100, Rc::new(|_| Ok(())));
    let xml = "<rss><channel><title>Feed</title><item><guid>one</guid><title>Original</title><description>body</description></item></channel></rss>";
    vm.receive_feed("https://example.com/feed", xml).unwrap();
    let id = vm.state().articles[0].id;
    vm.toggle_star(id);
    vm.receive_feed(
        "https://example.com/feed",
        &xml.replace("Original", "Updated"),
    )
    .unwrap();
    vm.undo();
    assert!(!vm.article(id).unwrap().starred);
    assert_eq!(vm.article(id).unwrap().title, "Updated");
}

#[test]
fn destructive_subscription_changes_and_restore_are_undoable() {
    let vm = vm();
    let initial = vm.export_json().unwrap();
    let id = vm.feeds()[0].id;
    vm.unsubscribe(id);
    assert!(vm.feed(id).is_none());
    vm.undo();
    assert_eq!(vm.total_count(&View::All), 9);
    vm.import_json(&AppState::new().to_json().unwrap()).unwrap();
    assert_eq!(vm.total_count(&View::All), 0);
    vm.undo();
    assert_eq!(vm.total_count(&View::All), 9);
    assert_eq!(
        AppState::from_json(&vm.export_json().unwrap())
            .unwrap()
            .articles,
        AppState::from_json(&initial).unwrap().articles
    );
}

#[test]
fn persistence_is_coalesced_and_storage_failure_remains_visible() {
    let saves = Rc::new(RefCell::new(Vec::new()));
    let output = saves.clone();
    let vm = AppViewModel::new(
        AppState::sample(1000),
        1000,
        Rc::new(move |json| {
            output.borrow_mut().push(json.to_string());
            Ok(())
        }),
    );
    let requests = Rc::new(RefCell::new(0));
    let captured = requests.clone();
    vm.set_services(HostServices {
        request: Rc::new(move |request| {
            if matches!(request, HostRequest::ScheduleSave) {
                *captured.borrow_mut() += 1;
            }
        }),
    });
    let id = vm.state().articles[0].id;
    vm.toggle_star(id);
    vm.toggle_read(id);
    assert!(saves.borrow().is_empty());
    vm.flush().unwrap();
    assert_eq!(saves.borrow().len(), 1);
    assert_eq!(*requests.borrow(), 2);
    vm.flush().unwrap();
    assert_eq!(saves.borrow().len(), 1);
    let failing = AppViewModel::new(
        AppState::sample(0),
        0,
        Rc::new(|_| Err("quota exceeded".into())),
    );
    let id = failing.state().articles[0].id;
    failing.toggle_star(id);
    assert!(failing.status().contains("quota exceeded"));
    assert!(failing.flush().is_err());
}
