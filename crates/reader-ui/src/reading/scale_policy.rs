//! Reader's magnification is a session preference applied to the current entry.
//! It is independent of the saved base font used by every article.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Scale(u16);

impl Default for Scale {
    fn default() -> Self {
        Self(100)
    }
}

impl Scale {
    pub(crate) fn percent(self) -> u16 {
        self.0
    }

    pub(crate) fn factor(self) -> f64 {
        f64::from(self.percent()) / 100.0
    }

    pub(crate) fn can_change(self, larger: bool) -> bool {
        self.changed(larger) != self
    }

    pub(crate) fn changed(self, larger: bool) -> Self {
        Self(if larger {
            (self.0 + 10).min(150)
        } else {
            self.0.saturating_sub(10).max(100)
        })
    }

    pub(crate) fn for_current(self, current: bool) -> Self {
        if current {
            self
        } else {
            Self::default()
        }
    }

    /// Headings magnify from their ordinary size, rather than waiting for the
    /// enlarged paragraph font to overtake their unscaled minimum.
    pub(crate) fn text_size(self, base: f64, heading: u8) -> f64 {
        let normal = if heading == 0 {
            base
        } else {
            (28.0 - f64::from(heading) * 2.0).max(base)
        };
        normal * self.factor()
    }
}

pub(crate) fn body_max_width(sidebar_hidden: bool) -> f64 {
    if sidebar_hidden {
        f64::INFINITY
    } else {
        650.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn magnification_uses_ten_point_steps_and_saturates_at_both_boundaries() {
        let mut scale = Scale::default();
        assert!(!scale.can_change(false));
        for percent in [110, 120, 130, 140, 150, 150] {
            scale = scale.changed(true);
            assert_eq!(scale.percent(), percent);
        }
        assert!(!scale.can_change(true));
        for percent in [140, 130, 120, 110, 100, 100] {
            scale = scale.changed(false);
            assert_eq!(scale.percent(), percent);
        }
    }

    #[test]
    fn only_the_current_article_uses_the_session_scale_and_base_font_is_preserved() {
        let scale = Scale::default().changed(true).changed(true);
        assert_eq!(scale.for_current(false), Scale::default());
        assert_eq!(scale.for_current(true).percent(), 120);
        assert_eq!(scale.for_current(false).text_size(19.0, 0), 19.0);
        assert!((scale.for_current(true).text_size(19.0, 0) - 22.8).abs() < 1e-9);
        assert_eq!(scale.for_current(false).text_size(19.0, 1), 26.0);
        assert!((scale.for_current(true).text_size(19.0, 1) - 31.2).abs() < 1e-9);
    }

    #[test]
    fn hidden_sidebar_removes_only_the_body_width_cap() {
        assert_eq!(body_max_width(false), 650.0);
        assert!(body_max_width(true).is_infinite());
    }
}
