use reader_domain::{AppState, View};
use reader_state::AppViewModel;
use std::rc::Rc;

fn fixture() -> Rc<AppViewModel> {
    let mut state = AppState::sample(1_800_000_000);
    let template = state.articles[0].clone();
    state.articles = (0..54)
        .map(|index| {
            let mut article = template.clone();
            article.id = 1_000 + index;
            article.guid = format!("marking-{index}");
            article.feed_id = state.feeds[usize::from(index >= 51)].id;
            article.read = false;
            article.read_at = None;
            article.published = if index < 25 { 100 } else { 200 };
            article
        })
        .collect();
    state.next_id = 2_000;
    AppViewModel::new(state, 300, Rc::new(|_| Ok(())))
}

#[test]
fn confirmation_uses_current_stream_and_strict_fifty_item_boundary() {
    let vm = fixture();
    let feeds = vm.feeds();
    vm.navigate(View::Feed(feeds[0].id));
    assert_eq!(vm.count(&vm.view()), 51);
    assert!(vm.should_confirm_mark_all());
    vm.set_read(1_000, true);
    assert_eq!(vm.count(&vm.view()), 50);
    assert!(!vm.should_confirm_mark_all());
    vm.navigate(View::Feed(feeds[1].id));
    assert_eq!(vm.count(&vm.view()), 3);
    assert!(!vm.should_confirm_mark_all());
    vm.navigate(View::All);
    assert!(vm.should_confirm_mark_all());
    let mut settings = vm.state().settings.clone();
    settings.confirm_mark_all = false;
    vm.set_settings(settings);
    assert!(!vm.should_confirm_mark_all());
}

#[test]
fn dated_marking_preserves_cutoff_scope_and_undo() {
    let vm = fixture();
    let feed = vm.feeds()[0].id;
    vm.navigate(View::Feed(feed));
    let before = vm.state().articles.clone();
    vm.mark_all_read(Some(200));
    let state = vm.state();
    assert_eq!(state.articles.iter().filter(|a| a.read).count(), 25);
    assert!(state
        .articles
        .iter()
        .filter(|a| a.published == 200)
        .all(|a| !a.read));
    assert!(state
        .articles
        .iter()
        .filter(|a| a.feed_id != feed)
        .all(|a| !a.read));
    drop(state);
    vm.undo();
    assert_eq!(vm.state().articles, before);
    assert_eq!(vm.view(), View::Feed(feed));
}
