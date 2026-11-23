//! A constrained popup can cover its opener. Re-hit-testing a resting mouse
//! after layout must not replace the item explicitly selected by the keyboard.
use ferroui_base::input::InputElement;
use ferroui_base::interactivity::{RoutedEventArgs, RoutingStrategies};
use ferroui_base::{FerroObject, Ref};
use ferroui_controls::{ContextMenu, Control, MenuBase, MenuItem};
use ferroui_view::on_cleanup;

pub(super) const KEYBOARD_OPEN: &str = "reader-keyboard-menu";

pub(super) fn attach(menu: &Ref<ContextMenu>) {
    // These handlers are installed before the native interaction handler is
    // attached on popup opening. Preserve its behavior for actual input.
    let weak = menu.downgrade();
    let entered = menu.add_handler(MenuItem::pointer_entered_item_event(), move |_, event| {
        if weak
            .upgrade()
            .is_some_and(|menu| menu.classes().contains(KEYBOARD_OPEN))
        {
            event.set_handled(true);
        }
    });
    let weak = menu.downgrade();
    let exited = menu.add_handler(MenuItem::pointer_exited_item_event(), move |_, event| {
        if weak
            .upgrade()
            .is_some_and(|menu| menu.classes().contains(KEYBOARD_OPEN))
        {
            event.set_handled(true);
        }
    });
    let weak = menu.downgrade();
    let moved = menu.add_handler_with(
        InputElement::pointer_moved_event(),
        move |_, event| {
            if let Some(menu) = weak.upgrade() {
                resume(&menu, event.source());
            }
        },
        RoutingStrategies::TUNNEL,
        false,
    );
    let weak = menu.downgrade();
    let pressed = menu.add_handler_with(
        InputElement::pointer_pressed_event(),
        move |_, event| {
            if let Some(menu) = weak.upgrade() {
                resume(&menu, event.source());
            }
        },
        RoutingStrategies::TUNNEL,
        false,
    );
    let weak = menu.downgrade();
    let closed = menu.closed(move |_, _| {
        if let Some(menu) = weak.upgrade() {
            menu.classes().set(KEYBOARD_OPEN, false);
        }
    });
    let weak = menu.downgrade();
    on_cleanup(move || {
        if let Some(menu) = weak.upgrade() {
            menu.remove_handler(MenuItem::pointer_entered_item_event(), entered);
            menu.remove_handler(MenuItem::pointer_exited_item_event(), exited);
            menu.remove_handler(InputElement::pointer_moved_event(), moved);
            menu.remove_handler(InputElement::pointer_pressed_event(), pressed);
            menu.remove_handler(MenuBase::closed_event(), closed);
        }
    });
}

fn resume(menu: &ContextMenu, source: Option<Ref<FerroObject>>) {
    if !menu.classes().contains(KEYBOARD_OPEN) {
        return;
    }
    menu.classes().set(KEYBOARD_OPEN, false);
    // Pointer-enter precedes pointer-move in native routing. Replay that native
    // item event once so the first deliberate move immediately resumes hover.
    if let Some(item) = source
        .and_then(|source| source.cast::<Control>())
        .and_then(|control| control.find_logical_ancestor_of_type::<MenuItem>(true))
        .filter(|item| item.is_effectively_enabled())
    {
        item.raise_event(&RoutedEventArgs::with_event(
            MenuItem::pointer_entered_item_event(),
        ));
    }
}
