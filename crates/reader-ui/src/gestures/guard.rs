use super::motion::Contacts;
use crate::session::Session;
use ferroui_base::input::{
    HoldingState, IPointer, InputElement, Key, MouseButton, Pointer, PointerType,
};
use ferroui_base::interactivity::RoutingStrategies;
use ferroui_base::Ref;
use ferroui_controls::TopLevel;
use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

#[derive(Default)]
struct Guard {
    contacts: RefCell<Contacts>,
    pointers: RefCell<Vec<Weak<dyn IPointer>>>,
}
ferroui_base::ferro_thread_local! {
    static ACTIVE: RefCell<Weak<Guard>> = const { RefCell::new(Weak::new()) };
}
impl Guard {
    fn cancel(&self) -> bool {
        self.contacts.borrow_mut().clear();
        let pointers = self.pointers.take();
        let mut active = false;
        for pointer in pointers.into_iter().filter_map(|p| p.upgrade()) {
            active |= pointer.captured().is_some()
                || pointer
                    .as_any()
                    .downcast_ref::<Pointer>()
                    .is_some_and(|p| p.captured_gesture_recognizer().is_some());
            pointer.capture(None);
        }
        active
    }
}
/// Modal changes and Escape cancel a native pan/hold before another surface owns input.
pub(crate) fn cancel_active() -> bool {
    let guard = ACTIVE.with(|active| active.borrow().upgrade());
    guard.is_some_and(|guard| guard.cancel())
}

pub(crate) fn attach(root: &Ref<TopLevel>, ui: Weak<Session>) -> Box<dyn FnOnce()> {
    let guard = Rc::new(Guard::default());
    ACTIVE.with(|active| *active.borrow_mut() = Rc::downgrade(&guard));
    let pressed_guard = guard.clone();
    let pressed_ui = ui.clone();
    let pressed = root.add_handler_with(
        InputElement::pointer_pressed_event(),
        move |_, event| {
            if let Some(ui) = pressed_ui.upgrade() {
                ui.cancel_prefix();
            }
            if event.pointer().type_() == PointerType::Mouse
                || !event.properties().is_left_button_pressed
            {
                return;
            }
            let pointer = event.pointer();
            let position = event.get_position(None);
            let eligible = pressed_guard.contacts.borrow_mut().press(
                pointer.id(),
                pointer.is_primary(),
                position.x,
                position.y,
            );
            let mut pointers = pressed_guard.pointers.borrow_mut();
            pointers.retain(|weak| weak.upgrade().is_some_and(|p| p.id() != pointer.id()));
            let previous: Vec<_> = if eligible {
                Vec::new()
            } else {
                pointers.iter().filter_map(Weak::upgrade).collect()
            };
            pointers.push(Rc::downgrade(pointer));
            drop(pointers);
            if !eligible {
                event.set_handled(true);
                // Cancel the first finger's held drag/pan before a second contact can
                // route to a different control. Capture loss resets native pressed state.
                for previous in previous {
                    previous.capture(None);
                }
            }
        },
        RoutingStrategies::TUNNEL,
        true,
    );
    let moving = guard.clone();
    let moved = root.add_handler_with(
        InputElement::pointer_moved_event(),
        move |_, event| {
            if event.pointer().type_() == PointerType::Mouse {
                return;
            }
            let position = event.get_position(None);
            moving
                .contacts
                .borrow_mut()
                .moved(event.pointer().id(), position.x, position.y);
        },
        RoutingStrategies::TUNNEL,
        true,
    );
    let releasing = guard.clone();
    let released = root.add_handler_with(
        InputElement::pointer_released_event(),
        move |_, event| {
            if event.pointer().type_() == PointerType::Mouse
                || event.initial_press_mouse_button() != MouseButton::Left
            {
                return;
            }
            let position = event.get_position(None);
            let tap = releasing.contacts.borrow_mut().release(
                event.pointer().id(),
                position.x,
                position.y,
            );
            if !tap {
                event.set_handled(true);
                // A pan at a scroll range boundary may never be captured by a scroll
                // recognizer. Clear the original native Button's pressed state too.
                event.pointer().capture(None);
            }
        },
        RoutingStrategies::TUNNEL,
        true,
    );
    let losing = guard.clone();
    let lost = root.add_handler_with(
        InputElement::pointer_capture_lost_event(),
        move |_, event| {
            losing.contacts.borrow_mut().lost(event.pointer().id());
        },
        RoutingStrategies::DIRECT,
        true,
    );
    let holding = guard.clone();
    let held = root.add_handler_with(
        InputElement::holding_event(),
        move |_, event| {
            if event.holding_state() == HoldingState::Started {
                holding
                    .contacts
                    .borrow_mut()
                    .hold(event.pointer_event_args().pointer().id());
            }
        },
        RoutingStrategies::BUBBLE,
        true,
    );
    let canceling = guard.clone();
    let key = root.add_handler_with(
        InputElement::key_down_event(),
        move |_, event| {
            if crate::keyboard::editing_source(event)
                || event.key_modifiers.intersects(
                    ferroui_base::input::KeyModifiers::CONTROL
                        | ferroui_base::input::KeyModifiers::META
                        | ferroui_base::input::KeyModifiers::ALT,
                )
            {
                if let Some(ui) = ui.upgrade() {
                    ui.cancel_prefix();
                }
            }
            if event.key == Key::Escape && canceling.cancel() {
                event.set_handled(true);
            }
        },
        RoutingStrategies::TUNNEL,
        false,
    );
    let weak = root.downgrade();
    Box::new(move || {
        guard.cancel();
        if let Some(root) = weak.upgrade() {
            root.remove_handler(InputElement::pointer_pressed_event(), pressed);
            root.remove_handler(InputElement::pointer_moved_event(), moved);
            root.remove_handler(InputElement::pointer_released_event(), released);
            root.remove_handler(InputElement::pointer_capture_lost_event(), lost);
            root.remove_handler(InputElement::holding_event(), held);
            root.remove_handler(InputElement::key_down_event(), key);
        }
    })
}
