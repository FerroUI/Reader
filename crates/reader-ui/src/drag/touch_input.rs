use super::{
    highlight, release_click, target_at,
    touch::{Contact, TouchDrag},
    DragSession,
};
use crate::gestures::motion::Motion;
use ferroui_base::input::gesture_recognizers::GestureRecognizerImpl;
use ferroui_base::input::{
    IPointer, PointerEventArgs, PointerPressedEventArgs, PointerReleasedEventArgs, PointerType,
};
use ferroui_controls::TopLevel;
use std::rc::Rc;

impl GestureRecognizerImpl for TouchDrag {
    fn pointer_pressed(this: &Self, event: &PointerPressedEventArgs) {
        if !matches!(
            event.pointer().type_(),
            PointerType::Touch | PointerType::Pen
        ) || !event.pointer().is_primary()
        {
            return;
        }
        this.cancel();
        let Some(vm) = this.vm.upgrade() else {
            return;
        };
        let Some(ui) = this.ui.upgrade() else { return };
        let Some(drag) = DragSession::new(&vm, &ui, &this.node) else {
            return;
        };
        let start = event.get_position(None);
        *this.contact.borrow_mut() = Some(Contact {
            pointer: event.pointer().clone(),
            motion: Motion::new(start.x, start.y),
            drag,
            held: false,
            moved: false,
            position: start,
        });
    }
    fn pointer_moved(this: &Self, event: &PointerEventArgs) {
        let Some(vm) = this.vm.upgrade() else {
            this.cancel();
            return;
        };
        let point = event.get_position(None);
        let mut state = this.contact.borrow_mut();
        let Some(contact) = state
            .as_mut()
            .filter(|c| c.pointer.id() == event.pointer().id())
        else {
            return;
        };
        if !contact.drag.valid(&vm) {
            drop(state);
            this.cancel();
            return;
        }
        contact.motion.update(point.x, point.y);
        contact.position = point;
        if !contact.held {
            if !contact.motion.within_tap() {
                drop(state);
                this.cancel();
            }
            // Before the hold, scrolling keeps its ordinary native gesture arbitration.
            return;
        }
        contact.moved |= contact.motion.traveled(5.0);
        drop(state);
        event.set_handled(true);
        this.hover();
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
        this.timer.borrow_mut().take();
        highlight(&this.highlight, None);
        if !contact.held {
            contact.drag.active.set(false);
            return;
        }
        event.set_handled(true);
        let position = event.get_position(None);
        contact.motion.update(position.x, position.y);
        contact.moved |= contact.motion.traveled(5.0);
        let menu_target = if !contact.moved {
            this.vm
                .upgrade()
                .filter(|vm| contact.drag.valid(vm))
                .and_then(|_| this.button.upgrade())
                .filter(|button| button.is_attached_to_visual_tree())
        } else {
            None
        };
        if contact.moved {
            if let (Some(vm), Some(button)) = (this.vm.upgrade(), this.button.upgrade()) {
                if let Some(target) = TopLevel::get_top_level(Some(&button))
                    .and_then(|root| target_at(&root, &vm, event.get_position(None), &contact.drag))
                {
                    contact.drag.finish(&vm, &target.operation);
                }
            }
        }
        contact.drag.active.set(false);
        contact.pointer.capture(None);
        release_click(&this.node);
        // Releasing a stationary hold exposes the existing feed actions. Capture
        // and the in-process drag payload are retired before opening a popup.
        if let Some(button) = menu_target {
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
