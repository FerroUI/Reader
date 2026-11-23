use reader_domain::AppState;
use reader_state::AppViewModel;
use std::rc::Rc;

#[test]
fn keeping_an_unread_item_updates_its_checkbox_and_blocks_scroll_marking() {
    let mut state = AppState::sample(1_800_000_000);
    state.settings.mark_read_on_scroll = true;
    let id = state.articles[0].id;
    state.articles[0].read = false;
    let vm = AppViewModel::new(state, 1_800_000_001, Rc::new(|_| Ok(())));
    let revision = vm.revision();
    assert!(!vm.is_kept_unread(id));
    vm.keep_unread(id);
    assert!(!vm.article(id).unwrap().read);
    assert!(vm.is_kept_unread(id));
    assert!(
        vm.revision() > revision,
        "an unchanged read flag still updates the explicit Keep unread state"
    );
    let revision = vm.revision();
    vm.keep_unread(id);
    assert_eq!(
        vm.revision(),
        revision,
        "setting the same kept flag is idempotent"
    );
    vm.select_from_scroll(id);
    assert!(!vm.article(id).unwrap().read);
    vm.set_read(id, true);
    assert!(!vm.is_kept_unread(id));
    assert!(vm.article(id).unwrap().read);
    vm.toggle_read(id);
    assert!(vm.is_kept_unread(id));
    assert!(!vm.article(id).unwrap().read);
    vm.toggle_read(id);
    assert!(!vm.is_kept_unread(id));
    assert!(vm.article(id).unwrap().read);
    let revision = vm.revision();
    vm.keep_unread(u64::MAX);
    assert!(!vm.is_kept_unread(u64::MAX));
    assert_eq!(vm.revision(), revision);
}
