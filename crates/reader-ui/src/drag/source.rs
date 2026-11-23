use super::{format, release_click, suppress, touch, DragSession};
use crate::{session::Session, sidebar::tree::TreeNode};
use ferroui_base::input::{
    DataTransfer, DataTransferItem, DragDrop, DragDropEffects, InputElement, Key,
    PointerPressedEventArgs, PointerType,
};
use ferroui_base::interactivity::RoutingStrategies;
use ferroui_base::threading::Dispatcher;
use ferroui_base::{Point, Ref};
use ferroui_controls::Button;
use ferroui_view::on_cleanup;
use reader_state::AppViewModel;
use std::{cell::RefCell, rc::Rc};

struct Press {
    start: Point,
    trigger: PointerPressedEventArgs,
    session: DragSession,
}

pub(crate) fn attach_source(
    button: &Ref<Button>,
    vm: Rc<AppViewModel>,
    ui: Rc<Session>,
    node: TreeNode,
) {
    if matches!(node, TreeNode::Feed { .. }) {
        touch::attach(button, vm.clone(), ui.clone(), node.clone());
    }
    let node = Rc::new(node);
    let pending = Rc::new(RefCell::<Option<Press>>::new(None));
    let active = Rc::new(RefCell::<Option<DragSession>>::new(None));
    let arm = pending.clone();
    let arm_vm = vm.clone();
    let arm_node = node.clone();
    let pressed = button.add_handler_with(
        InputElement::pointer_pressed_event(),
        move |_, event| {
            super::consume_click(&arm_node);
            if let Some(old) = arm.borrow_mut().take() {
                old.session.active.set(false);
            }
            if event.pointer().type_() != PointerType::Mouse
                || !event.properties().is_left_button_pressed
            {
                return;
            }
            if let Some(session) = DragSession::new(&arm_vm, &ui, &arm_node) {
                *arm.borrow_mut() = Some(Press {
                    start: event.get_position(None),
                    trigger: event.clone(),
                    session,
                });
            }
        },
        RoutingStrategies::TUNNEL,
        false,
    );
    let moving = pending.clone();
    let moving_active = active.clone();
    let moving_node = node.clone();
    let moved = button.add_handler_with(
        InputElement::pointer_moved_event(),
        move |_, event| {
            let mut state = moving.borrow_mut();
            let Some(press) = state.as_ref() else {
                return;
            };
            if press.trigger.pointer().id() != event.pointer().id()
                || !event.properties().is_left_button_pressed
                || !press.session.valid(&vm)
            {
                if let Some(old) = state.take() {
                    old.session.active.set(false);
                }
                return;
            }
            let point = event.get_position(None);
            if (point.x - press.start.x).hypot(point.y - press.start.y) < 6.0 {
                return;
            }
            let press = state.take().unwrap();
            drop(state);
            event.set_handled(true);
            suppress(&moving_node);
            *moving_active.borrow_mut() = Some(press.session.clone());
            let data = DataTransfer::new();
            data.add(DataTransferItem::create(
                &format(),
                Some(press.session.clone()),
            ));
            press.trigger.pointer().capture(None);
            let operation =
                DragDrop::do_drag_drop_async(&press.trigger, data, DragDropEffects::MOVE);
            let session = press.session;
            let node = moving_node.clone();
            Dispatcher::ui_thread().spawn_local(async move {
                let _ = operation.await;
                session.active.set(false);
                release_click(&node);
            });
        },
        RoutingStrategies::TUNNEL,
        false,
    );
    let released_state = pending.clone();
    let released = button.add_handler_with(
        InputElement::pointer_released_event(),
        move |_, event| {
            let matches = released_state
                .borrow()
                .as_ref()
                .is_some_and(|press| press.trigger.pointer().id() == event.pointer().id());
            if matches {
                if let Some(old) = released_state.borrow_mut().take() {
                    old.session.active.set(false);
                }
            }
        },
        RoutingStrategies::TUNNEL,
        true,
    );
    let lost_state = pending.clone();
    let lost = button.add_handler(InputElement::pointer_capture_lost_event(), move |_, _| {
        if let Some(old) = lost_state.borrow_mut().take() {
            old.session.active.set(false);
        }
    });
    let cancel_pending = pending.clone();
    let cancel_active = active.clone();
    let cancel_node = node.clone();
    let key = button.add_handler_with(
        InputElement::key_down_event(),
        move |_, event| {
            if event.key == Key::Escape {
                if let Some(old) = cancel_pending.borrow_mut().take() {
                    old.session.active.set(false);
                }
                if let Some(old) = cancel_active.borrow_mut().take() {
                    old.active.set(false);
                }
                release_click(&cancel_node);
            }
        },
        RoutingStrategies::TUNNEL,
        true,
    );
    let detach_pending = pending.clone();
    let detach_active = active.clone();
    let detach_node = node.clone();
    let detached = button.detached_from_visual_tree(move |_| {
        if let Some(old) = detach_pending.borrow_mut().take() {
            old.session.active.set(false);
        }
        if let Some(old) = detach_active.borrow_mut().take() {
            old.active.set(false);
        }
        release_click(&detach_node);
    });
    let weak = button.downgrade();
    on_cleanup(move || {
        detached.dispose();
        if let Some(old) = pending.borrow_mut().take() {
            old.session.active.set(false);
        }
        if let Some(old) = active.borrow_mut().take() {
            old.active.set(false);
        }
        if let Some(button) = weak.upgrade() {
            button.remove_handler(InputElement::pointer_pressed_event(), pressed);
            button.remove_handler(InputElement::pointer_moved_event(), moved);
            button.remove_handler(InputElement::pointer_released_event(), released);
            button.remove_handler(InputElement::pointer_capture_lost_event(), lost);
            button.remove_handler(InputElement::key_down_event(), key);
        }
    });
}
