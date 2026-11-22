use super::{highlight, release_click, suppress, DragSession};
use ferroui_base::input::gesture_recognizers::GestureRecognizer;
use ferroui_base::input::{HoldingState, IPointer, InputElement, Key, PointerType};
use ferroui_base::interactivity::RoutingStrategies;
use ferroui_base::{
    ferro_class, ferro_impl_classes, instantiate, FerroObjectImpl, Point, Ref, StyledElementImpl,
    WeakRef,
};
use ferroui_controls::{Button, Control, TopLevel};
use ferroui_view::on_cleanup;
use reader_domain::Id;
use reader_state::AppViewModel;
use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

#[repr(C)]
pub(super) struct TouchDrag {
    pub base: GestureRecognizer,
    pub button: WeakRef<Button>,
    pub vm: Weak<AppViewModel>,
    pub id: Id,
    pub contact: RefCell<Option<Contact>>,
    pub highlight: RefCell<Option<WeakRef<Control>>>,
}
ferro_class!(TouchDrag: GestureRecognizer);
ferro_impl_classes!(TouchDrag: FerroObjectImpl, StyledElementImpl);

pub(super) struct Contact {
    pub pointer: Rc<dyn IPointer>,
    pub start: Point,
    pub drag: DragSession,
    pub held: bool,
    pub moved: bool,
}
type Guard = Rc<RefCell<Option<Box<dyn FnOnce()>>>>;

impl TouchDrag {
    pub fn cancel(&self) {
        let contact = self.contact.borrow_mut().take();
        highlight(&self.highlight, None);
        if let Some(contact) = contact {
            contact.drag.active.set(false);
            if contact.held {
                contact.pointer.capture(None);
                release_click(self.id);
            }
        }
    }
    fn hold(&self, pointer: &Rc<dyn IPointer>) {
        let (Some(vm), Some(button)) = (self.vm.upgrade(), self.button.upgrade()) else {
            return;
        };
        if !button.is_attached_to_visual_tree() {
            return;
        }
        {
            let mut current = self.contact.borrow_mut();
            let Some(contact) = current.as_mut().filter(|c| c.pointer.id() == pointer.id()) else {
                return;
            };
            if !contact.drag.valid(&vm) {
                return;
            }
            contact.held = true;
        }
        suppress(self.id);
        self.capture(pointer);
    }
}

fn guard_contacts(button: &Button, recognizer: &Ref<TouchDrag>, slot: &Guard) {
    if let Some(remove) = slot.borrow_mut().take() {
        remove();
    }
    let Some(root) = TopLevel::get_top_level(Some(button)) else {
        return;
    };
    let weak = recognizer.downgrade();
    let pressed = root.add_handler_with(
        InputElement::pointer_pressed_event(),
        move |_, event| {
            if let Some(recognizer) = weak.upgrade() {
                let other = recognizer
                    .contact
                    .borrow()
                    .as_ref()
                    .is_some_and(|contact| contact.pointer.id() != event.pointer().id());
                if other {
                    recognizer.cancel();
                }
            }
        },
        RoutingStrategies::TUNNEL,
        true,
    );
    let weak = recognizer.downgrade();
    let key = root.add_handler_with(
        InputElement::key_down_event(),
        move |_, event| {
            if event.key == Key::Escape {
                if let Some(recognizer) = weak.upgrade() {
                    recognizer.cancel();
                }
            }
        },
        RoutingStrategies::TUNNEL,
        true,
    );
    let weak = root.downgrade();
    *slot.borrow_mut() = Some(Box::new(move || {
        if let Some(root) = weak.upgrade() {
            root.remove_handler(InputElement::pointer_pressed_event(), pressed);
            root.remove_handler(InputElement::key_down_event(), key);
        }
    }));
}

pub(super) fn attach(button: &Ref<Button>, vm: Rc<AppViewModel>, id: Id) {
    let recognizer = instantiate(TouchDrag {
        base: GestureRecognizer::construct(),
        button: button.downgrade(),
        vm: Rc::downgrade(&vm),
        id,
        contact: RefCell::new(None),
        highlight: RefCell::new(None),
    });
    button.gesture_recognizers().add(&recognizer);
    InputElement::set_is_holding_enabled(button, true);
    let weak = recognizer.downgrade();
    let holding = button.add_handler(InputElement::holding_event(), move |_, event| {
        if event.holding_state() == HoldingState::Started
            && matches!(event.pointer_type(), PointerType::Touch | PointerType::Pen)
        {
            if let Some(recognizer) = weak.upgrade() {
                event.set_handled(true);
                recognizer.hold(event.pointer_event_args().pointer());
            }
        }
    });
    let guard: Guard = Rc::new(RefCell::new(None));
    guard_contacts(button, &recognizer, &guard);
    let weak = recognizer.downgrade();
    let on_attach_guard = guard.clone();
    let attached = button.attached_to_visual_tree(move |_| {
        if let Some(recognizer) = weak.upgrade() {
            if let Some(button) = recognizer.button.upgrade() {
                guard_contacts(&button, &recognizer, &on_attach_guard);
            }
        }
    });
    let weak = recognizer.downgrade();
    let detach_guard = guard.clone();
    let detached = button.detached_from_visual_tree(move |_| {
        if let Some(recognizer) = weak.upgrade() {
            recognizer.cancel();
        }
        if let Some(remove) = detach_guard.borrow_mut().take() {
            remove();
        }
    });
    let weak = button.downgrade();
    on_cleanup(move || {
        attached.dispose();
        detached.dispose();
        recognizer.cancel();
        if let Some(remove) = guard.borrow_mut().take() {
            remove();
        }
        if let Some(button) = weak.upgrade() {
            button.remove_handler(InputElement::holding_event(), holding);
            button.gesture_recognizers().remove(&recognizer);
        }
    });
}
