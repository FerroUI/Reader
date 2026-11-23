//! Auto-read follows actual input and an observed offset change in the same frame.
//! Layout, image reflow, route restoration and keyboard scanning alone never read.
mod extent;
mod geometry;
mod input;
mod metrics;
mod page;
mod page_policy;
mod panel;
mod viewport;

use crate::session::{Panel, Session};
use ferroui_base::threading::{Dispatcher, DispatcherPriority};
use ferroui_base::{Point, Ref, WeakRef};
use ferroui_controls::{Control, ListBox};
pub use input::attach;
pub(crate) use page::page;
use reader_domain::{View, ViewMode};
use reader_state::AppViewModel;
use std::{
    cell::{Cell, RefCell},
    rc::{Rc, Weak},
    time::Instant,
};

struct InputIntent {
    at: Instant,
    route: View,
    viewport: (f64, f64),
}
struct Reading {
    list: WeakRef<ListBox>,
    vm: Weak<AppViewModel>,
    ui: Weak<Session>,
    queued: Cell<bool>,
    frame: Cell<Frame>,
    intent: RefCell<Option<InputIntent>>,
}
#[derive(Clone, Copy, Default)]
struct Frame {
    input: bool,
    offset_changed: bool,
}
ferroui_base::ferro_thread_local! {
    static CURRENT: RefCell<Weak<Reading>> = const { RefCell::new(Weak::new()) };
}

/// Space can page the list while keyboard focus is on the application root.
pub(crate) fn user_scroll() {
    let reading = CURRENT.with(|current| current.borrow().upgrade());
    if let Some(reading) = reading {
        reading.input();
    }
}
impl Reading {
    fn input(self: &Rc<Self>) {
        if let Some(list) = self.list.upgrade() {
            panel::user_input(&list);
        }
        let (Some(vm), Some(ui)) = (self.vm.upgrade(), self.ui.upgrade()) else {
            return;
        };
        if blocked(&vm, &ui) {
            return;
        }
        *self.intent.borrow_mut() = Some(InputIntent {
            at: Instant::now(),
            route: vm.view(),
            viewport: (vm.viewport(), vm.viewport_height()),
        });
        let mut frame = self.frame.get();
        frame.input = true;
        self.frame.set(frame);
        self.queue();
    }
    fn changed(self: &Rc<Self>) {
        let mut frame = self.frame.get();
        frame.offset_changed = true;
        self.frame.set(frame);
        self.queue();
    }
    fn queue(self: &Rc<Self>) {
        if self.queued.replace(true) {
            return;
        }
        let weak = Rc::downgrade(self);
        Dispatcher::ui_thread().post_local(
            move || {
                if let Some(reading) = weak.upgrade() {
                    reading.flush();
                }
            },
            DispatcherPriority::BACKGROUND,
        );
    }
    fn flush(&self) {
        self.queued.set(false);
        let frame = self.frame.take();
        let intent = self.intent.borrow_mut().take();
        if !frame.input || !frame.offset_changed {
            return;
        }
        let (Some(intent), Some(list), Some(vm), Some(ui)) = (
            intent,
            self.list.upgrade(),
            self.vm.upgrade(),
            self.ui.upgrade(),
        ) else {
            return;
        };
        if !list.is_attached_to_visual_tree()
            || blocked(&vm, &ui)
            || vm.view() != intent.route
            || (vm.viewport(), vm.viewport_height()) != intent.viewport
            || ui
                .programmatic_scroll
                .get()
                .is_some_and(|at| at >= intent.at)
        {
            return;
        }
        let settings = vm.state().settings.clone();
        if !settings.mark_read_on_scroll || settings.view_mode != ViewMode::Expanded {
            return;
        }
        if let Some(id) = reading_line(&list) {
            vm.select_from_scroll(id);
        }
    }
}
fn blocked(vm: &AppViewModel, ui: &Session) -> bool {
    ui.panel.get_untracked() != Panel::None
        || crate::transient::has_menu()
        || (vm.viewport() < 700.0 && vm.sidebar_open())
}
fn reading_line(list: &Ref<ListBox>) -> Option<reader_domain::Id> {
    for visual in list.get_visual_descendants() {
        let Some(control) = visual.cast::<Control>() else {
            continue;
        };
        let Some(id) = control
            .has_name("ArticleEntry")
            .then(|| control.tag())
            .flatten()
            .and_then(|value| value.downcast_ref::<u64>().copied())
        else {
            continue;
        };
        let Some(position) = control.translate_point(Point::new(0.0, 0.0), list) else {
            continue;
        };
        if position.y <= 40.0 && position.y + control.bounds().height > 40.0 {
            return Some(id);
        }
    }
    None
}
