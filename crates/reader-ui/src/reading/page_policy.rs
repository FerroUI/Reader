//! Reader's Space command pages within a tall item, then opens its neighbor.
//! Kept independent of controls so the variable-height boundary rules are testable.

#[derive(Clone, Copy, Debug)]
pub(super) struct Current {
    pub index: usize,
    pub top: f64,
    pub bottom: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum Action {
    None,
    First,
    Previous,
    Next,
    NextStream,
    Scroll(f64),
}

pub(super) fn distance(viewport: f64) -> f64 {
    // The original client overlaps successive pages by 50 CSS pixels. Tiny
    // embedded windows must still move in the requested direction.
    (viewport - 50.0).max(1.0)
}

pub(super) fn decide(
    forward: bool,
    count: usize,
    current: Option<Current>,
    offset: f64,
    viewport: f64,
) -> Action {
    if count == 0 || !offset.is_finite() || !viewport.is_finite() || viewport <= 0.0 {
        return Action::None;
    }
    let Some(current) = current else {
        return if forward { Action::First } else { Action::None };
    };
    if current.index >= count
        || !current.top.is_finite()
        || !current.bottom.is_finite()
        || current.bottom < current.top
    {
        return Action::None;
    }
    // Native layout uses subpixels; the original DOM exposed integer offsets.
    let edge_visible = if forward {
        current.bottom <= offset + viewport + 0.5
    } else {
        current.top >= offset - 0.5
    };
    if !edge_visible {
        return Action::Scroll(distance(viewport) * if forward { 1.0 } else { -1.0 });
    }
    match (forward, current.index) {
        (true, index) if index + 1 == count => Action::NextStream,
        (true, _) => Action::Next,
        (false, 0) => Action::None,
        (false, _) => Action::Previous,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(index: usize, top: f64, bottom: f64) -> Option<Current> {
        Some(Current { index, top, bottom })
    }

    #[test]
    fn initial_reverse_and_empty_stream_do_not_wrap() {
        assert_eq!(decide(false, 3, None, 0.0, 700.0), Action::None);
        assert_eq!(decide(true, 3, None, 900.0, 700.0), Action::First);
        assert_eq!(decide(true, 0, None, 0.0, 700.0), Action::None);
        assert_eq!(
            decide(false, 3, item(0, 0.0, 180.0), 0.0, 700.0),
            Action::None
        );
    }

    #[test]
    fn short_items_advance_only_one_item_per_command() {
        assert_eq!(
            decide(true, 3, item(0, 0.0, 180.0), 0.0, 700.0),
            Action::Next
        );
        assert_eq!(
            decide(false, 3, item(1, 180.0, 420.0), 0.0, 700.0),
            Action::Previous
        );
    }

    #[test]
    fn tall_items_page_with_overlap_until_their_edge_is_visible() {
        let tall = item(1, 200.0, 2800.0);
        assert_eq!(decide(true, 3, tall, 200.0, 700.0), Action::Scroll(650.0));
        assert_eq!(decide(true, 3, tall, 1500.0, 700.0), Action::Scroll(650.0));
        assert_eq!(decide(true, 3, tall, 2100.0, 700.0), Action::Next);
        assert_eq!(
            decide(false, 3, tall, 1500.0, 700.0),
            Action::Scroll(-650.0)
        );
        assert_eq!(decide(false, 3, tall, 200.0, 700.0), Action::Previous);
    }

    #[test]
    fn last_item_finishes_its_content_before_advancing_stream() {
        let last = item(2, 2800.0, 4200.0);
        assert_eq!(decide(true, 3, last, 2800.0, 700.0), Action::Scroll(650.0));
        assert_eq!(decide(true, 3, last, 3500.0, 700.0), Action::NextStream);
    }

    #[test]
    fn pixel_rounding_and_small_viewports_remain_directional() {
        assert_eq!(
            decide(true, 3, item(1, 0.0, 700.25), 0.0, 700.0),
            Action::Next
        );
        assert_eq!(
            decide(false, 3, item(1, 99.75, 800.0), 100.0, 700.0),
            Action::Previous
        );
        assert_eq!(
            decide(true, 3, item(0, 0.0, 100.0), 0.0, 30.0),
            Action::Scroll(1.0)
        );
        assert_eq!(
            decide(false, 3, item(0, 0.0, 100.0), 50.0, 30.0),
            Action::Scroll(-1.0)
        );
    }

    #[test]
    fn invalid_layout_never_moves_or_opens_items() {
        for viewport in [0.0, -1.0, f64::NAN, f64::INFINITY] {
            assert_eq!(decide(true, 3, None, 0.0, viewport), Action::None);
        }
        assert_eq!(
            decide(true, 3, item(3, 0.0, 200.0), 0.0, 700.0),
            Action::None
        );
        assert_eq!(
            decide(true, 3, item(1, f64::NAN, 200.0), 0.0, 700.0),
            Action::None
        );
    }
}
