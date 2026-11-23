use super::ArticlePanel;
use crate::reading::page_policy::Current;
use ferroui_base::{Point, Ref};
use ferroui_controls::{Control, ListBox};
use reader_domain::Id;

/// Resolve the current item through the persistent identity/height index. A
/// realized entry supplies its exact bounds; an offscreen cursor uses cached
/// geometry without realizing or scanning the rest of a large stream.
pub(in crate::reading) fn position(
    list: &Ref<ListBox>,
    id: Option<Id>,
) -> Option<(usize, Option<Current>)> {
    let panel = list.items_panel_root()?.cast::<ArticlePanel>()?;
    if !panel.is_attached_to_visual_tree() || panel.presenter.is_in_layout() {
        return None;
    }
    panel.update_layout();
    panel.synchronize_metrics(panel.bounds().width);
    let metrics = panel.metrics.borrow();
    let count = metrics.extents.len();
    let mut current = id.and_then(|id| {
        let index = metrics.index(id)?;
        let top = metrics.extents.start(index);
        Some(Current {
            index,
            top,
            bottom: top + metrics.extents.height(index),
        })
    });
    drop(metrics);
    if let Some(current) = current.as_mut() {
        if let Some(container) = panel.presenter.try_get_element(current.index as i32) {
            let entry = container
                .get_visual_descendants()
                .take(96)
                .filter_map(|visual| visual.cast::<Control>())
                .find(|control| {
                    (control.has_name("ArticleEntry") || control.has_name("ArticleRow"))
                        && control
                            .tag()
                            .is_some_and(|tag| tag.downcast_ref::<Id>().copied() == id)
                });
            if let Some(entry) = entry {
                if let Some(point) = entry.translate_point(Point::new(0.0, 0.0), &panel) {
                    current.top = point.y;
                    current.bottom = point.y + entry.bounds().height;
                }
            }
        }
    }
    Some((count, current))
}
