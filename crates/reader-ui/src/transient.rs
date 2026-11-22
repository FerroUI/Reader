//! Native menus participate in the same transient navigation stack as dialogs.
use ferroui_base::{Ref, WeakRef};
use ferroui_controls::{ContextMenu, MenuBase};
use ferroui_view::on_cleanup;
use std::cell::RefCell;

ferroui_base::ferro_thread_local! {
    static MENUS: RefCell<Vec<WeakRef<ContextMenu>>> = const { RefCell::new(Vec::new()) };
}
fn active() -> Vec<Ref<ContextMenu>> {
    MENUS.with(|menus| {
        let mut menus = menus.borrow_mut();
        menus.retain(|weak| weak.upgrade().is_some_and(|menu| menu.is_open()));
        menus.iter().filter_map(WeakRef::upgrade).collect()
    })
}
pub(crate) fn has_menu() -> bool {
    !active().is_empty()
}
pub(crate) fn dismiss_menu() -> bool {
    let Some(menu) = active().pop() else {
        return false;
    };
    crate::gestures::cancel_active();
    menu.close();
    true
}
pub(crate) fn close_menus() {
    for menu in active().into_iter().rev() {
        menu.close();
    }
}
pub(crate) fn attach_menu(menu: &Ref<ContextMenu>) {
    let weak = menu.downgrade();
    let opened = menu.opened(move |_, _| {
        let Some(menu) = weak.upgrade() else {
            return;
        };
        crate::gestures::cancel_active();
        MENUS.with(|menus| {
            let mut menus = menus.borrow_mut();
            menus.retain(|weak| {
                weak.upgrade()
                    .is_some_and(|current| current != menu && current.is_open())
            });
            menus.push(menu.downgrade());
        });
        crate::notify_navigation();
    });
    let closed = menu.closed(move |_, _| {
        active();
        crate::notify_navigation();
    });
    let weak = menu.downgrade();
    on_cleanup(move || {
        if let Some(menu) = weak.upgrade() {
            menu.close();
            menu.remove_handler(MenuBase::opened_event(), opened);
            menu.remove_handler(MenuBase::closed_event(), closed);
        }
        active();
        crate::notify_navigation();
    });
}
