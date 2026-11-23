use super::page_policy::{self, Action};
use crate::session::Session;
use ferroui_base::{Ref, Vector};
use ferroui_controls::ScrollViewer;
use reader_domain::View;
use reader_state::AppViewModel;

/// Google Reader's `iu`: Space opens the first/next item only after the current
/// item's bottom is visible; Shift+Space applies the inverse rule at its top.
pub(crate) fn page(vm: &AppViewModel, ui: &Session, forward: bool) {
    if super::blocked(vm, ui) {
        return;
    }
    if matches!(
        vm.view(),
        View::Home | View::Trends | View::Explore | View::Settings
    ) {
        if let Some(scroll) = ui
            .page_scroll
            .get_untracked()
            .filter(|s| s.is_attached_to_visual_tree())
        {
            let viewport = scroll.viewport().height;
            if viewport.is_finite() && viewport > 0.0 {
                scroll_by(
                    &scroll,
                    page_policy::distance(viewport) * if forward { 1.0 } else { -1.0 },
                );
            }
        }
        return;
    }
    let Some(list) = ui
        .article_list
        .get_untracked()
        .filter(|list| list.is_attached_to_visual_tree())
    else {
        return;
    };
    let Some(scroll) = list
        .scroll()
        .and_then(|control| control.cast::<ScrollViewer>())
    else {
        return;
    };
    // Real input supersedes any queued restoration from image reflow or edits.
    super::panel::user_input(&list);
    let Some((count, current)) = super::panel::position(&list, vm.cursor().or(vm.selected()))
    else {
        return;
    };
    match page_policy::decide(
        forward,
        count,
        current,
        scroll.offset().y,
        scroll.viewport().height,
    ) {
        Action::None => {}
        Action::First | Action::Next => vm.next_article(1),
        Action::Previous => vm.next_article(-1),
        Action::NextStream => crate::sidebar::cursor::open_next_unread(vm, ui),
        Action::Scroll(delta) => {
            super::user_scroll();
            scroll_by(&scroll, delta);
        }
    }
}

fn scroll_by(scroll: &Ref<ScrollViewer>, delta: f64) {
    let offset = scroll.offset();
    let maximum = (scroll.extent().height - scroll.viewport().height).max(0.0);
    if offset.y.is_finite() && maximum.is_finite() && delta.is_finite() {
        scroll.set_offset(Vector::new(
            offset.x,
            (offset.y + delta).clamp(0.0, maximum),
        ));
    }
}
