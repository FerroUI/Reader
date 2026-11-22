use super::ArticlePanel;
use ferroui_base::input::{InputElement, NavigationDirection};
use ferroui_base::{Rect, Ref, Vector};
use ferroui_controls::{Control, ScrollViewer};

impl ArticlePanel {
    pub(super) fn scroll_viewer(&self) -> Option<Ref<ScrollViewer>> {
        self.get_visual_ancestors()
            .find_map(|visual| visual.cast::<ScrollViewer>())
    }

    pub(super) fn scroll_to(&self, index: i32) -> Option<Ref<Control>> {
        if index < 0
            || index as usize >= self.items().count()
            || self.presenter.is_in_layout()
            || !self.is_attached_to_visual_tree()
            || self.jumping.replace(true)
        {
            return None;
        }
        self.cancel_restore();
        // The native TreeDataGrid BringIntoView estimates unseen rows from a
        // window average and measures with infinite width. Articles must wrap at
        // their real width and seek through the persistent pixel index instead.
        self.update_layout();
        self.synchronize_metrics(self.bounds().width);
        let result = if let Some(control) = self.presenter.try_get_element(index) {
            self.bring_start_into_view(&control);
            Some(control)
        } else if let Some(scroll) = self.scroll_viewer() {
            let offset = self.metrics.borrow().extents.start(index as usize);
            scroll.set_offset(Vector::new(scroll.offset().x, offset));
            self.invalidate_measure();
            self.update_layout();
            let control = self.presenter.try_get_element(index);
            if let Some(control) = &control {
                self.bring_start_into_view(control);
            }
            control
        } else {
            None
        };
        self.jumping.set(false);
        result
    }

    fn bring_start_into_view(&self, control: &Ref<Control>) {
        let height = self
            .scroll_viewer()
            .map_or(1.0, |scroll| scroll.viewport().height.max(1.0));
        control.bring_into_view_rect(Rect::new(
            0.0,
            0.0,
            control.bounds().width,
            control.bounds().height.min(height),
        ));
    }

    pub(super) fn navigate(
        &self,
        direction: NavigationDirection,
        from: Option<&Ref<InputElement>>,
        wrap: bool,
    ) -> Option<Ref<InputElement>> {
        let count = self.items().count() as i32;
        if count == 0 {
            return None;
        }
        let current = from
            .and_then(|from| from.cast::<Control>())
            .map_or(-1, |control| self.index_from_container(&control));
        let mut next = match direction {
            NavigationDirection::First => 0,
            NavigationDirection::Last => count - 1,
            NavigationDirection::Next | NavigationDirection::Down if current >= 0 => current + 1,
            NavigationDirection::Previous | NavigationDirection::Up if current >= 0 => current - 1,
            _ => return None,
        };
        if wrap {
            next = next.rem_euclid(count);
        }
        self.scroll_to(next).map(|control| control.upcast())
    }
}
