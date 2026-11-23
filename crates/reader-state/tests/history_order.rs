use reader_domain::{
    Action, AppState, Article, Feed, Id, SubscriptionNode as Node, SubscriptionOrder,
    SubscriptionSort, View,
};
use reader_state::AppViewModel;
use std::{collections::BTreeMap, rc::Rc};

fn state() -> AppState {
    let mut state = AppState::new();
    state.folders = vec!["A".into(), "B".into(), "Empty".into()];
    for (id, title, folders) in [
        (1, "Zulu", vec!["A", "B"]),
        (2, "Alpha", vec!["A", "B"]),
        (3, "Beta", vec!["A"]),
        (4, "Zulu root", vec![]),
        (5, "Alpha root", vec![]),
    ] {
        state.feeds.push(Feed {
            id,
            title: title.into(),
            url: format!("https://example.com/{id}"),
            folder: folders.first().copied().unwrap_or_default().into(),
            folders: folders.into_iter().map(str::to_owned).collect(),
            ..Feed::default()
        });
        state.articles.push(Article {
            id: id + 10,
            feed_id: id,
            guid: "one".into(),
            title: format!("Article {id}"),
            content: "Original content".into(),
            published: 100,
            ..Article::default()
        });
    }
    state.subscription_order = SubscriptionOrder {
        sort: SubscriptionSort::Manual,
        root: vec![
            Node::Folder("B".into()),
            Node::Feed(5),
            Node::Folder("A".into()),
            Node::Feed(4),
            Node::Folder("Empty".into()),
        ],
        folders: BTreeMap::from([("A".into(), vec![3, 1, 2]), ("B".into(), vec![2, 1])]),
    };
    state.normalize().unwrap();
    state
}

fn vm(state: AppState) -> Rc<AppViewModel> {
    AppViewModel::new(state, 100, Rc::new(|_| Ok(())))
}

fn assert_roundtrip(vm: &AppViewModel) {
    let restored = AppState::from_json(&vm.export_json().unwrap()).unwrap();
    assert_eq!(restored.subscription_order, vm.state().subscription_order);
    assert_eq!(restored.subscription_tree(), vm.state().subscription_tree());
    assert_eq!(restored.feeds, vm.state().feeds);
    assert_eq!(restored.articles, vm.state().articles);
    let mut normalized = restored;
    assert!(
        !normalized.normalize_subscription_order(),
        "persisted ordering is already canonical"
    );
}

#[test]
fn folder_moves_undo_every_occurrence_without_changing_other_labels_positions() {
    for action in [
        Action::PositionFeedOccurrence {
            id: 1,
            from_folder: "A".into(),
            to_folder: "Empty".into(),
            target: None,
            before: false,
        },
        Action::PositionFeedOccurrence {
            id: 1,
            from_folder: "A".into(),
            to_folder: String::new(),
            target: Some(Node::Feed(5)),
            before: true,
        },
        Action::PositionFeedOccurrence {
            id: 4,
            from_folder: String::new(),
            to_folder: "A".into(),
            target: Some(Node::Feed(1)),
            before: true,
        },
        Action::MoveFeed {
            id: 1,
            folder: "Fresh".into(),
        },
        Action::SetFeedFolders {
            id: 1,
            folders: vec!["B".into()],
        },
        Action::PositionFeed {
            id: 1,
            target: 5,
            before: true,
            folder: Some(String::new()),
        },
    ] {
        let initial = state();
        let vm = vm(initial.clone());
        vm.dispatch(action);
        assert!(vm.can_undo());
        assert_roundtrip(&vm);
        vm.undo();
        assert_eq!(vm.state().subscription_order, initial.subscription_order);
        assert_eq!(vm.state().feeds, initial.feeds);
        assert_eq!(vm.state().folders, initial.folders);
        assert_eq!(vm.state().subscription_tree(), initial.subscription_tree());
        assert!(!vm.can_undo());
        assert_roundtrip(&vm);
    }
}

#[test]
fn rename_delete_and_create_undo_restore_folder_nodes_and_child_positions() {
    for action in [
        Action::RenameFolder {
            from: "A".into(),
            to: "Renamed".into(),
        },
        Action::DeleteFolder("A".into()),
        Action::CreateFolder("Added".into()),
    ] {
        let initial = state();
        let vm = vm(initial.clone());
        vm.dispatch(action);
        assert_roundtrip(&vm);
        vm.undo();
        assert_eq!(vm.state().subscription_order, initial.subscription_order);
        assert_eq!(vm.state().subscription_tree(), initial.subscription_tree());
        assert!(!vm.can_undo());
        assert_roundtrip(&vm);
    }
}

#[test]
fn alphabetical_mode_and_one_time_sorts_restore_prior_manual_positions() {
    for action in [
        Action::SetSubscriptionSort(SubscriptionSort::Alphabetical),
        Action::SortSubscriptions(None),
        Action::SortSubscriptions(Some("A".into())),
        Action::PositionRoot {
            item: Node::Folder("A".into()),
            target: Node::Feed(4),
            before: false,
        },
        Action::PositionFeedOccurrence {
            id: 2,
            from_folder: "A".into(),
            to_folder: "A".into(),
            target: Some(Node::Feed(3)),
            before: true,
        },
    ] {
        let initial = state();
        let vm = vm(initial.clone());
        let epoch = vm.subscription_epoch(1);
        vm.dispatch(action);
        assert!(vm.can_undo());
        assert_roundtrip(&vm);
        vm.undo();
        assert_eq!(vm.state().subscription_order, initial.subscription_order);
        assert_eq!(vm.subscription_epoch(1), epoch);
        assert!(!vm.can_undo());
        assert_roundtrip(&vm);
    }
}

const UPDATED: &str = "<rss><channel><title>New publisher title</title><item><guid>one</guid><title>Updated article</title><description>Latest content</description></item><item><guid>new</guid><title>New arrival</title></item></channel></rss>";

fn replace_id(order: &mut SubscriptionOrder, old: Id, new: Id) {
    for node in &mut order.root {
        if *node == Node::Feed(old) {
            *node = Node::Feed(new);
        }
    }
    for ids in order.folders.values_mut() {
        for id in ids {
            if *id == old {
                *id = new;
            }
        }
    }
}

#[test]
fn bulk_remove_resubscribe_undo_rebinds_root_folder_and_older_order_snapshots() {
    let initial = state();
    let vm = vm(initial.clone());
    vm.dispatch(Action::PositionFeedOccurrence {
        id: 2,
        from_folder: "A".into(),
        to_folder: "A".into(),
        target: Some(Node::Feed(3)),
        before: true,
    });
    let before_removal = vm.state().subscription_order.clone();
    vm.unsubscribe_many(vec![1, 4]);
    assert!(vm.state().feed(1).is_none() && vm.state().feed(4).is_none());
    let new_folder_source = vm.receive_feed("https://example.com/1", UPDATED).unwrap();
    let new_root_source = vm.receive_feed("https://example.com/4", UPDATED).unwrap();
    assert_ne!(new_folder_source, 1);
    assert_ne!(new_root_source, 4);
    vm.undo();
    let mut expected = before_removal;
    replace_id(&mut expected, 1, new_folder_source);
    replace_id(&mut expected, 4, new_root_source);
    assert_eq!(vm.state().subscription_order, expected);
    assert_eq!(
        vm.feed(new_folder_source).unwrap().folder_names(),
        ["A", "B"]
    );
    assert_eq!(
        vm.state()
            .articles
            .iter()
            .filter(|article| article.feed_id == new_folder_source)
            .count(),
        2
    );
    assert_eq!(
        vm.state()
            .articles
            .iter()
            .filter(|article| article.feed_id == new_root_source)
            .count(),
        2
    );
    assert_roundtrip(&vm);
    vm.undo();
    let mut expected = initial.subscription_order;
    replace_id(&mut expected, 1, new_folder_source);
    replace_id(&mut expected, 4, new_root_source);
    assert_eq!(vm.state().subscription_order, expected);
    assert_eq!(
        vm.feed(new_folder_source).unwrap().folder_names(),
        ["A", "B"]
    );
    assert!(!vm.can_undo());
    assert_roundtrip(&vm);
}

#[test]
fn archived_reactivation_undo_restores_positions_and_retains_new_downloads() {
    let mut initial = state();
    initial
        .articles
        .iter_mut()
        .find(|article| article.feed_id == 1)
        .unwrap()
        .starred = true;
    let vm = vm(initial.clone());
    vm.unsubscribe(1);
    assert!(vm.state().feed(1).unwrap().archived);
    assert_eq!(
        vm.receive_feed("https://example.com/1", UPDATED).unwrap(),
        1
    );
    vm.undo();
    assert_eq!(vm.state().subscription_order, initial.subscription_order);
    assert_eq!(vm.state().subscription_tree(), initial.subscription_tree());
    assert_eq!(vm.total_count(&View::Feed(1)), 2);
    assert!(vm.state().article(11).unwrap().starred);
    assert_eq!(vm.state().article(11).unwrap().content, "Latest content");
    assert_roundtrip(&vm);
}

#[test]
fn import_and_backup_undo_restore_ordering_without_stale_references() {
    let initial = state();
    let vm = vm(initial.clone());
    let opml = "<opml version='2.0'><body><outline text='Imported'><outline type='rss' text='New source' xmlUrl='https://new.example/feed'/></outline></body></opml>";
    assert_eq!(vm.import_opml(opml).unwrap(), 1);
    assert_roundtrip(&vm);
    vm.undo();
    assert_eq!(vm.state().subscription_order, initial.subscription_order);
    assert_eq!(vm.state().subscription_tree(), initial.subscription_tree());
    assert!(!vm.folders().contains(&"Imported".into()));
    assert_roundtrip(&vm);
    let mut replacement = state();
    replacement.subscription_order.sort = SubscriptionSort::Alphabetical;
    replacement.subscription_order.root.reverse();
    vm.import_json(&replacement.to_json().unwrap()).unwrap();
    assert_eq!(
        vm.state().subscription_order,
        replacement.subscription_order
    );
    vm.undo();
    assert_eq!(vm.state().subscription_order, initial.subscription_order);
    assert_eq!(vm.state().subscription_tree(), initial.subscription_tree());
    assert_roundtrip(&vm);
}

#[test]
fn undo_add_subscription_removes_its_new_folder_and_preserves_existing_order() {
    let initial = state();
    let vm = vm(initial.clone());
    vm.dispatch(Action::AddSubscription {
        title: "New".into(),
        url: "https://new.example/feed".into(),
        site_url: String::new(),
        folder: "Added".into(),
    });
    assert!(vm.folders().contains(&"Added".into()));
    vm.undo();
    assert_eq!(vm.state().feeds, initial.feeds);
    assert_eq!(vm.state().folders, initial.folders);
    assert_eq!(vm.state().subscription_order, initial.subscription_order);
    assert!(!vm.can_undo());
    assert_roundtrip(&vm);
}
