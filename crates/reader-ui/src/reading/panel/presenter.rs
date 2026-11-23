use super::ArticlePanel;
use ferroui_base::layout::{EffectiveViewportChangedEventArgs, Orientation};
use ferroui_base::{BoxedValue, Rect, Ref, Size};
use ferroui_controls::Control;
use ferroui_controls_treedatagrid::primitives::{
    PresenterItemsChange, TreeDataGridPresenterBase, TreeDataGridPresenterHooks,
};
use std::rc::Rc;

impl TreeDataGridPresenterHooks for ArticlePanel {
    type Item = Option<BoxedValue>;
    fn presenter_base(&self) -> &TreeDataGridPresenterBase {
        &self.presenter
    }
    fn orientation(&self) -> Orientation {
        Orientation::Vertical
    }
    fn item_count(&self) -> Option<usize> {
        Some(self.items().count())
    }
    fn item_at(&self, index: usize) -> Self::Item {
        self.items().get_at(index)
    }
    // VirtualizingPanel already subscribes to post-collection notifications.
    fn subscribe_to_items(&self, _: Rc<dyn Fn(&PresenterItemsChange)>) -> Option<u64> {
        None
    }
    fn unsubscribe_from_items(&self, _: u64) {}
    fn get_element_from_factory(&self, item: &Self::Item, index: i32) -> Ref<Control> {
        self.create_or_recycle(item, index)
    }
    fn realize_element(&self, control: &Ref<Control>, item: &Self::Item, index: i32) {
        self.prepare(control, item, index);
    }
    fn update_element_index(&self, control: &Ref<Control>, old: i32, new: i32) {
        self.update_index(control, old, new);
    }
    fn unrealize_element(&self, control: &Ref<Control>) {
        self.recycle(control);
    }

    fn measure_element(&self, index: i32, control: &Ref<Control>, available: Size) -> Size {
        let width = self.metrics.borrow().width;
        let measured = TreeDataGridPresenterBase::default_measure_element(
            control,
            Size::new(width, available.height),
        );
        self.metrics
            .borrow_mut()
            .measure(index as usize, measured.height);
        measured
    }

    fn arrange_element(&self, index: i32, control: &Ref<Control>, rect: Rect) -> Rect {
        let metrics = self.metrics.borrow();
        let rect = Rect::new(
            rect.x,
            metrics.extents.start(index as usize),
            rect.width,
            metrics.extents.height(index as usize),
        );
        drop(metrics);
        control.arrange(rect);
        rect
    }

    fn get_or_estimate_anchor_element_for_viewport(
        &self,
        start: f64,
        _: f64,
        count: i32,
    ) -> (i32, f64) {
        if let Some(target) = self.requested.get().filter(|target| target.index < count) {
            return (
                target.index,
                self.metrics.borrow().extents.start(target.index as usize),
            );
        }
        self.metrics
            .borrow()
            .extents
            .at(start)
            .map_or((0, 0.0), |(index, offset)| (index as i32, offset))
    }

    fn estimate_element_size_u(&self) -> f64 {
        let metrics = self.metrics.borrow();
        metrics.extents.total() / metrics.extents.len().max(1) as f64
    }

    fn get_measure_viewport(&self, viewport: Rect) -> Rect {
        // TreeDataGrid intersects its cached effective viewport with the old
        // panel bounds. After short content expands, that clipped rectangle
        // need not change again. The owning ScrollViewer has the actual finite
        // visible window; transient unarranged sizes retain the base fallback.
        let viewport = self
            .scroll_viewer()
            .and_then(|scroll| {
                let offset = scroll.offset();
                let size = scroll.viewport();
                (offset.x.is_finite()
                    && offset.y.is_finite()
                    && size.width.is_finite()
                    && size.height.is_finite()
                    && size.width > 0.0
                    && size.height > 0.0)
                    .then(|| Rect::new(offset.x, offset.y, size.width, size.height))
            })
            .unwrap_or(viewport);
        let viewport = if let Some(target) = self.requested.get() {
            let height = self
                .scroll_viewer()
                .map_or(viewport.height, |scroll| scroll.viewport().height);
            let height = if height.is_finite() && height > 0.0 {
                height
            } else {
                1.0
            };
            let metrics = self.metrics.borrow();
            let start = metrics.extents.start(target.index as usize);
            // EffectiveViewportChanged can still carry the previous offset in
            // this pass. Use the requested finite window, never the intervening
            // range between the old viewport and the explicit target. Measure
            // the target's start first: an old pixel deep inside an article that
            // shrank must not realize thousands of following rows to reach it.
            Rect::new(
                0.0,
                target.offset.clamp((start - height).max(0.0), start),
                metrics.width,
                height,
            )
        } else {
            viewport
        };
        // TreeDataGrid-style overscan: half a viewport on each side, redistributed
        // at the ends. Rapid wheel/touch frames inside this range only translate.
        let Some((start, end)) = crate::reading::viewport::realization_range(
            viewport.top(),
            viewport.height,
            self.metrics.borrow().extents.total(),
        ) else {
            return viewport;
        };
        Rect::new(viewport.x, start, viewport.width, end - start)
    }

    fn needs_measure_for_viewport_change(&self, measured: Rect, viewport: Rect) -> bool {
        let first = self.presenter.first_index().max(0) as usize;
        let last = first + self.presenter.realized_element_count();
        let metrics = self.metrics.borrow();
        (measured.width - viewport.width).abs() > 0.5
            || viewport.top() < measured.top()
            || viewport.bottom() > measured.bottom()
            || viewport.top() < metrics.extents.start(first)
            || viewport.bottom() > metrics.extents.start(last)
    }

    fn on_effective_viewport_changed(&self, event: &EffectiveViewportChangedEventArgs) {
        self.presenter
            .default_on_effective_viewport_changed(self, event);
        self.queue_body_anchors();
    }
}
