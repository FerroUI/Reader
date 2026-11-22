//! Native mouse dragging and held-touch dragging share a validated in-process subscription payload.
mod feedback;
mod source;
mod target;
mod touch;
mod touch_input;

use feedback::highlight;
pub(crate) use feedback::marker;
use ferroui_base::input::{DataFormat, DataFormatOf, DataTransferExtensions, DragEventArgs};
use ferroui_base::threading::{Dispatcher, DispatcherPriority};
use reader_domain::Id;
use reader_state::AppViewModel;
pub(crate) use source::attach_source;
use std::{
    cell::{Cell, RefCell},
    collections::BTreeSet,
    rc::{Rc, Weak},
};
pub(crate) use target::{attach_row_target, attach_target};
use target::{target_at, DropOperation};

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
    generation: u64,
    owner: Weak<AppViewModel>,
    active: Rc<Cell<bool>>,
}
impl DragSession {
    fn new(vm: &Rc<AppViewModel>, id: Id) -> Option<Self> {
        Some(Self {
            id,
            url: vm.feed(id)?.url,
            generation: vm.library_generation(),
            owner: Rc::downgrade(vm),
            active: Rc::new(Cell::new(true)),
        })
    }
    fn valid(&self, vm: &Rc<AppViewModel>) -> bool {
        self.active.get()
            && self.generation == vm.library_generation()
            && self.owner.ptr_eq(&Rc::downgrade(vm))
            && vm.feed(self.id).is_some_and(|f| f.url == self.url)
    }
    fn finish(&self, vm: &Rc<AppViewModel>, operation: &DropOperation) -> bool {
        if !self.valid(vm) || !operation.valid(vm, self.id) || !self.active.replace(false) {
            return false;
        }
        match operation {
            DropOperation::Folder(folder) => vm.move_feed(self.id, folder.clone()),
            DropOperation::Row { id, before, folder } => {
                vm.position_feed(self.id, *id, *before, Some(folder.clone()))
            }
        }
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
