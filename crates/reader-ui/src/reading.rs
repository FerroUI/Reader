//! Marks the entry crossing the reading line after a user scroll, without doing
//! document mutations inside layout or collection notifications.
use crate::session::Session;
use ferroui_base::threading::{Dispatcher, DispatcherPriority};
use ferroui_base::{Point, Ref};
use ferroui_controls::{Control, ListBox, ScrollViewer};
use ferroui_view::on_cleanup;
use reader_domain::ViewMode;
use reader_state::AppViewModel;
use std::{cell::Cell, rc::Rc};

pub fn attach(list: &Ref<ListBox>, vm: Rc<AppViewModel>, ui: Rc<Session>) {
    let queued = Rc::new(Cell::new(false));
    let weak_list = list.downgrade();
    let weak_vm = Rc::downgrade(&vm);
    let weak_ui = Rc::downgrade(&ui);
    let token = list.add_handler(ScrollViewer::scroll_changed_event(), move |_, event| {
        if weak_ui.upgrade().is_some_and(|ui| {
            ui.programmatic_scroll
                .get()
                .is_some_and(|at| at.elapsed().as_millis() < 250)
        }) {
            return;
        }
        if event.offset_delta().y.abs() < 0.5 || queued.replace(true) {
            return;
        }
        let list = weak_list.clone();
        let vm = weak_vm.clone();
        let ui = weak_ui.clone();
        let queued = queued.clone();
        Dispatcher::ui_thread().post_local(
            move || {
                queued.set(false);
                let (Some(list), Some(vm), Some(ui)) = (list.upgrade(), vm.upgrade(), ui.upgrade())
                else {
                    return;
                };
                if ui
                    .programmatic_scroll
                    .get()
                    .is_some_and(|at| at.elapsed().as_millis() < 250)
                {
                    return;
                }
                if !list.is_attached_to_visual_tree() {
                    return;
                }
                let settings = vm.state().settings.clone();
                if !settings.mark_read_on_scroll || settings.view_mode != ViewMode::Expanded {
                    return;
                }
                let mut candidate = None;
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
                    let Some(position) = control.translate_point(Point::new(0.0, 0.0), &list)
                    else {
                        continue;
                    };
                    let bottom = position.y + control.bounds().height;
                    if position.y <= 40.0 && bottom > 40.0 {
                        candidate = Some(id);
                        break;
                    }
                }
                if let Some(id) = candidate {
                    // Explicit Keep unread is sticky until a different entry is opened.
                    vm.select_from_scroll(id);
                }
            },
            DispatcherPriority::BACKGROUND,
        );
    });
    let weak = list.downgrade();
    on_cleanup(move || {
        if let Some(list) = weak.upgrade() {
            list.remove_handler(ScrollViewer::scroll_changed_event(), token);
        }
    });
}
