use super::{
    highlight, release_click, target_at,
    touch::{Contact, TouchDrag},
    DragSession,
};
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
        let Some(drag) = DragSession::new(&vm, this.id) else {
            return;
        };
        *this.contact.borrow_mut() = Some(Contact {
            pointer: event.pointer().clone(),
            start: event.get_position(None),
            drag,
            held: false,
            moved: false,
        });
    }
    fn pointer_moved(this: &Self, event: &PointerEventArgs) {
        let Some(vm) = this.vm.upgrade() else {
            this.cancel();
            return;
        };
        let Some(button) = this.button.upgrade() else {
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
        let distance = (point.x - contact.start.x).hypot(point.y - contact.start.y);
        if !contact.held {
            if distance >= 8.0 {
                drop(state);
                this.cancel();
            }
            // Before the hold, scrolling keeps its ordinary native gesture arbitration.
            return;
        }
        contact.moved |= distance >= 5.0;
        let moved = contact.moved;
        drop(state);
        event.set_handled(true);
        let target =
            TopLevel::get_top_level(Some(&button)).and_then(|root| target_at(&root, &vm, point));
        highlight(
            &this.highlight,
            if moved {
                target.as_ref().map(|(control, _)| control)
            } else {
                None
            },
        );
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
        let contact = this.contact.borrow_mut().take().unwrap();
        highlight(&this.highlight, None);
        if !contact.held {
            contact.drag.active.set(false);
            return;
        }
        event.set_handled(true);
        if contact.moved {
            if let (Some(vm), Some(button)) = (this.vm.upgrade(), this.button.upgrade()) {
                if let Some((_, folder)) = TopLevel::get_top_level(Some(&button))
                    .and_then(|root| target_at(&root, &vm, event.get_position(None)))
                {
                    contact.drag.finish(&vm, &folder);
                }
            }
        }
        contact.drag.active.set(false);
        contact.pointer.capture(None);
        release_click(this.id);
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
