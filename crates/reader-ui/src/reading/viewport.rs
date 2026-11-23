//! A finite realization window must cover the visible pixels even while the
//! indexed extent is an estimate from the content's previous geometry.

pub(super) fn realization_range(offset: f64, height: f64, extent: f64) -> Option<(f64, f64)> {
    if !offset.is_finite() || !height.is_finite() || height <= 0.0 {
        return None;
    }
    let offset = offset.max(0.0);
    let visible_end = offset + height;
    if !visible_end.is_finite() {
        return None;
    }
    // Expansion can make the first measured row taller than the old total.
    // Capping below the visible bottom would stop realization at that row,
    // even when following headlines should fill the rest of the viewport.
    let limit = extent.max(visible_end);
    let buffer = height * 0.5;
    let mut start = (offset - buffer).max(0.0);
    let mut end = (visible_end + buffer).min(limit);
    let missing_before = buffer - (offset - start);
    let missing_after = buffer - (end - visible_end);
    if missing_before > 0.0 {
        end = (end + missing_before).min(limit);
    } else if missing_after > 0.0 {
        start = (start - missing_after).max(0.0);
    }
    Some((start, end))
}

#[cfg(test)]
mod tests {
    use super::realization_range;

    #[test]
    fn expanding_short_content_cannot_clip_the_visible_realization_window() {
        // Nine collapsed phone headlines occupy 405px. When the first expands
        // to 478px, the next rows still need to fill the 571px native viewport.
        assert_eq!(realization_range(0.0, 571.0, 405.0), Some((0.0, 571.0)));
        assert_eq!(realization_range(0.0, 571.0, 838.0), Some((0.0, 838.0)));
    }

    #[test]
    fn overscan_stays_bounded_at_either_end_and_in_large_streams() {
        for (offset, extent) in [(0.0, 1e9), (50_000.0, 1e9), (943.0, 1514.0)] {
            let (start, end) = realization_range(offset, 571.0, extent).unwrap();
            assert!(start >= 0.0 && start <= offset);
            assert!(end >= offset + 571.0);
            assert!(end - start <= 1142.0);
        }
        assert_eq!(realization_range(0.0, 0.0, 100.0), None);
        assert_eq!(realization_range(0.0, f64::INFINITY, 100.0), None);
        assert_eq!(realization_range(f64::NAN, 571.0, 100.0), None);
    }
}
