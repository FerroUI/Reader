//! Native mouse dragging and held-touch dragging share a validated in-process subscription payload.
mod source;
mod touch;
mod touch_input;

use ferroui_base::input::{
    DataFormat, DataFormatOf, DataTransferExtensions, DragDrop, DragDropEffects, DragEventArgs,
};
use ferroui_base::threading::{Dispatcher, DispatcherPriority};
use ferroui_base::{Point, Ref, WeakRef};
use ferroui_controls::{primitives::TemplatedControl, Control, Panel, TextBlock, TopLevel};
use ferroui_view::{on_cleanup, style};
use reader_domain::Id;
use reader_state::AppViewModel;
pub(crate) use source::attach_source;
use std::{
    cell::{Cell, RefCell},
    collections::BTreeSet,
    rc::{Rc, Weak},
};

ferroui_base::ferro_thread_local! {
    static SUPPRESS_CLICK: RefCell<BTreeSet<Id>> = const { RefCell::new(BTreeSet::new()) };
}
pub(crate) fn consume_click(id: Id) -> bool {
    SUPPRESS_CLICK.with(|set| set.borrow_mut().remove(&id))
}
fn suppress(id: Id) {
    SUPPRESS_CLICK.with(|set| {
        set.borrow_mut().insert(id);
    });
}
fn release_click(id: Id) {
    Dispatcher::ui_thread().post_local(
        move || {
            consume_click(id);
        },
        DispatcherPriority::BACKGROUND,
    );
}

#[derive(Clone)]
struct DragSession {
    id: Id,
    url: String,
    owner: Weak<AppViewModel>,
    active: Rc<Cell<bool>>,
}
impl DragSession {
    fn new(vm: &Rc<AppViewModel>, id: Id) -> Option<Self> {
        Some(Self {
            id,
            url: vm.feed(id)?.url,
            owner: Rc::downgrade(vm),
            active: Rc::new(Cell::new(true)),
        })
    }
    fn valid(&self, vm: &Rc<AppViewModel>) -> bool {
        self.active.get()
            && self.owner.ptr_eq(&Rc::downgrade(vm))
            && vm.feed(self.id).is_some_and(|f| f.url == self.url)
    }
    fn finish(&self, vm: &Rc<AppViewModel>, folder: &str) -> bool {
        if !self.valid(vm) || (!folder.is_empty() && !vm.folders().iter().any(|f| f == folder)) {
            return false;
        }
        if !self.active.replace(false) {
            return false;
        }
        vm.move_feed(self.id, folder.to_string());
        true
    }
}
fn format() -> DataFormatOf<DragSession> {
    DataFormat::create_in_process_format("FerroUI.Reader.Subscription")
}
fn payload(event: &DragEventArgs, vm: &Rc<AppViewModel>) -> Option<DragSession> {
    let data = event.data_transfer().try_get_value(&format())?;
    data.valid(vm).then_some(data)
}

struct FolderTarget {
    folder: String,
    owner: Weak<AppViewModel>,
}
impl PartialEq for FolderTarget {
    fn eq(&self, other: &Self) -> bool {
        self.folder == other.folder && self.owner.ptr_eq(&other.owner)
    }
}
fn target_at(
    root: &TopLevel,
    vm: &Rc<AppViewModel>,
    point: Point,
) -> Option<(Ref<Control>, String)> {
    let hit = root.input_hit_test_with(point, false)?;
    for control in hit
        .get_self_and_visual_ancestors()
        .filter_map(|v| v.cast::<Control>())
    {
        let tag = control.tag();
        let Some(target) = tag
            .as_ref()
            .and_then(|tag| tag.downcast_ref::<FolderTarget>())
        else {
            continue;
        };
        if target.owner.ptr_eq(&Rc::downgrade(vm)) {
            return Some((control, target.folder.clone()));
        }
    }
    None
}
fn highlight(previous: &RefCell<Option<WeakRef<Control>>>, target: Option<&Ref<Control>>) {
    if let Some(old) = previous.borrow_mut().take().and_then(|weak| weak.upgrade()) {
        old.classes().set("reader-drop-target", false);
    }
    if let Some(target) = target {
        target.classes().set("reader-drop-target", true);
        *previous.borrow_mut() = Some(target.downgrade());
    }
}

pub(crate) fn attach_target(control: &Ref<Control>, vm: Rc<AppViewModel>, folder: String) {
    let old_tag = control.tag();
    control.set_tag(Some(Rc::new(FolderTarget {
        folder: folder.clone(),
        owner: Rc::downgrade(&vm),
    })));
    control.styles().add(
        style("Button.reader-drop-target")
            .set(TemplatedControl::background_property(), "#D9E7FB")
            .set(TemplatedControl::border_brush_property(), "#4D90FE")
            .build(),
    );
    control.styles().add(
        style("Grid.reader-drop-target")
            .set(Panel::background_property(), "#D9E7FB")
            .build(),
    );
    control.styles().add(
        style("TextBlock.reader-drop-target")
            .set(TextBlock::background_property(), "#D9E7FB")
            .build(),
    );
    DragDrop::set_allow_drop(control, true);
    let weak = control.downgrade();
    let hover_vm = vm.clone();
    let hover_folder = folder.clone();
    let hover = move |event: &DragEventArgs| {
        let Some(control) = weak.upgrade() else {
            return;
        };
        let valid = payload(event, &hover_vm).is_some()
            && (hover_folder.is_empty() || hover_vm.folders().contains(&hover_folder));
        control.classes().set("reader-drop-target", valid);
        event.set_drag_effects(if valid {
            DragDropEffects::MOVE
        } else {
            DragDropEffects::NONE
        });
        if valid {
            event.set_handled(true);
        }
    };
    let enter_hover = hover.clone();
    let enter = DragDrop::add_drag_enter_handler(control, move |_, event| enter_hover(event));
    let over = DragDrop::add_drag_over_handler(control, move |_, event| hover(event));
    let weak = control.downgrade();
    let leave = DragDrop::add_drag_leave_handler(control, move |_, _| {
        if let Some(control) = weak.upgrade() {
            control.classes().set("reader-drop-target", false);
        }
    });
    let weak = control.downgrade();
    let drop = DragDrop::add_drop_handler(control, move |_, event| {
        if let Some(control) = weak.upgrade() {
            control.classes().set("reader-drop-target", false);
        }
        let Some(session) = payload(event, &vm) else {
            return;
        };
        event.set_handled(true);
        event.set_drag_effects(if session.finish(&vm, &folder) {
            DragDropEffects::MOVE
        } else {
            DragDropEffects::NONE
        });
    });
    let weak = control.downgrade();
    on_cleanup(move || {
        if let Some(control) = weak.upgrade() {
            control.classes().set("reader-drop-target", false);
            control.set_tag(old_tag);
            DragDrop::remove_drag_enter_handler(&control, enter);
            DragDrop::remove_drag_over_handler(&control, over);
            DragDrop::remove_drag_leave_handler(&control, leave);
            DragDrop::remove_drop_handler(&control, drop);
        }
    });
}
