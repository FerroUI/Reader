use reader_domain::{Action, AppState, View};
use reader_state::{AppViewModel, HostRequest, HostServices};
use std::{cell::RefCell, rc::Rc};

fn vm() -> Rc<AppViewModel> {
    let mut state = AppState::sample(1000);
    state
        .apply(Action::ToggleStar(state.articles[0].id), 1000)
        .unwrap();
    AppViewModel::new(state, 1000, Rc::new(|_| Ok(())))
}

#[test]
fn same_identity_reactivation_rejects_pre_unsubscribe_responses_and_preserves_other_lifetimes() {
    let vm = vm();
    let source = vm.feeds()[0].clone();
    let other = vm.feeds()[1].clone();
    let epoch = vm.subscription_epoch(source.id).unwrap();
    let other_epoch = vm.subscription_epoch(other.id).unwrap();
    assert!(vm.accepts_feed_result(source.id, &source.url, epoch));
    assert!(!vm.accepts_feed_result(source.id, "https://wrong.example/", epoch));
    assert!(!vm.accepts_feed_result(source.id, &source.url, 0));
    vm.unsubscribe(source.id);
    assert_eq!(vm.subscription_epoch(source.id), None);
    assert!(!vm.accepts_feed_result(source.id, &source.url, epoch));
    assert_eq!(vm.subscription_epoch(other.id), Some(other_epoch));
    vm.receive_feed(
        &source.url,
        "<rss><channel><title>Returned</title></channel></rss>",
    )
    .unwrap();
    assert_eq!(vm.feed(source.id).unwrap().url, source.url);
    let current = vm.subscription_epoch(source.id).unwrap();
    assert_ne!(current, epoch);
    assert!(!vm.accepts_feed_result(source.id, &source.url, epoch));
    assert!(vm.accepts_feed_result(source.id, &source.url, current));
    assert_eq!(vm.subscription_epoch(other.id), Some(other_epoch));
    vm.rename_feed(source.id, "Local label".into());
    vm.set_feed_folders(source.id, vec!["Reading".into()]);
    vm.feed_error(&source.url, "Temporary network failure");
    assert_eq!(vm.subscription_epoch(source.id), Some(current));
}

#[test]
fn undo_reactivation_gets_a_new_lifetime_even_when_it_restores_the_same_source_snapshot() {
    let vm = vm();
    let source = vm.feeds()[0].clone();
    let initial = vm.subscription_epoch(source.id).unwrap();
    vm.unsubscribe(source.id);
    vm.undo();
    let restored = vm.subscription_epoch(source.id).unwrap();
    assert_ne!(restored, initial);
    assert!(!vm.accepts_feed_result(source.id, &source.url, initial));
    assert!(vm.accepts_feed_result(source.id, &source.url, restored));
    assert_eq!(vm.feed(source.id).unwrap(), source);
}

#[test]
fn imported_library_and_snapshot_undo_invalidate_all_captured_lifetimes() {
    let vm = vm();
    let source = vm.feeds()[0].clone();
    let initial = vm.subscription_epoch(source.id).unwrap();
    let backup = vm.export_json().unwrap();
    vm.import_json(&backup).unwrap();
    let imported = vm.subscription_epoch(source.id).unwrap();
    assert_ne!(imported, initial);
    assert!(!vm.accepts_feed_result(source.id, &source.url, initial));
    vm.undo();
    let restored = vm.subscription_epoch(source.id).unwrap();
    assert_ne!(restored, imported);
    assert!(!vm.accepts_feed_result(source.id, &source.url, imported));
    assert!(vm.accepts_feed_result(source.id, &source.url, restored));
    assert!(
        !vm.export_json().unwrap().contains("epoch"),
        "lifetimes never enter portable backups"
    );
}

#[test]
fn refresh_request_captures_exact_source_lifetimes_before_async_host_work() {
    let vm = vm();
    let source = vm.feeds()[0].clone();
    let captured = Rc::new(RefCell::new(Vec::new()));
    let output = captured.clone();
    vm.set_services(HostServices {
        request: Rc::new(move |request| output.borrow_mut().push(request)),
    });
    assert!(vm.refresh_view(View::Feed(source.id)));
    let epoch = match &captured.borrow()[0] {
        HostRequest::Refresh { feeds, epochs, .. } => {
            assert_eq!(feeds, &[source.clone()]);
            assert_eq!(epochs.len(), 1);
            epochs[&source.id]
        }
        _ => panic!("expected refresh request"),
    };
    assert_eq!(Some(epoch), vm.subscription_epoch(source.id));
    vm.unsubscribe(source.id);
    vm.undo();
    assert!(!vm.accepts_feed_result(source.id, &source.url, epoch));
}
