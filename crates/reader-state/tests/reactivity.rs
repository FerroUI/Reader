use ferroui_dynamic_data::IObservableCache;
use ferroui_reactive::ObservableExt;
use ferroui_reactiveui::IReactiveNotifyPropertyChanged;
use reader_domain::{AppState, View};
use reader_state::AppViewModel;
use std::{cell::Cell, rc::Rc};

#[test]
fn reentrant_property_notification_queues_mutation_and_keeps_caches_consistent() {
    let vm = AppViewModel::new(AppState::sample(1000), 1000, Rc::new(|_| Ok(())));
    let first = vm.state().articles[0].id;
    let second = vm.state().articles[1].id;
    let fired = Rc::new(Cell::new(false));
    let once = fired.clone();
    let weak = Rc::downgrade(&vm);
    let subscription = vm.changed().subscribe_next(move |_| {
        if !once.replace(true) {
            if let Some(vm) = weak.upgrade() {
                vm.toggle_star(second);
            }
        }
    });
    vm.toggle_star(first);
    assert!(fired.get());
    assert!(vm.articles.lookup(&first).unwrap().starred);
    assert!(vm.articles.lookup(&second).unwrap().starred);
    vm.navigate(View::Starred);
    assert_eq!(vm.visible.count(), 3);
    assert_eq!(vm.total_count(&View::Starred), 3);
    subscription.dispose();
}

#[test]
fn unchanged_article_snapshots_are_shared_across_mutations_and_notifications() {
    let vm = AppViewModel::new(AppState::sample(1000), 1000, Rc::new(|_| Ok(())));
    vm.navigate(View::All);
    let before = vm.visible_shared();
    let changed = before[0].id;
    vm.toggle_star(changed);
    let after = vm.visible_shared();
    assert!(!Rc::ptr_eq(&before[0], &after[0]));
    for (old, new) in before.iter().zip(&after).skip(1) {
        assert!(Rc::ptr_eq(old, new));
    }
    vm.set_status("Background refresh complete".into());
    for (old, new) in after.iter().zip(vm.visible_shared()) {
        assert!(Rc::ptr_eq(old, &new));
    }
    vm.set_read(changed, true);
    assert!(!Rc::ptr_eq(&after[0], &vm.article_shared(changed).unwrap()));
    for old in after.iter().skip(1) {
        assert!(Rc::ptr_eq(old, &vm.article_shared(old.id).unwrap()));
    }
    let cache = vm.visible.lookup(&changed).unwrap();
    assert!(Rc::ptr_eq(&cache, &vm.articles.lookup(&changed).unwrap()));
}
