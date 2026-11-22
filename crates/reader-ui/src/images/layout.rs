//! Reserve publisher dimensions before download; retain native aspect ratio afterwards.
pub(crate) fn size(
    available: f64,
    declared: Option<(f64, f64)>,
    decoded: Option<(f64, f64)>,
) -> (f64, f64) {
    let valid = |(width, height): &(f64, f64)| {
        width.is_finite() && height.is_finite() && *width > 0.0 && *height > 0.0
    };
    if let Some((width, height)) = declared.filter(valid).or_else(|| decoded.filter(valid)) {
        let scale = (available / width).min(900.0 / height).min(1.0);
        ((width * scale).max(1.0), (height * scale).max(1.0))
    } else {
        (available, (available * 9.0 / 16.0).min(240.0))
    }
}

/// Native effective viewports are relative to the image's local bounds.
/// Half a screen of overscan warms images before they enter the reading window.
pub(crate) fn near_viewport(top: f64, height: f64, image_height: f64) -> bool {
    let overscan = (height * 0.5).clamp(100.0, 600.0);
    height > 0.0 && image_height > 0.0 && top < image_height + overscan && top + height > -overscan
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn declared_geometry_is_stable_when_image_arrives() {
        assert_eq!(size(600.0, Some((1200.0, 800.0)), None), (600.0, 400.0));
        assert_eq!(
            size(600.0, Some((1200.0, 800.0)), Some((1300.0, 867.0))),
            (600.0, 400.0)
        );
        assert_eq!(
            size(250.0, Some((1200.0, 800.0)), None),
            (250.0, 166.66666666666669)
        );
    }
    #[test]
    fn unknown_geometry_is_bounded_and_decoded_images_are_not_enlarged() {
        assert_eq!(size(650.0, None, None), (650.0, 240.0));
        assert_eq!(size(650.0, None, Some((120.0, 80.0))), (120.0, 80.0));
        assert_eq!(size(650.0, None, Some((500.0, 2000.0))), (225.0, 900.0));
    }

    #[test]
    fn only_visible_or_nearby_images_retain_decoded_bitmaps() {
        assert!(near_viewport(0.0, 800.0, 400.0));
        assert!(near_viewport(-900.0, 800.0, 400.0));
        assert!(near_viewport(600.0, 800.0, 400.0));
        assert!(!near_viewport(1600.0, 800.0, 400.0));
        assert!(!near_viewport(-1600.0, 800.0, 400.0));
        assert!(!near_viewport(0.0, 0.0, 400.0));
    }
}
