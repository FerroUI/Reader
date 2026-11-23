//! Transient magnification shared by keyboard and view-menu commands.
pub(crate) use super::scale_policy::{body_max_width, Scale};
use crate::session::Session;

pub(crate) fn change(ui: &Session, larger: bool) {
    let current = ui.article_scale.get_untracked();
    if current.can_change(larger) {
        // Reflow is not a new reading gesture, even if a preceding wheel event
        // is still awaiting its deferred auto-read frame.
        ui.programmatic_scroll.set(Some(std::time::Instant::now()));
        ui.article_scale.set(current.changed(larger));
    }
}
