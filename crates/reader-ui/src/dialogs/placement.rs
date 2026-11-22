//! Follow native layout without recreating the active TextBox or polling the host.
use ferroui_base::{Rect, Ref, Thickness};
use ferroui_controls::Border;
use ferroui_view::on_cleanup;
use reader_state::AppViewModel;
use std::rc::Rc;

pub(super) fn follow(
    popup: &Ref<Border>,
    vm: &Rc<AppViewModel>,
    anchor: impl Fn(&AppViewModel) -> Option<Rect> + 'static,
) {
    let target = popup.downgrade();
    let model = Rc::downgrade(vm);
    let layout = popup.layout_updated(move || {
        let (Some(popup), Some(vm)) = (target.upgrade(), model.upgrade()) else {
            return;
        };
        let Some(anchor) = anchor(&vm) else {
            return;
        };
        let size = popup.bounds();
        if size.width <= 0.0 || size.height <= 0.0 {
            return;
        }
        let x = anchor
            .x
            .clamp(8.0, (vm.viewport() - size.width - 8.0).max(8.0));
        let y = if anchor.bottom() + size.height <= vm.viewport_height() - 8.0 {
            anchor.bottom()
        } else {
            (anchor.y - size.height).max(8.0)
        };
        let margin = Thickness::new(x, y, 0.0, 0.0);
        if popup.margin() != margin {
            popup.set_margin(margin);
        }
        let maximum = (vm.viewport_height() - y - 8.0).max(1.0);
        if popup.max_height() != maximum {
            popup.set_max_height(maximum);
        }
    });
    on_cleanup(move || layout.dispose());
}
