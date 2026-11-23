//! A live row target is either a folder move or a before/after insertion.
use super::{feedback, payload, DragSession};
use crate::sidebar::tree::TreeNode;
use ferroui_base::input::{DragDrop, DragDropEffects, DragEventArgs};
use ferroui_base::{Point, Ref};
use ferroui_controls::primitives::TemplatedControl;
use ferroui_controls::{Control, Panel, TextBlock, TopLevel};
use ferroui_view::{on_cleanup, style};
use reader_domain::{Id, SubscriptionNode, SubscriptionSort};
use reader_state::AppViewModel;
use std::rc::{Rc, Weak};

#[derive(Clone, PartialEq)]
enum Target {
    Folder(String),
    Row { id: Id, folder: String },
}
struct TargetTag {
    target: Target,
    owner: Weak<AppViewModel>,
}
impl PartialEq for TargetTag {
    fn eq(&self, other: &Self) -> bool {
        self.target == other.target && self.owner.ptr_eq(&other.owner)
    }
}

#[derive(Clone)]
pub(super) enum DropOperation {
    Folder(String),
    Row {
        node: SubscriptionNode,
        before: bool,
        folder: String,
    },
}
impl DropOperation {
    pub(super) fn valid(&self, vm: &AppViewModel, source: &TreeNode) -> bool {
        let from = match source {
            TreeNode::Feed { folder, .. } => folder.as_str(),
            TreeNode::Folder(_) => "",
            TreeNode::Tag(_) => return false,
        };
        match self {
            Self::Folder(folder) => {
                (folder.is_empty() || super::folder_exists(vm, folder))
                    && match source {
                        TreeNode::Folder(_) => {
                            folder.is_empty() && vm.subscription_sort() == SubscriptionSort::Manual
                        }
                        TreeNode::Feed { .. } => from != folder,
                        TreeNode::Tag(_) => false,
                    }
            }
            Self::Row { node, folder, .. } => {
                if from == folder && vm.subscription_sort() == SubscriptionSort::Alphabetical {
                    return false;
                }
                if matches!(source, TreeNode::Folder(_)) && !folder.is_empty() {
                    return false;
                }
                match node {
                    SubscriptionNode::Folder(name) => {
                        folder.is_empty()
                            && source != &TreeNode::Folder(name.clone())
                            && super::folder_exists(vm, name)
                    }
                    SubscriptionNode::Feed(id) => {
                        source
                            != &TreeNode::Feed {
                                id: *id,
                                folder: folder.clone(),
                            }
                            && vm
                                .state()
                                .feed(*id)
                                .is_some_and(|feed| !feed.archived && super::in_group(feed, folder))
                    }
                }
            }
        }
    }
}
pub(super) struct PlannedTarget {
    pub control: Ref<Control>,
    pub operation: DropOperation,
}

fn plan(
    control: &Control,
    vm: &Rc<AppViewModel>,
    source: &TreeNode,
    point: Point,
) -> Option<DropOperation> {
    let bounds = control.bounds();
    if !point.x.is_finite()
        || !point.y.is_finite()
        || point.x < 0.0
        || point.y < 0.0
        || point.x > bounds.width
        || point.y > bounds.height
    {
        return None;
    }
    let tag = control.tag();
    let tag = tag.as_ref()?.downcast_ref::<TargetTag>()?;
    if !tag.owner.ptr_eq(&Rc::downgrade(vm)) {
        return None;
    }
    let manual = vm.subscription_sort() == SubscriptionSort::Manual;
    let operation = match &tag.target {
        Target::Folder(folder) if folder.is_empty() => DropOperation::Folder(String::new()),
        Target::Folder(folder)
            if matches!(source, TreeNode::Folder(_))
                || (manual
                    && (point.y < bounds.height * 0.25 || point.y > bounds.height * 0.75)) =>
        {
            DropOperation::Row {
                node: SubscriptionNode::Folder(folder.clone()),
                before: point.y < bounds.height / 2.0,
                folder: String::new(),
            }
        }
        Target::Folder(folder) => DropOperation::Folder(folder.clone()),
        Target::Row { folder, .. } if !manual => DropOperation::Folder(folder.clone()),
        Target::Row { id, folder } => DropOperation::Row {
            node: SubscriptionNode::Feed(*id),
            before: point.y < bounds.height / 2.0,
            folder: folder.clone(),
        },
    };
    operation.valid(vm, source).then_some(operation)
}

pub(super) fn target_at(
    root: &TopLevel,
    vm: &Rc<AppViewModel>,
    point: Point,
    drag: &DragSession,
) -> Option<PlannedTarget> {
    if !drag.valid(vm) {
        return None;
    }
    let hit = root.input_hit_test_with(point, false)?;
    for control in hit
        .get_self_and_visual_ancestors()
        .filter_map(|visual| visual.cast::<Control>())
    {
        if let Some(local) = root.translate_point(point, &control) {
            if let Some(operation) = plan(&control, vm, &drag.node, local) {
                return Some(PlannedTarget { control, operation });
            }
        }
    }
    None
}

pub(crate) fn attach_row_target(
    control: &Ref<Control>,
    vm: Rc<AppViewModel>,
    id: Id,
    folder: String,
) {
    styles(control);
    attach(control, vm, Target::Row { id, folder });
}
pub(crate) fn attach_target(control: &Ref<Control>, vm: Rc<AppViewModel>, folder: String) {
    styles(control);
    attach(control, vm, Target::Folder(folder));
}

fn styles(control: &Ref<Control>) {
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
}

fn attach(control: &Ref<Control>, vm: Rc<AppViewModel>, target: Target) {
    let old_tag = control.tag();
    control.set_tag(Some(Rc::new(TargetTag {
        target,
        owner: Rc::downgrade(&vm),
    })));
    DragDrop::set_allow_drop(control, true);
    let weak = control.downgrade();
    let hover_vm = vm.clone();
    let hover = move |event: &DragEventArgs| {
        let Some(control) = weak.upgrade() else {
            return;
        };
        let planned = payload(event, &hover_vm).and_then(|drag| {
            plan(
                &control,
                &hover_vm,
                &drag.node,
                event.get_position(&control),
            )
        });
        feedback::show(&control, planned.as_ref());
        event.set_drag_effects(if planned.is_some() {
            DragDropEffects::MOVE
        } else {
            DragDropEffects::NONE
        });
        if planned.is_some() {
            event.set_handled(true);
            feedback::scroll(&control, event);
        }
    };
    let enter_hover = hover.clone();
    let enter = DragDrop::add_drag_enter_handler(control, move |_, event| enter_hover(event));
    let over = DragDrop::add_drag_over_handler(control, move |_, event| hover(event));
    let weak = control.downgrade();
    let leave = DragDrop::add_drag_leave_handler(control, move |_, _| {
        if let Some(control) = weak.upgrade() {
            feedback::show(&control, None);
        }
    });
    let weak = control.downgrade();
    let drop = DragDrop::add_drop_handler(control, move |_, event| {
        let Some(control) = weak.upgrade() else {
            return;
        };
        feedback::show(&control, None);
        let Some(drag) = payload(event, &vm) else {
            return;
        };
        event.set_handled(true);
        let accepted = plan(&control, &vm, &drag.node, event.get_position(&control))
            .is_some_and(|operation| drag.finish(&vm, &operation));
        event.set_drag_effects(if accepted {
            DragDropEffects::MOVE
        } else {
            DragDropEffects::NONE
        });
    });
    let weak = control.downgrade();
    on_cleanup(move || {
        if let Some(control) = weak.upgrade() {
            feedback::show(&control, None);
            control.set_tag(old_tag);
            DragDrop::remove_drag_enter_handler(&control, enter);
            DragDrop::remove_drag_over_handler(&control, over);
            DragDrop::remove_drag_leave_handler(&control, leave);
            DragDrop::remove_drop_handler(&control, drop);
        }
    });
}
