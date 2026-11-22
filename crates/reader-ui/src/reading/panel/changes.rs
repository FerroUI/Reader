use super::{Anchor, ArticlePanel};
use crate::rows::Row;
use ferroui_base::collections::NotifyCollectionChangedAction as Action;
use ferroui_base::input::{InputElement, NavigationDirection};
use ferroui_base::threading::{Dispatcher, DispatcherPriority};
use ferroui_base::{Point, Ref, Vector};
use ferroui_controls::{
    ContentControl, Control, ItemsChangedEventArgs, ItemsControl, ItemsSourceView,
    VirtualizingPanelImpl,
};
use ferroui_controls_treedatagrid::primitives::PresenterItemsChange;
use std::{rc::Rc, time::Instant};

impl VirtualizingPanelImpl for ArticlePanel {
    fn on_items_control_changed(this: &Self, _: Option<&Ref<ItemsControl>>) {
        this.cancel_restore();
        this.reset_containers();
        *this.generator.borrow_mut() = this.item_container_generator();
        this.metrics.borrow_mut().dirty = true;
    }

    fn on_items_changed(this: &Self, _: &Rc<ItemsSourceView>, event: &ItemsChangedEventArgs<'_>) {
        // An index belongs to the source snapshot that requested it.
        this.requested.set(None);
        // Read/star/cursor updates retain the same container, its native scroll
        // anchor and its component instance. Only its projected item changes.
        if event.action == Action::Replace
            && event.new_items.len() == event.old_items.len()
            && event.new_starting_index == event.old_starting_index
            && (0..event.new_items.len()).all(|index| {
                let old = event.old_items.get(index);
                let new = event.new_items.get(index);
                old.as_ref()
                    .and_then(|value| value.downcast_ref::<Row>())
                    .zip(new.as_ref().and_then(|value| value.downcast_ref::<Row>()))
                    .is_some_and(|(old, new)| old.article.id == new.article.id)
            })
        {
            // Capture before a template switches list/expanded or feed/full
            // bodies. A surviving block gives a precise anchor; replaced blocks
            // fall back to the article identity instead of a clamped old offset.
            this.capture_anchor();
            for offset in 0..event.new_items.len() {
                let index = event.new_starting_index + offset as i32;
                let item = event.new_items.get(offset);
                let row = item
                    .as_ref()
                    .and_then(|value| value.downcast_ref::<Row>())
                    .unwrap()
                    .clone();
                this.metrics.borrow_mut().replace(index as usize, row);
                let container = this
                    .containers
                    .borrow()
                    .iter()
                    .find(|entry| entry.index == index)
                    .map(|entry| entry.control.clone());
                if let Some(container) = container {
                    // The generator only initializes unset Content properties.
                    // A retained ListBoxItem must receive the new projection
                    // explicitly so its recycling template updates in place.
                    container.set_current_value(ContentControl::content_property(), item.clone());
                    this.prepare(&container, &item, index);
                }
            }
            this.invalidate_measure();
            return;
        }
        this.capture_anchor();
        this.metrics.borrow_mut().dirty = true;
        this.presenter.on_items_collection_changed(
            this,
            &PresenterItemsChange {
                action: event.action,
                new_starting_index: event.new_starting_index,
                new_count: event.new_items.len() as i32,
                old_starting_index: event.old_starting_index,
                old_count: event.old_items.len() as i32,
            },
        );
    }

    fn scroll_into_view(this: &Self, index: i32) -> Option<Ref<Control>> {
        this.scroll_to(index)
    }
    fn container_from_index(this: &Self, index: i32) -> Option<Ref<Control>> {
        this.containers
            .borrow()
            .iter()
            .find(|entry| entry.index == index && index >= 0)
            .map(|entry| entry.control.clone())
    }
    fn index_from_container(this: &Self, container: &Ref<Control>) -> i32 {
        this.containers
            .borrow()
            .iter()
            .find(|entry| entry.control == *container)
            .map_or(-1, |entry| entry.index)
    }
    fn get_realized_containers(this: &Self) -> Option<Vec<Ref<Control>>> {
        Some(this.presenter.get_realized_elements())
    }
    fn get_control_in_direction(
        this: &Self,
        direction: NavigationDirection,
        from: Option<&Ref<InputElement>>,
        wrap: bool,
    ) -> Option<Ref<InputElement>> {
        this.navigate(direction, from, wrap)
    }
}

impl ArticlePanel {
    pub(super) fn capture_anchor(&self) {
        if self.anchor.borrow().is_some() || self.jumping.get() {
            return;
        }
        let Some(vm) = self.vm.upgrade() else { return };
        let route = vm.view();
        if self.route.borrow().as_ref() != Some(&route)
            || self.generation.get() != vm.library_generation()
        {
            return;
        }
        let viewport = self.presenter.viewport();
        if !viewport.y.is_finite() || viewport.height <= 0.0 {
            return;
        }
        let content = self
            .body_anchors
            .borrow()
            .iter()
            .filter_map(|control| {
                let position = control.translate_point(Point::default(), self)?;
                (position.y < viewport.bottom()
                    && position.y + control.bounds().height > viewport.y)
                    .then(|| (control.clone(), position.y - viewport.y))
            })
            .min_by(|(_, a), (_, b)| a.abs().total_cmp(&b.abs()));
        let candidate = content
            .and_then(|(control, relative)| {
                self.containers
                    .borrow()
                    .iter()
                    .find(|entry| entry.index >= 0 && entry.control.is_visual_ancestor_of(&control))
                    .map(|entry| {
                        (
                            entry.index,
                            viewport.y - entry.control.bounds().y,
                            Self::remember_block(&entry.control, &control, relative),
                        )
                    })
            })
            .or_else(|| {
                self.containers
                    .borrow()
                    .iter()
                    .find(|entry| {
                        let bounds = entry.control.bounds();
                        entry.index >= 0
                            && bounds.height > 0.0
                            && bounds.y <= viewport.y
                            && bounds.bottom() > viewport.y
                    })
                    .map(|entry| (entry.index, viewport.y - entry.control.bounds().y, None))
            });
        if let Some((index, within, content)) = candidate {
            if let Some(id) = self.metrics.borrow().id(index as usize) {
                *self.anchor.borrow_mut() = Some(Anchor {
                    id,
                    within,
                    route,
                    epoch: self.epoch.get(),
                    generation: vm.library_generation(),
                    content,
                });
            }
        }
    }

    pub(super) fn restore_anchor_after_layout(&self) {
        let Some(anchor) = self.anchor.borrow_mut().take() else {
            return;
        };
        let weak = self.to_ref().downgrade();
        Dispatcher::ui_thread().post_local(
            move || {
                let Some(panel) = weak.upgrade() else { return };
                if !panel.can_restore(&anchor) {
                    return;
                }
                // When a paragraph survives resize/reflow, preserve its screen
                // position. Restoring the old pixel within the article would
                // undo the native provider's more precise block correction.
                let exact = panel.block_anchor_offset(&anchor);
                let target =
                    exact.or_else(|| panel.metrics.borrow().restore(anchor.id, anchor.within));
                let Some(target) = target else { return };
                panel.apply_restored_offset(target);
                if exact.is_none() && panel.can_restore(&anchor) {
                    // The first offset can use estimates for unvisited overscan
                    // rows. Measure once at the new viewport, then correct the
                    // same block/pixel using the resulting actual prefix. This
                    // is bounded and never recursively schedules restoration.
                    let index = panel.metrics.borrow().index(anchor.id);
                    let Some(index) = index else { return };
                    panel.measure_requested(index as i32, target);
                    if !panel.can_restore(&anchor) {
                        return;
                    }
                    if let Some(target) = panel
                        .block_anchor_offset(&anchor)
                        .or_else(|| panel.metrics.borrow().restore(anchor.id, anchor.within))
                    {
                        panel.apply_restored_offset(target);
                    }
                }
            },
            DispatcherPriority::BACKGROUND,
        );
    }

    fn can_restore(&self, anchor: &Anchor) -> bool {
        self.is_attached_to_visual_tree()
            && self.epoch.get() == anchor.epoch
            && self.vm.upgrade().is_some_and(|vm| {
                vm.view() == anchor.route && vm.library_generation() == anchor.generation
            })
    }

    fn apply_restored_offset(&self, target: f64) {
        if let Some(scroll) = self.scroll_viewer() {
            if target.is_finite() && (scroll.offset().y - target).abs() > 0.5 {
                if let Some(ui) = self.ui.upgrade() {
                    ui.programmatic_scroll.set(Some(Instant::now()));
                }
                scroll.set_offset(Vector::new(scroll.offset().x, target));
            }
        }
    }
}
