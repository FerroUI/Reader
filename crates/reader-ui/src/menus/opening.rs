use ferroui_base::input::NavigationDirection;
use ferroui_base::threading::{Dispatcher, DispatcherPriority};
use ferroui_base::Ref;
use ferroui_controls::{Button, Control, MenuBase};
use ferroui_view::on_cleanup;

/// The opener remains visibly pressed until native dismissal completes. Keep
/// this tied to the actual popup lifecycle, including Escape and browser Back.
pub(super) fn track(button: &Ref<Button>) {
    let Some(menu) = button.context_menu() else {
        return;
    };
    let weak_button = button.downgrade();
    let opened = menu.opened(move |_, _| {
        if let Some(button) = weak_button.upgrade() {
            button.classes().set("reader-menu-open", true);
        }
    });
    let weak_button = button.downgrade();
    let closed = menu.closed(move |_, _| {
        if let Some(button) = weak_button.upgrade() {
            button.classes().set("reader-menu-open", false);
        }
    });
    let weak_menu = menu.downgrade();
    let weak_button = button.downgrade();
    on_cleanup(move || {
        if let Some(menu) = weak_menu.upgrade() {
            menu.remove_handler(MenuBase::opened_event(), opened);
            menu.remove_handler(MenuBase::closed_event(), closed);
        }
        if let Some(button) = weak_button.upgrade() {
            button.classes().set("reader-menu-open", false);
        }
    });
}

/// Arrow keys open the dropdown and select its first/last enabled native item.
/// Defer selection until the popup's presenter has completed its first layout.
pub(super) fn with_selection(button: &Control, forward: bool) {
    let Some(menu) = button.context_menu() else {
        return;
    };
    menu.classes().set(super::pointer::KEYBOARD_OPEN, true);
    super::request(button);
    if !menu.is_open() {
        menu.classes().set(super::pointer::KEYBOARD_OPEN, false);
        return;
    }
    let weak = menu.downgrade();
    Dispatcher::ui_thread().post_local(
        move || {
            if let Some(menu) = weak.upgrade().filter(|menu| menu.is_open()) {
                menu.move_selection(
                    if forward {
                        NavigationDirection::First
                    } else {
                        NavigationDirection::Last
                    },
                    true,
                    false,
                );
            }
        },
        DispatcherPriority::BACKGROUND,
    );
}
