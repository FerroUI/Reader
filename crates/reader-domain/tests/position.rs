use reader_domain::{Action, AppState, Id};

fn ids(state: &AppState) -> Vec<Id> {
    state.feeds.iter().map(|feed| feed.id).collect()
}
fn position(id: Id, target: Id, before: bool, folder: Option<&str>) -> Action {
    Action::PositionFeed {
        id,
        target,
        before,
        folder: folder.map(str::to_owned),
    }
}

#[test]
fn positioning_preserves_feed_identity_annotations_indexes_and_backup_order() {
    let mut state = AppState::sample(1000);
    let old = state.clone();
    let order = ids(&state);
    assert!(
        state
            .apply(position(order[3], order[0], true, None), 1000)
            .unwrap()
            .changed
    );
    assert_eq!(
        ids(&state),
        [order[3], order[0], order[1], order[2], order[4], order[5]]
    );
    assert_eq!(state.articles, old.articles);
    for feed in &old.feeds {
        assert_eq!(state.feed(feed.id), Some(feed));
    }
    assert_eq!(
        AppState::from_json(&state.to_json().unwrap()).unwrap(),
        state
    );
    state
        .apply(position(order[3], order[5], false, None), 1000)
        .unwrap();
    assert_eq!(
        ids(&state),
        [order[0], order[1], order[2], order[4], order[5], order[3]]
    );
}

#[test]
fn stale_invalid_and_self_targets_are_atomic_and_do_not_mutate_membership() {
    let mut state = AppState::sample(1000);
    let original = state.clone();
    let a = state.feeds[0].id;
    let b = state.feeds[1].id;
    for action in [
        position(99_999, b, true, None),
        position(a, 99_999, false, Some("Science")),
        position(a, b, true, Some("Design")),
        position(a, b, true, Some("")),
    ] {
        assert!(state.apply(action, 1000).is_err());
        assert_eq!(state, original);
    }
    assert!(
        !state
            .apply(position(a, a, false, Some("Science")), 1000)
            .unwrap()
            .changed
    );
    assert_eq!(state, original);
    assert!(
        !state
            .apply(position(a, b, true, None), 1000)
            .unwrap()
            .changed
    );
    assert_eq!(state, original);
}
