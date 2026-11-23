//! Native mouse dragging and held-touch dragging share a validated in-process subscription payload.
mod feedback;
mod source;
mod target;
mod touch;
mod touch_input;

use crate::{session::Session, sidebar::tree::TreeNode};
use feedback::highlight;
pub(crate) use feedback::marker;
use ferroui_base::input::{DataFormat, DataFormatOf, DataTransferExtensions, DragEventArgs};
use ferroui_base::threading::{Dispatcher, DispatcherPriority};
use reader_domain::{SubscriptionNode, SubscriptionSort};
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
    static SUPPRESS_CLICK: RefCell<BTreeSet<TreeNode>> = const { RefCell::new(BTreeSet::new()) };
}
pub(crate) fn consume_click(node: &TreeNode) -> bool {
    SUPPRESS_CLICK.with(|set| set.borrow_mut().remove(node))
}
fn suppress(node: &TreeNode) {
    SUPPRESS_CLICK.with(|set| {
        set.borrow_mut().insert(node.clone());
    });
}
fn release_click(node: &TreeNode) {
    let node = node.clone();
    Dispatcher::ui_thread().post_local(
        move || {
            consume_click(&node);
        },
        DispatcherPriority::BACKGROUND,
    );
}

#[derive(Clone)]
struct DragSession {
    node: TreeNode,
    url: Option<String>,
    generation: u64,
    sort: SubscriptionSort,
    owner: Weak<AppViewModel>,
    ui: Weak<Session>,
    active: Rc<Cell<bool>>,
}
impl DragSession {
    fn new(vm: &Rc<AppViewModel>, ui: &Rc<Session>, node: &TreeNode) -> Option<Self> {
        let url = match node {
            TreeNode::Feed { id, folder } => {
                let feed = vm.feed(*id)?;
                if !in_group(&feed, folder) {
                    return None;
                }
                Some(feed.url)
            }
            TreeNode::Folder(name)
                if vm.subscription_sort() == SubscriptionSort::Manual
                    && folder_exists(vm, name) =>
            {
                None
            }
            _ => return None,
        };
        Some(Self {
            node: node.clone(),
            url,
            generation: vm.library_generation(),
            sort: vm.subscription_sort(),
            owner: Rc::downgrade(vm),
            ui: Rc::downgrade(ui),
            active: Rc::new(Cell::new(true)),
        })
    }
    fn valid(&self, vm: &Rc<AppViewModel>) -> bool {
        self.active.get()
            && self.generation == vm.library_generation()
            && self.sort == vm.subscription_sort()
            && self.owner.ptr_eq(&Rc::downgrade(vm))
            && match &self.node {
                TreeNode::Feed { id, folder } => vm.state().feed(*id).is_some_and(|feed| {
                    !feed.archived && self.url.as_ref() == Some(&feed.url) && in_group(feed, folder)
                }),
                TreeNode::Folder(name) => folder_exists(vm, name),
                TreeNode::Tag(_) => false,
            }
    }
    fn finish(&self, vm: &Rc<AppViewModel>, operation: &DropOperation) -> bool {
        if !self.valid(vm) || !operation.valid(vm, &self.node) || !self.active.replace(false) {
            return false;
        }
        let (folder, target, before) = match operation {
            DropOperation::Folder(folder) => (folder.clone(), None, false),
            DropOperation::Row {
                node,
                before,
                folder,
            } => (folder.clone(), Some(node.clone()), *before),
        };
        match &self.node {
            TreeNode::Feed { id, folder: from } => {
                let changed =
                    vm.position_feed_occurrence(*id, from.clone(), folder.clone(), target, before);
                if changed {
                    if let Some(ui) = self.ui.upgrade() {
                        ui.moved_feed_occurrence(vm, *id, from, &folder);
                    }
                }
                changed
            }
            TreeNode::Folder(name) => {
                let target = target.or_else(|| vm.subscription_tree().root.last().cloned());
                target.is_some_and(|target| {
                    vm.position_root(SubscriptionNode::Folder(name.clone()), target, before)
                })
            }
            TreeNode::Tag(_) => false,
        }
    }
}

fn in_group(feed: &reader_domain::Feed, folder: &str) -> bool {
    if folder.is_empty() {
        feed.folder.is_empty() && feed.folders.is_empty()
    } else {
        feed.in_folder(folder)
    }
}
fn folder_exists(vm: &AppViewModel, name: &str) -> bool {
    let state = vm.state();
    state.folders.iter().any(|folder| folder == name)
        || state
            .feeds
            .iter()
            .any(|feed| !feed.archived && feed.in_folder(name))
}
fn format() -> DataFormatOf<DragSession> {
    DataFormat::create_in_process_format("FerroUI.Reader.Subscription")
}
fn payload(event: &DragEventArgs, vm: &Rc<AppViewModel>) -> Option<DragSession> {
    let data = event.data_transfer().try_get_value(&format())?;
    data.valid(vm).then_some(data)
}
