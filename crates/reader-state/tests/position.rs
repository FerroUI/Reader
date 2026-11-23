use reader_domain::{Action, AppState, Id};
use reader_state::AppViewModel;
use std::rc::Rc;

fn ids(vm: &AppViewModel) -> Vec<Id> {
    vm.feeds().iter().map(|feed| feed.id).collect()
}

#[test]
fn row_reorder_and_cross_folder_move_are_single_undoable_transactions() {
    let mut state = AppState::sample(1000);
    let a = state.feeds[0].id;
    let b = state.feeds[1].id;
    let d = state.feeds[3].id;
    let unfiled = state.feeds[5].id;
    state
        .apply(
            Action::SetFeedFolders {
                id: unfiled,
                folders: vec![],
            },
            1000,
        )
        .unwrap();
    state
        .apply(
            Action::SetFeedFolders {
                id: d,
                folders: vec!["Technology".into(), "Reading".into()],
            },
            1000,
        )
        .unwrap();
    let vm = AppViewModel::new(state, 1000, Rc::new(|_| Ok(())));
    let before = vm.state().clone();
    let articles: Vec<_> = before
        .articles
        .iter()
        .map(|article| vm.article_shared(article.id).unwrap())
        .collect();
    vm.position_feed(d, a, true, Some("Technology".into()));
    assert_eq!(vm.subscription_tree().folders["Technology"][0], d);
    assert_eq!(
        ids(&vm),
        before.feeds.iter().map(|feed| feed.id).collect::<Vec<_>>()
    );
    assert_eq!(
        vm.feed(d).unwrap().folder_names(),
        vec!["Reading", "Technology"]
    );
    vm.undo();
    assert_eq!(*vm.state(), before);
    assert!(!vm.can_undo());
    vm.position_feed(d, b, false, Some("Science".into()));
    let order = &vm.subscription_tree().folders["Science"];
    assert_eq!(
        order.iter().position(|id| *id == d),
        order.iter().position(|id| *id == b).map(|i| i + 1)
    );
    assert_eq!(vm.feed(d).unwrap().folder_names(), vec!["Science"]);
    vm.undo();
    assert_eq!(*vm.state(), before);
    assert!(!vm.can_undo());
    vm.position_feed(d, unfiled, false, Some(String::new()));
    assert!(vm.feed(d).unwrap().folder_names().is_empty());
    let root = vm.subscription_tree().root;
    let position = root
        .iter()
        .position(|node| *node == reader_domain::SubscriptionNode::Feed(d))
        .unwrap();
    assert_eq!(
        root[position - 1],
        reader_domain::SubscriptionNode::Feed(unfiled)
    );
    vm.undo();
    assert_eq!(*vm.state(), before);
    assert!(!vm.can_undo());
    for old in articles {
        assert!(Rc::ptr_eq(&old, &vm.article_shared(old.id).unwrap()));
    }
}

#[test]
fn undo_reorder_retains_background_feed_updates_and_new_subscriptions() {
    let vm = AppViewModel::new(AppState::sample(1000), 1000, Rc::new(|_| Ok(())));
    let original = ids(&vm);
    vm.reorder_feed(original[0], original[5], false);
    let url = vm.feed(original[0]).unwrap().url;
    vm.feed_error(&url, "Background response failed");
    let new_id = vm
        .receive_feed(
            "https://example.org/new.xml",
            "<rss><channel><title>New</title></channel></rss>",
        )
        .unwrap();
    vm.undo();
    let mut expected = original;
    expected.push(new_id);
    assert_eq!(ids(&vm), expected);
    assert_eq!(
        vm.feed(expected[0]).unwrap().error.as_deref(),
        Some("Background response failed")
    );
    assert!(vm.feed(new_id).is_some());
}
