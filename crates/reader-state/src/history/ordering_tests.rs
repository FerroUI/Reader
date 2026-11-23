use super::{aliases::Aliases, UndoChange, UndoEntry};
use crate::AppViewModel;
use reader_domain::{
    Action, AppState, Article, Feed, SubscriptionNode as Node, SubscriptionOrder, SubscriptionSort,
    View,
};
use std::{collections::BTreeMap, rc::Rc};

fn state() -> AppState {
    let mut state = AppState::new();
    for (id, title, folder) in [
        (1, "Zulu", ""),
        (2, "Alpha", ""),
        (3, "Zulu child", "Folder"),
        (4, "Alpha child", "Folder"),
    ] {
        state.feeds.push(Feed {
            id,
            title: title.into(),
            url: format!("https://example.com/{id}"),
            folder: folder.into(),
            folders: if folder.is_empty() {
                vec![]
            } else {
                vec![folder.into()]
            },
            ..Feed::default()
        });
        state.articles.push(Article {
            id: id + 10,
            feed_id: id,
            guid: "one".into(),
            title: format!("Article {id}"),
            content: "A substantial article body. ".repeat(2048),
            ..Article::default()
        });
    }
    state.normalize().unwrap();
    state
}

#[test]
fn pure_order_commands_and_undo_preserve_article_allocations_and_never_touch_indexes() {
    let initial = state();
    let vm = AppViewModel::new(initial.clone(), 100, Rc::new(|_| Ok(())));
    vm.navigate(View::All);
    let article_allocation = vm.state().articles.as_ptr();
    let body_allocations: Vec<_> = vm
        .state()
        .articles
        .iter()
        .map(|article| article.content.as_ptr())
        .collect();
    let shared = vm.visible_shared();
    let generation = vm.library_generation();
    let counts = vm.counts.borrow();
    let visible = vm.visible_ids.borrow();
    let epochs = vm.feed_epochs.borrow();
    // These immutable guards deliberately prohibit query/index mutation. An
    // order-only action must not enter any of the normal article publish work.
    for action in [
        Action::PositionRoot {
            item: Node::Feed(2),
            target: Node::Folder("Folder".into()),
            before: false,
        },
        Action::PositionFeedOccurrence {
            id: 4,
            from_folder: "Folder".into(),
            to_folder: "Folder".into(),
            target: Some(Node::Feed(3)),
            before: true,
        },
        Action::SortSubscriptions(None),
        Action::SortSubscriptions(Some("Folder".into())),
        Action::SetSubscriptionSort(SubscriptionSort::Alphabetical),
        Action::PositionFeed {
            id: 1,
            target: 4,
            before: false,
            folder: None,
        },
        Action::PositionFeed {
            id: 4,
            target: 3,
            before: true,
            folder: Some("Folder".into()),
        },
    ] {
        vm.dispatch(action);
        assert!(vm.can_undo());
        vm.undo();
        assert!(!vm.can_undo());
        assert_eq!(vm.state().feeds, initial.feeds);
        assert_eq!(vm.state().subscription_order, initial.subscription_order);
        assert_eq!(vm.state().articles.as_ptr(), article_allocation);
        assert_eq!(
            vm.state()
                .articles
                .iter()
                .map(|article| article.content.as_ptr())
                .collect::<Vec<_>>(),
            body_allocations
        );
        assert_eq!(vm.library_generation(), generation);
        for (before, after) in shared.iter().zip(vm.visible_shared()) {
            assert!(Rc::ptr_eq(before, &after));
        }
    }
    drop((counts, visible, epochs));
}

#[test]
fn aliased_order_snapshots_deduplicate_only_when_restored_against_live_memberships() {
    let mut state = state();
    let mut undo = UndoEntry {
        changes: vec![UndoChange::SubscriptionOrder(SubscriptionOrder {
            sort: SubscriptionSort::Manual,
            root: vec![
                Node::Feed(99),
                Node::Feed(1),
                Node::Folder("Folder".into()),
                Node::Feed(3),
            ],
            folders: BTreeMap::from([
                ("Folder".into(), vec![98, 3, 4, 2]),
                ("Removed".into(), vec![99]),
            ]),
        })],
        snapshot: None,
    };
    let aliases = Aliases {
        feeds: BTreeMap::from([(99, 1), (98, 3)]),
        ..Aliases::default()
    };
    undo.remap(&aliases);
    let pointer = state.articles.as_ptr();
    let (changes, _) = undo.restore(&mut state).unwrap();
    assert!(changes.order && changes.changed);
    assert!(!changes.feeds && !changes.settings);
    assert!(changes.articles.is_empty() && changes.removed.is_empty());
    assert_eq!(
        state.subscription_order.root,
        [Node::Feed(1), Node::Folder("Folder".into())]
    );
    assert_eq!(
        state.subscription_order.folders,
        BTreeMap::from([("Folder".into(), vec![3, 4])])
    );
    assert_eq!(state.articles.as_ptr(), pointer);
    assert!(!state.normalize_subscription_order());
}
