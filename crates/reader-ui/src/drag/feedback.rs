use super::target::{DropOperation, PlannedTarget};
use ferroui_base::input::DragEventArgs;
use ferroui_base::layout::VerticalAlignment as V;
use ferroui_base::{Point, Vector, WeakRef};
use ferroui_controls::{Border, Control, ScrollViewer, TopLevel};
use ferroui_view::prelude::*;
use std::cell::RefCell;

const BEFORE: &str = "SubscriptionDropBefore";
const AFTER: &str = "SubscriptionDropAfter";

pub(crate) fn marker(after: bool) -> Element {
    border()
        .name(if after { AFTER } else { BEFORE })
        .height(2.0)
        .vertical_alignment(if after { V::Bottom } else { V::Top })
        .background("#4D90FE")
        .is_hit_test_visible(false)
        .is_visible(false)
        .into()
}
pub(super) fn show(control: &Control, operation: Option<&DropOperation>) {
    control.classes().set(
        "reader-drop-target",
        matches!(operation, Some(DropOperation::Folder(_))),
    );
    for (name, visible) in [
        (
            BEFORE,
            matches!(operation, Some(DropOperation::Row { before: true, .. })),
        ),
        (
            AFTER,
            matches!(operation, Some(DropOperation::Row { before: false, .. })),
        ),
    ] {
        if let Some(marker) = control.find_descendant_of_type_where::<Border>(false, |child| {
            child.name().as_deref() == Some(name)
        }) {
            marker.set_is_visible(visible);
        }
    }
}
pub(super) fn highlight(
    previous: &RefCell<Option<WeakRef<Control>>>,
    target: Option<&PlannedTarget>,
) {
    if let Some(old) = previous.borrow_mut().take().and_then(|weak| weak.upgrade()) {
        show(&old, None);
    }
    if let Some(target) = target {
        show(&target.control, Some(&target.operation));
        *previous.borrow_mut() = Some(target.control.downgrade());
    }
}
pub(super) fn scroll(control: &Control, event: &DragEventArgs) {
    if let Some(scroll) = control.find_ancestor_of_type::<ScrollViewer>(false) {
        scroll_at(&scroll, event.get_position(&scroll));
    }
}
pub(super) fn touch_scroll(control: &Control, root: &TopLevel, point: Point) {
    if let Some(scroll) = control.find_ancestor_of_type::<ScrollViewer>(false) {
        if let Some(local) = root.translate_point(point, &scroll) {
            scroll_at(&scroll, local);
        }
    }
}
fn scroll_at(scroll: &ScrollViewer, point: Point) {
    let bounds = scroll.bounds();
    if point.x < 0.0 || point.x > bounds.width || point.y < 0.0 || point.y > bounds.height {
        return;
    }
    let delta = if point.y < 28.0 {
        -14.0
    } else if point.y > bounds.height - 28.0 {
        14.0
    } else {
        return;
    };
    let offset = scroll.offset();
    let maximum = (scroll.extent().height - scroll.viewport().height).max(0.0);
    let y = (offset.y + delta).clamp(0.0, maximum);
    if y != offset.y {
        scroll.set_offset(Vector::new(offset.x, y));
    }
}
