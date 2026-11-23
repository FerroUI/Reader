//! A stationary held release opens folder actions. It never becomes a feed
//! drag; ordinary movement remains owned by the native scroll recognizer.
use super::motion::Motion;
use ferroui_base::input::gesture_recognizers::{GestureRecognizer, GestureRecognizerImpl};
use ferroui_base::input::{
    HoldingState, IPointer, InputElement, PointerEventArgs, PointerPressedEventArgs,
    PointerReleasedEventArgs, PointerType,
};
use ferroui_base::{
    ferro_class, ferro_impl_classes, instantiate, FerroObjectImpl, Ref, StyledElementImpl, WeakRef,
};
use ferroui_controls::Button;
use ferroui_view::on_cleanup;
use std::{cell::RefCell, rc::Rc};

#[repr(C)]
struct HeldMenu {
    base: GestureRecognizer,
    button: WeakRef<Button>,
    valid: Rc<dyn Fn() -> bool>,
    contact: RefCell<Option<Contact>>,
}
ferro_class!(HeldMenu: GestureRecognizer);
ferro_impl_classes!(HeldMenu: FerroObjectImpl, StyledElementImpl);

struct Contact {
    pointer: Rc<dyn IPointer>,
    motion: Motion,
    held: bool,
}
impl HeldMenu {
    fn cancel(&self) {
        let contact = self.contact.borrow_mut().take();
        if let Some(contact) = contact {
            if contact.held {
                contact.pointer.capture(None);
            }
        }
    }
    fn hold(&self, pointer: &Rc<dyn IPointer>) -> bool {
        if !(self.valid)()
            || !self
                .button
                .upgrade()
                .is_some_and(|button| button.is_attached_to_visual_tree())
        {
            return false;
        }
        {
            let mut contact = self.contact.borrow_mut();
            let Some(contact) = contact.as_mut().filter(|contact| {
                contact.pointer.id() == pointer.id() && contact.motion.within_tap()
            }) else {
                return false;
            };
            contact.held = true;
        }
        // Gesture capture clears the native Button's pending click. Release is
        // delivered only here, so opening a menu cannot also navigate the folder.
        self.capture(pointer);
        true
    }
}
impl GestureRecognizerImpl for HeldMenu {
    fn pointer_pressed(this: &Self, event: &PointerPressedEventArgs) {
        if !matches!(
            event.pointer().type_(),
            PointerType::Touch | PointerType::Pen
        ) || !event.pointer().is_primary()
            || !(this.valid)()
        {
            return;
        }
        this.cancel();
        let position = event.get_position(None);
        *this.contact.borrow_mut() = Some(Contact {
            pointer: event.pointer().clone(),
            motion: Motion::new(position.x, position.y),
            held: false,
        });
    }
    fn pointer_moved(this: &Self, event: &PointerEventArgs) {
        let position = event.get_position(None);
        let mut current = this.contact.borrow_mut();
        let Some(contact) = current
            .as_mut()
            .filter(|contact| contact.pointer.id() == event.pointer().id())
        else {
            return;
        };
        contact.motion.update(position.x, position.y);
        if !contact.motion.within_tap() || !(this.valid)() {
            drop(current);
            this.cancel();
        }
    }
    fn pointer_released(this: &Self, event: &PointerReleasedEventArgs) {
        if !this
            .contact
            .borrow()
            .as_ref()
            .is_some_and(|contact| contact.pointer.id() == event.pointer().id())
        {
            return;
        }
        let mut contact = this.contact.borrow_mut().take().unwrap();
        if !contact.held {
            return;
        }
        event.set_handled(true);
        let position = event.get_position(None);
        contact.motion.update(position.x, position.y);
        let target = contact
            .motion
            .within_tap()
            .then(|| this.button.upgrade())
            .flatten()
            .filter(|button| button.is_attached_to_visual_tree() && (this.valid)());
        contact.pointer.capture(None);
        if let Some(button) = target {
            crate::menus::request(&button);
        }
    }
    fn pointer_capture_lost(this: &Self, pointer: &Rc<dyn IPointer>) {
        if this
            .contact
            .borrow()
            .as_ref()
            .is_some_and(|contact| contact.pointer.id() == pointer.id())
        {
            this.cancel();
        }
    }
}

pub(crate) fn attach(button: &Ref<Button>, valid: Rc<dyn Fn() -> bool>) {
    let recognizer = instantiate(HeldMenu {
        base: GestureRecognizer::construct(),
        button: button.downgrade(),
        valid,
        contact: RefCell::new(None),
    });
    button.gesture_recognizers().add(&recognizer);
    InputElement::set_is_holding_enabled(button, true);
    let weak = recognizer.downgrade();
    let holding = button.add_handler(InputElement::holding_event(), move |_, event| {
        if event.holding_state() == HoldingState::Started
            && matches!(event.pointer_type(), PointerType::Touch | PointerType::Pen)
            && weak
                .upgrade()
                .is_some_and(|recognizer| recognizer.hold(event.pointer_event_args().pointer()))
        {
            event.set_handled(true);
        }
    });
    let weak = recognizer.downgrade();
    let detached = button.detached_from_visual_tree(move |_| {
        if let Some(recognizer) = weak.upgrade() {
            recognizer.cancel();
        }
    });
    let weak = button.downgrade();
    on_cleanup(move || {
        detached.dispose();
        recognizer.cancel();
        if let Some(button) = weak.upgrade() {
            button.remove_handler(InputElement::holding_event(), holding);
            button.gesture_recognizers().remove(&recognizer);
        }
    });
}
