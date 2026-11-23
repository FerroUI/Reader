use reader_domain::AppState;
use reader_state::{AppViewModel, HostRequest, HostServices};
use std::{cell::Cell, rc::Rc};

#[test]
fn update_notice_is_transient_and_does_not_replace_a_reading_status() {
    let vm = AppViewModel::new(AppState::new(), 0, Rc::new(|_| Ok(())));
    let saved = vm.export_json().unwrap();
    vm.set_status("Subscription refreshed.".into());
    vm.set_update_notice("An update is ready.".into(), false);
    assert_eq!(vm.status(), "Subscription refreshed.");
    assert_eq!(vm.export_json().unwrap(), saved);
    vm.dismiss_update();
    assert!(vm.update_notice().is_none());
    assert_eq!(vm.status(), "Subscription refreshed.");
}

#[test]
fn updating_is_explicit_and_does_not_dispatch_duplicate_requests() {
    let vm = AppViewModel::new(AppState::new(), 0, Rc::new(|_| Ok(())));
    let requests = Rc::new(Cell::new(0));
    let received = requests.clone();
    vm.set_services(HostServices {
        request: Rc::new(move |request| {
            if matches!(request, HostRequest::ApplyUpdate) {
                received.set(received.get() + 1);
            }
        }),
    });
    vm.apply_update();
    assert_eq!(requests.get(), 0);
    vm.set_update_notice("An update is ready.".into(), false);
    assert_eq!(requests.get(), 0);
    vm.apply_update();
    vm.apply_update();
    vm.dismiss_update();
    assert_eq!(requests.get(), 1);
    assert!(vm.update_notice().unwrap().applying);
    vm.set_update_notice("Close other Reader tabs.".into(), false);
    vm.apply_update();
    assert_eq!(requests.get(), 2);
}
