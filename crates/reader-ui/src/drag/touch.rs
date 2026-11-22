use super::{highlight, release_click, suppress, DragSession};
use crate::gestures::motion::Motion;
use ferroui_base::input::gesture_recognizers::GestureRecognizer;
use ferroui_base::input::{HoldingState, IPointer, InputElement, PointerType};
use ferroui_base::reactive::Subscription;
use ferroui_base::threading::{DispatcherPriority, DispatcherTimer};
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
    time::Duration,
};

#[repr(C)]
pub(super) struct TouchDrag {
    pub base: GestureRecognizer,
    pub button: WeakRef<Button>,
    pub vm: Weak<AppViewModel>,
    pub id: Id,
    pub contact: RefCell<Option<Contact>>,
    pub highlight: RefCell<Option<WeakRef<Control>>>,
    pub timer: RefCell<Option<Subscription>>,
}
ferro_class!(TouchDrag: GestureRecognizer);
ferro_impl_classes!(TouchDrag: FerroObjectImpl, StyledElementImpl);

pub(super) struct Contact {
    pub pointer: Rc<dyn IPointer>,
    pub motion: Motion,
    pub drag: DragSession,
    pub held: bool,
    pub moved: bool,
    pub position: Point,
}

impl TouchDrag {
    pub fn cancel(&self) {
        self.timer.borrow_mut().take();
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
    fn hold(&self, pointer: &Rc<dyn IPointer>) -> bool {
        let (Some(vm), Some(button)) = (self.vm.upgrade(), self.button.upgrade()) else {
            return false;
        };
        if !button.is_attached_to_visual_tree() {
            return false;
        }
        {
            let mut current = self.contact.borrow_mut();
            let Some(contact) = current.as_mut().filter(|c| c.pointer.id() == pointer.id()) else {
                return false;
            };
            if !contact.drag.valid(&vm) || !contact.motion.within_tap() {
                return false;
            }
            contact.held = true;
        }
        suppress(self.id);
        self.capture(pointer);
        true
    }
    pub(super) fn hover(&self) -> bool {
        let snapshot = self
            .contact
            .borrow()
            .as_ref()
            .filter(|contact| contact.held)
            .map(|contact| (contact.drag.clone(), contact.position, contact.moved));
        let Some((drag, point, moved)) = snapshot else {
            return false;
        };
        let (Some(vm), Some(button)) = (self.vm.upgrade(), self.button.upgrade()) else {
            self.cancel();
            return false;
        };
        if !drag.valid(&vm) || !button.is_attached_to_visual_tree() {
            self.cancel();
            return false;
        }
        if !moved {
            return true;
        }
        let Some(root) = TopLevel::get_top_level(Some(&button)) else {
            self.cancel();
            return false;
        };
        let target = super::target_at(&root, &vm, point, &drag);
        highlight(&self.highlight, target.as_ref());
        super::feedback::touch_scroll(&button, &root, point);
        true
    }
}

pub(super) fn attach(button: &Ref<Button>, vm: Rc<AppViewModel>, id: Id) {
    let recognizer = instantiate(TouchDrag {
        base: GestureRecognizer::construct(),
        button: button.downgrade(),
        vm: Rc::downgrade(&vm),
        id,
        contact: RefCell::new(None),
        highlight: RefCell::new(None),
        timer: RefCell::new(None),
    });
    button.gesture_recognizers().add(&recognizer);
    InputElement::set_is_holding_enabled(button, true);
    let weak = recognizer.downgrade();
    let holding = button.add_handler(InputElement::holding_event(), move |_, event| {
        if event.holding_state() == HoldingState::Started
            && matches!(event.pointer_type(), PointerType::Touch | PointerType::Pen)
        {
            if let Some(recognizer) = weak.upgrade() {
                if recognizer.hold(event.pointer_event_args().pointer()) {
                    let weak = recognizer.downgrade();
                    *recognizer.timer.borrow_mut() = Some(DispatcherTimer::run(
                        move || weak.upgrade().is_some_and(|recognizer| recognizer.hover()),
                        Duration::from_millis(45),
                        DispatcherPriority::BACKGROUND,
                    ));
                    event.set_handled(true);
                }
            }
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
