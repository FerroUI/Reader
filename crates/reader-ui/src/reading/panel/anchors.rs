use super::ArticlePanel;
use ferroui_base::threading::{Dispatcher, DispatcherPriority};
use ferroui_base::{Point, Rect, Ref};
use ferroui_controls::{as_scroll_anchor_provider, Control, StackPanel};

impl ArticlePanel {
    /// Whole-row anchors cannot preserve a paragraph below a newly decoded
    /// image in the same article. Offer visible text blocks to the same native
    /// anchor provider, with a strict bound independent of article/feed length.
    pub(super) fn queue_body_anchors(&self) {
        if self.anchors_queued.replace(true) {
            return;
        }
        let weak = self.to_ref().downgrade();
        Dispatcher::ui_thread().post_local(
            move || {
                let Some(panel) = weak.upgrade() else { return };
                panel.anchors_queued.set(false);
                if panel.is_attached_to_visual_tree() {
                    panel.update_body_anchors();
                }
            },
            DispatcherPriority::BACKGROUND,
        );
    }

    fn update_body_anchors(&self) {
        let Some(provider) = self
            .get_visual_ancestors()
            .find(|visual| as_scroll_anchor_provider(visual).is_some())
        else {
            return;
        };
        let Some(anchor_provider) = as_scroll_anchor_provider(&provider) else {
            return;
        };
        *self.anchor_provider.borrow_mut() = Some(provider.downgrade());
        let viewport = self.presenter.viewport();
        let region = Rect::new(
            viewport.x,
            (viewport.y - viewport.height * 0.5).max(0.0),
            viewport.width,
            viewport.height * 2.0,
        );
        let next = self.visible_body_blocks(region);
        let old = self.body_anchors.replace(next.clone());
        for control in &next {
            if !old.contains(control) {
                anchor_provider.register_anchor_candidate(control);
            }
        }
        // Run after the native arrange has compared the previous anchor's
        // position. Unregistering it during arrange would suppress correction.
        for control in old {
            if !next.contains(&control) {
                anchor_provider.unregister_anchor_candidate(&control);
            }
        }
    }

    fn visible_body_blocks(&self, region: Rect) -> Vec<Ref<Control>> {
        let mut candidates = Vec::new();
        for container in self.presenter.get_realized_elements() {
            // ArticleBody is before its content in the visual traversal. Do not
            // enumerate thousands of paragraphs merely to find a visible one.
            let Some(body) = container
                .get_visual_descendants()
                .take(96)
                .find(|visual| visual.has_name("ArticleBody"))
                .and_then(|visual| visual.cast::<StackPanel>())
            else {
                continue;
            };
            let Some(origin) = body.translate_point(Point::default(), self) else {
                continue;
            };
            let start = region.top() - origin.y;
            let end = region.bottom() - origin.y;
            let children = body.children();
            // Reader's body is an ordered vertical stack with positive spacing.
            // Binary search the arranged block bounds, then inspect only blocks
            // around the viewport. Quotes, code and tables anchor as one block.
            let (mut low, mut high) = (0, children.count());
            while low < high {
                let middle = low + (high - low) / 2;
                if children.get(middle).bounds().bottom() < start {
                    low = middle + 1;
                } else {
                    high = middle;
                }
            }
            for index in low..children.count() {
                let child = children.get(index);
                let bounds = child.bounds();
                if bounds.top() > end {
                    break;
                }
                if bounds.height > 0.0 && child.is_effectively_visible() {
                    candidates.push(child);
                    if candidates.len() == 64 {
                        return candidates;
                    }
                }
            }
        }
        candidates
    }

    pub(super) fn clear_body_anchors(&self, container: Option<&Ref<Control>>) {
        let removed = {
            let mut anchors = self.body_anchors.borrow_mut();
            let mut removed = Vec::new();
            anchors.retain(|anchor| {
                if container.is_none_or(|container| container.is_visual_ancestor_of(anchor)) {
                    removed.push(anchor.clone());
                    false
                } else {
                    true
                }
            });
            removed
        };
        if let Some(provider) = self
            .anchor_provider
            .borrow()
            .as_ref()
            .and_then(|provider| provider.upgrade())
        {
            if let Some(provider) = as_scroll_anchor_provider(&provider) {
                for anchor in removed {
                    provider.unregister_anchor_candidate(&anchor);
                }
            }
        }
    }
}
