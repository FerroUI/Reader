use super::{Frame, Reading, CURRENT};
use crate::session::Session;
use ferroui_base::input::{InputElement, Key, KeyModifiers};
use ferroui_base::interactivity::RoutingStrategies;
use ferroui_base::{FerroObject, Ref};
use ferroui_controls::{primitives::ScrollBar, Control, ListBox, ScrollViewer};
use ferroui_view::on_cleanup;
use reader_state::AppViewModel;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

fn scrollbar(source: Option<Ref<FerroObject>>) -> bool {
    source
        .and_then(|source| source.cast::<Control>())
        .is_some_and(|control| {
            control
                .get_self_and_visual_ancestors()
                .any(|ancestor| ancestor.cast::<ScrollBar>().is_some())
        })
}
pub fn attach(list: &Ref<ListBox>, vm: Rc<AppViewModel>, ui: Rc<Session>) {
    super::panel::install(list, &vm, &ui);
    let reading = Rc::new(Reading {
        list: list.downgrade(),
        vm: Rc::downgrade(&vm),
        ui: Rc::downgrade(&ui),
        queued: Cell::new(false),
        frame: Cell::new(Frame::default()),
        intent: RefCell::new(None),
    });
    CURRENT.with(|current| *current.borrow_mut() = Rc::downgrade(&reading));
    let state = reading.clone();
    let changed = list.add_handler(ScrollViewer::scroll_changed_event(), move |_, event| {
        if event.offset_delta().y.abs() < 0.5 {
            return;
        }
        let Some(list) = state.list.upgrade() else {
            return;
        };
        let source = event
            .source()
            .and_then(|source| source.cast::<ScrollViewer>());
        let own = list
            .scroll()
            .and_then(|scroll| scroll.cast::<ScrollViewer>());
        if source.is_some() && source == own {
            state.changed();
        }
    });
    let state = reading.clone();
    let wheel = list.add_handler_with(
        InputElement::pointer_wheel_changed_event(),
        move |_, event| {
            if event.delta().y != 0.0 {
                state.input();
            }
        },
        RoutingStrategies::TUNNEL,
        true,
    );
    let state = reading.clone();
    let gesture = list.add_handler_with(
        InputElement::scroll_gesture_event(),
        move |_, event| {
            if event.delta().y != 0.0 {
                state.input();
            }
        },
        RoutingStrategies::BUBBLE,
        true,
    );
    let state = reading.clone();
    let press = list.add_handler_with(
        InputElement::pointer_pressed_event(),
        move |_, event| {
            if scrollbar(event.source()) {
                state.input();
            }
        },
        RoutingStrategies::TUNNEL,
        true,
    );
    let state = reading.clone();
    let moved = list.add_handler_with(
        InputElement::pointer_moved_event(),
        move |_, event| {
            if event.properties().is_left_button_pressed && scrollbar(event.source()) {
                state.input();
            }
        },
        RoutingStrategies::TUNNEL,
        true,
    );
    let state = reading.clone();
    let key = list.add_handler_with(
        InputElement::key_down_event(),
        move |_, event| {
            if !event
                .key_modifiers
                .intersects(KeyModifiers::ALT | KeyModifiers::CONTROL | KeyModifiers::META)
                && matches!(
                    event.key,
                    Key::Space
                        | Key::PageUp
                        | Key::PageDown
                        | Key::Home
                        | Key::End
                        | Key::Up
                        | Key::Down
                )
                && !crate::keyboard::editing_source(event)
            {
                state.input();
            }
        },
        RoutingStrategies::TUNNEL,
        true,
    );
    let weak = list.downgrade();
    on_cleanup(move || {
        reading.frame.set(Frame::default());
        if let Some(list) = weak.upgrade() {
            list.remove_handler(ScrollViewer::scroll_changed_event(), changed);
            list.remove_handler(InputElement::pointer_wheel_changed_event(), wheel);
            list.remove_handler(InputElement::scroll_gesture_event(), gesture);
            list.remove_handler(InputElement::pointer_pressed_event(), press);
            list.remove_handler(InputElement::pointer_moved_event(), moved);
            list.remove_handler(InputElement::key_down_event(), key);
        }
    });
}
