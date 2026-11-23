use reader_domain::{Action, AppState, Id, SubscriptionNode as Node, SubscriptionSort as Sort};

fn library() -> (AppState, Vec<Id>) {
    let mut state = AppState::new();
    let mut ids = Vec::new();
    for (index, title) in ["Zebra", "alpha", "Beta", "delta", "Charlie"]
        .into_iter()
        .enumerate()
    {
        ids.push(
            state
                .apply(
                    Action::AddSubscription {
                        title: title.into(),
                        url: format!("https://example.org/{index}.xml"),
                        site_url: String::new(),
                        folder: String::new(),
                    },
                    1000,
                )
                .unwrap()
                .created_feed
                .unwrap(),
        );
    }
    for &id in &ids[..3] {
        state
            .apply(
                Action::SetFeedFolders {
                    id,
                    folders: vec!["Work".into(), "Reading".into()],
                },
                1000,
            )
            .unwrap();
    }
    state
        .apply(Action::CreateFolder("empty".into()), 1000)
        .unwrap();
    (state, ids)
}

fn occurrence(id: Id, from: &str, to: &str, target: Option<Node>, before: bool) -> Action {
    Action::PositionFeedOccurrence {
        id,
        from_folder: from.into(),
        to_folder: to.into(),
        target,
        before,
    }
}

#[test]
fn missing_metadata_preserves_legacy_order_and_invalid_overrides_are_repaired() {
    let (mut state, ids) = library();
    let expected = state.subscription_tree();
    assert_eq!(
        expected.root,
        vec![
            Node::Feed(ids[3]),
            Node::Feed(ids[4]),
            Node::Folder("Reading".into()),
            Node::Folder("Work".into()),
            Node::Folder("empty".into())
        ]
    );
    assert_eq!(expected.folders["Work"], ids[..3]);
    let json = state.to_json().unwrap();
    assert!(!json.contains("subscription_order"));
    assert_eq!(
        AppState::from_json(&json).unwrap().subscription_tree(),
        expected
    );
    state.subscription_order.root = vec![
        Node::Feed(999),
        Node::Feed(ids[0]),
        Node::Folder("missing".into()),
        Node::Feed(ids[4]),
        Node::Feed(ids[4]),
    ];
    state
        .subscription_order
        .folders
        .insert("Work".into(), vec![ids[2], ids[2], 999, ids[3]]);
    state
        .subscription_order
        .folders
        .insert("missing".into(), vec![ids[0]]);
    state.normalize().unwrap();
    assert_eq!(state.subscription_order.root, vec![Node::Feed(ids[4])]);
    assert_eq!(state.subscription_order.folders["Work"], vec![ids[2]]);
    assert!(!state.subscription_order.folders.contains_key("missing"));
    assert_eq!(
        state.subscription_tree().folders["Work"],
        vec![ids[2], ids[0], ids[1]]
    );
    assert_eq!(
        AppState::from_json(&state.to_json().unwrap()).unwrap(),
        state
    );
}

#[test]
fn manual_positions_are_independent_per_occurrence_and_mix_root_node_kinds() {
    let (mut state, ids) = library();
    let before = state.clone();
    let changes = state
        .apply(
            occurrence(ids[2], "Work", "Work", Some(Node::Feed(ids[0])), true),
            1000,
        )
        .unwrap();
    assert!(changes.changed && changes.order && !changes.feeds && !changes.settings);
    assert!(changes.articles.is_empty() && changes.removed.is_empty());
    assert_eq!(
        state.subscription_tree().folders["Work"],
        vec![ids[2], ids[0], ids[1]]
    );
    assert_eq!(state.subscription_tree().folders["Reading"], ids[..3]);
    assert_eq!(state.feeds, before.feeds);
    assert_eq!(state.articles, before.articles);
    state
        .apply(
            Action::PositionRoot {
                item: Node::Folder("Work".into()),
                target: Node::Feed(ids[3]),
                before: true,
            },
            1000,
        )
        .unwrap();
    state
        .apply(
            Action::PositionRoot {
                item: Node::Feed(ids[4]),
                target: Node::Folder("Reading".into()),
                before: false,
            },
            1000,
        )
        .unwrap();
    assert_eq!(
        state.subscription_tree().root,
        vec![
            Node::Folder("Work".into()),
            Node::Feed(ids[3]),
            Node::Folder("Reading".into()),
            Node::Feed(ids[4]),
            Node::Folder("empty".into())
        ]
    );
    assert_eq!(
        AppState::from_json(&state.to_json().unwrap()).unwrap(),
        state
    );
}

#[test]
fn alphabetical_mode_groups_folders_preserves_manual_positions_and_follows_title_changes() {
    let (mut state, ids) = library();
    state
        .apply(
            occurrence(ids[2], "Work", "Work", Some(Node::Feed(ids[0])), true),
            1000,
        )
        .unwrap();
    state
        .apply(
            Action::PositionRoot {
                item: Node::Folder("Work".into()),
                target: Node::Feed(ids[3]),
                before: true,
            },
            1000,
        )
        .unwrap();
    let manual = state.subscription_tree();
    let saved = state.subscription_order.clone();
    let changes = state
        .apply(Action::SetSubscriptionSort(Sort::Alphabetical), 1000)
        .unwrap();
    assert!(changes.order && !changes.feeds && !changes.settings);
    let alphabetic = state.subscription_tree();
    assert_eq!(
        alphabetic.root,
        vec![
            Node::Folder("empty".into()),
            Node::Folder("Reading".into()),
            Node::Folder("Work".into()),
            Node::Feed(ids[4]),
            Node::Feed(ids[3])
        ]
    );
    assert_eq!(alphabetic.folders["Work"], vec![ids[1], ids[2], ids[0]]);
    for action in [
        occurrence(ids[0], "Work", "Work", Some(Node::Feed(ids[1])), true),
        Action::SortSubscriptions(Some("Work".into())),
        Action::PositionRoot {
            item: Node::Feed(ids[3]),
            target: Node::Folder("Work".into()),
            before: true,
        },
    ] {
        assert!(!state.apply(action, 1000).unwrap().changed);
    }
    assert_eq!(state.subscription_order.root, saved.root);
    assert_eq!(state.subscription_order.folders, saved.folders);
    state
        .apply(
            Action::RenameFeed {
                id: ids[0],
                title: "aardvark".into(),
            },
            1000,
        )
        .unwrap();
    assert_eq!(state.subscription_tree().folders["Work"], ids[..3]);
    let reloaded = AppState::from_json(&state.to_json().unwrap()).unwrap();
    assert_eq!(reloaded.subscription_tree(), state.subscription_tree());
    state
        .apply(Action::SetSubscriptionSort(Sort::Manual), 1000)
        .unwrap();
    assert_eq!(state.subscription_tree(), manual);
}

#[test]
fn occurrence_moves_preserve_other_labels_and_folder_to_root_only_removes_source_label() {
    let (mut state, ids) = library();
    state
        .apply(
            occurrence(ids[2], "Reading", "Reading", Some(Node::Feed(ids[0])), true),
            1000,
        )
        .unwrap();
    let reading = state.subscription_tree().folders["Reading"].clone();
    state
        .apply(occurrence(ids[0], "Work", "empty", None, false), 1000)
        .unwrap();
    assert_eq!(
        state.feed(ids[0]).unwrap().folder_names(),
        vec!["Reading", "empty"]
    );
    assert_eq!(state.subscription_tree().folders["Reading"], reading);
    state
        .apply(
            occurrence(ids[0], "empty", "", Some(Node::Folder("Work".into())), true),
            1000,
        )
        .unwrap();
    assert_eq!(state.feed(ids[0]).unwrap().folder_names(), vec!["Reading"]);
    assert!(!state.subscription_tree().root.contains(&Node::Feed(ids[0])));
    state
        .apply(
            occurrence(
                ids[0],
                "Reading",
                "",
                Some(Node::Folder("Work".into())),
                true,
            ),
            1000,
        )
        .unwrap();
    let root = state.subscription_tree().root;
    let index = root
        .iter()
        .position(|node| *node == Node::Feed(ids[0]))
        .unwrap();
    assert_eq!(root[index + 1], Node::Folder("Work".into()));
    assert!(state.feed(ids[0]).unwrap().folder_names().is_empty());
}

#[test]
fn stale_source_destination_and_anchor_reject_the_whole_mutation() {
    let (mut state, ids) = library();
    let original = state.clone();
    for action in [
        occurrence(ids[0], "", "Work", None, false),
        occurrence(ids[0], "Work", "deleted", None, false),
        occurrence(ids[0], "Work", "empty", Some(Node::Feed(ids[1])), true),
        occurrence(
            ids[0],
            "Work",
            "Reading",
            Some(Node::Folder("Reading".into())),
            true,
        ),
        occurrence(ids[0], "Work", "", Some(Node::Feed(ids[1])), true),
        occurrence(999, "Work", "Reading", None, false),
        Action::PositionRoot {
            item: Node::Feed(ids[0]),
            target: Node::Folder("Work".into()),
            before: true,
        },
        Action::SortSubscriptions(Some("deleted".into())),
    ] {
        assert!(state.apply(action, 1000).is_err());
        assert_eq!(state, original);
    }
}

#[test]
fn alphabetical_cross_parent_move_updates_membership_without_changing_mode_or_other_orders() {
    let (mut state, ids) = library();
    state
        .apply(
            occurrence(ids[2], "Work", "Work", Some(Node::Feed(ids[0])), true),
            1000,
        )
        .unwrap();
    state
        .apply(
            occurrence(ids[1], "Reading", "Reading", Some(Node::Feed(ids[0])), true),
            1000,
        )
        .unwrap();
    state
        .apply(Action::SetSubscriptionSort(Sort::Alphabetical), 1000)
        .unwrap();
    let reading = state.subscription_order.folders["Reading"].clone();
    state
        .apply(occurrence(ids[2], "Work", "empty", None, false), 1000)
        .unwrap();
    assert_eq!(state.subscription_order.sort, Sort::Alphabetical);
    assert_eq!(
        state.subscription_order.folders["Work"],
        vec![ids[0], ids[1]]
    );
    assert_eq!(state.subscription_order.folders["Reading"], reading);
    assert!(!state.subscription_order.folders.contains_key("empty"));
    assert_eq!(state.subscription_tree().folders["empty"], vec![ids[2]]);
}

#[test]
fn rename_delete_unsubscribe_and_opml_keep_order_references_valid() {
    let (mut state, ids) = library();
    state
        .apply(
            occurrence(ids[2], "Work", "Work", Some(Node::Feed(ids[0])), true),
            1000,
        )
        .unwrap();
    state
        .apply(
            Action::PositionRoot {
                item: Node::Folder("Work".into()),
                target: Node::Feed(ids[3]),
                before: true,
            },
            1000,
        )
        .unwrap();
    state
        .apply(
            Action::RenameFolder {
                from: "Work".into(),
                to: "Morning".into(),
            },
            1000,
        )
        .unwrap();
    assert_eq!(
        state.subscription_tree().root[0],
        Node::Folder("Morning".into())
    );
    assert_eq!(
        state.subscription_tree().folders["Morning"],
        vec![ids[2], ids[0], ids[1]]
    );
    let opml = state.export_opml();
    assert!(opml.find("text=\"Morning\"").unwrap() < opml.find("text=\"delta\"").unwrap());
    assert!(opml.find("text=\"Beta\"").unwrap() < opml.find("text=\"Zebra\"").unwrap());
    state
        .apply(Action::RemoveFeeds(vec![ids[0], ids[3]]), 1000)
        .unwrap();
    assert_eq!(
        state.subscription_order.folders["Morning"],
        vec![ids[2], ids[1]]
    );
    assert!(!state.subscription_order.root.contains(&Node::Feed(ids[3])));
    state
        .apply(Action::DeleteFolder("Morning".into()), 1000)
        .unwrap();
    assert!(!state.subscription_order.folders.contains_key("Morning"));
    assert!(!state
        .subscription_order
        .root
        .contains(&Node::Folder("Morning".into())));
    assert_eq!(
        AppState::from_json(&state.to_json().unwrap()).unwrap(),
        state
    );
}

#[test]
fn scoped_sort_is_stable_and_new_subscriptions_append_without_replacing_positions() {
    let (mut state, ids) = library();
    state
        .apply(
            Action::RenameFeed {
                id: ids[0],
                title: "ALPHA".into(),
            },
            1000,
        )
        .unwrap();
    state
        .apply(
            occurrence(ids[1], "Work", "Work", Some(Node::Feed(ids[0])), true),
            1000,
        )
        .unwrap();
    state
        .apply(Action::SortSubscriptions(Some("Work".into())), 1000)
        .unwrap();
    assert_eq!(
        state.subscription_tree().folders["Work"],
        vec![ids[1], ids[0], ids[2]]
    );
    assert_eq!(state.subscription_tree().folders["Reading"], ids[..3]);
    let new = state
        .apply(
            Action::AddSubscription {
                title: "A new source".into(),
                url: "https://example.org/new.xml".into(),
                site_url: String::new(),
                folder: "Work".into(),
            },
            1000,
        )
        .unwrap()
        .created_feed
        .unwrap();
    assert_eq!(
        state.subscription_tree().folders["Work"],
        vec![ids[1], ids[0], ids[2], new]
    );
    state
        .apply(Action::SetSubscriptionSort(Sort::Alphabetical), 1000)
        .unwrap();
    assert_eq!(state.subscription_tree().folders["Work"][0], new);
    state
        .apply(Action::SetSubscriptionSort(Sort::Manual), 1000)
        .unwrap();
    assert_eq!(state.subscription_tree().folders["Work"].last(), Some(&new));
}
