use super::{UndoChange, UndoEntry};
use crate::AppViewModel;
use reader_domain::{Action, AppState, Article, Feed, FullArticle};
use std::{cell::Cell, rc::Rc};

fn state() -> AppState {
    let mut state = AppState::new();
    state.feeds.push(Feed {
        id: 1,
        title: "Feed".into(),
        url: "https://example.com/feed".into(),
        ..Feed::default()
    });
    state.articles.push(Article {
        id: 2,
        feed_id: 1,
        guid: "post".into(),
        url: "https://example.com/post".into(),
        title: "Before removal".into(),
        content: "Original feed body".into(),
        starred: true,
        tags: vec!["Original".into()],
        note: "Original note".into(),
        ..Article::default()
    });
    state.normalize().unwrap();
    state
}

#[test]
fn same_url_restoration_retains_both_annotation_sets_and_newest_full_content() {
    let mut state = state();
    let undo = UndoEntry::capture(&state, &Action::RemoveFeed(1)).unwrap();
    state.apply(Action::RemoveFeed(1), 100).unwrap();
    state.feeds[0].archived = false;
    state.articles[0].title = "Updated publisher title".into();
    state.articles[0].read = true;
    state.articles[0].read_at = Some(200);
    state.articles[0].tags = vec!["Current".into()];
    state.articles[0].note = "Current note".into();
    let full = FullArticle::new(
        "<p>Current full article</p>".into(),
        "https://example.com/post",
        "https://example.com/post",
        200,
    )
    .unwrap();
    state.articles[0].full_content = Some(Rc::new(full.clone()));
    state.articles[0].show_full_content = true;
    state.normalize().unwrap();
    let (changes, _) = undo.restore(&mut state).unwrap();
    assert_eq!(changes.articles, [2]);
    let article = state.article(2).unwrap();
    assert_eq!(article.title, "Updated publisher title");
    assert!(article.starred && article.read);
    assert_eq!(article.read_at, Some(200));
    assert_eq!(article.tags, ["Current", "Original"]);
    assert_eq!(article.note, "Current note\n\nOriginal note");
    assert_eq!(article.full_content.as_deref(), Some(&full));
    assert!(article.show_full_content);
    AppState::from_json(&state.to_json().unwrap()).unwrap();
}

#[test]
fn normalization_failure_is_transactional_and_keeps_failed_undo_available() {
    let original = state();
    let before = original.to_json().unwrap();
    let writes = Rc::new(Cell::new(0));
    let count = writes.clone();
    let vm = AppViewModel::new(
        original,
        100,
        Rc::new(move |_| {
            count.set(count.get() + 1);
            Ok(())
        }),
    );
    vm.undo.borrow_mut().push_back(UndoEntry {
        changes: vec![
            UndoChange::Title(1, "Must be rolled back".into()),
            UndoChange::RestoreFeed(
                Feed {
                    id: 10,
                    url: "javascript:invalid".into(),
                    ..Feed::default()
                },
                0,
                vec![],
            ),
        ],
        snapshot: None,
    });
    vm.undo();
    assert_eq!(vm.export_json().unwrap(), before);
    assert_eq!(writes.get(), 0);
    assert!(vm.can_undo());
    assert!(vm.status().contains("Could not undo this change"));
    assert!(vm.status().contains("kept unchanged"));
    assert_eq!(vm.state().feed(1).unwrap().title, "Feed");
    assert_eq!(vm.state().article(2).unwrap().feed_id, 1);
}

#[test]
fn identity_collision_never_replaces_an_unrelated_item() {
    let mut state = state();
    let before = state.to_json().unwrap();
    let undo = UndoEntry {
        changes: vec![UndoChange::RestoreFeed(
            Feed {
                id: 2,
                url: "https://different.example/feed".into(),
                ..Feed::default()
            },
            0,
            vec![],
        )],
        snapshot: None,
    };
    assert!(undo.restore(&mut state).is_err());
    assert_eq!(state.to_json().unwrap(), before);
    assert_eq!(state.article(2).unwrap().title, "Before removal");
}
