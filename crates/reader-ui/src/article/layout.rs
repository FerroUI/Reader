use crate::rows::Row;
use reader_domain::Density;

pub(crate) fn action_height(row: &Row) -> f64 {
    let normal: f64 = if row.touch {
        44.0
    } else if row.list_mode {
        // A 13px normal text line plus the native button's two focus-border
        // pixels. List actions retain inline density; cards use their own bar.
        17.0
    } else {
        26.0
    };
    // Font-relative line boxes grow under magnification; fixed touch targets
    // and the card's existing extra space remain usable at every scale.
    normal.max(15.0 * row.scale.factor() + 2.0)
}

pub(crate) fn headline_height(row: &Row) -> f64 {
    if row.touch {
        44.0
    } else {
        match row.density {
            Density::Comfortable => 37.0,
            Density::Cozy => 31.0,
            Density::Compact => 27.0,
        }
    }
}

pub(crate) fn right_inset(density: Density) -> f64 {
    match density {
        Density::Comfortable => 33.0,
        Density::Cozy => 21.0,
        Density::Compact => 10.0,
    }
}

pub(super) fn card_padding(row: &Row) -> f64 {
    if row.narrow {
        12.0
    } else {
        match row.density {
            Density::Comfortable => 20.0,
            Density::Cozy => 10.0,
            Density::Compact => 5.0,
        }
    }
}

/// Images reserve their intrinsic ratio before decode. Follow the same actual
/// shell/sidebar and article insets as text, with room for the native scrollbar.
pub(crate) fn media_width(row: &Row, viewport: f64, sidebar: f64) -> f64 {
    let inset = if row.list_mode {
        1.0 + 26.0 * row.scale.factor()
    } else {
        let right = if row.narrow {
            10.0
        } else {
            right_inset(row.density)
        };
        5.0 + right + 2.0 + 2.0 * card_padding(row)
    };
    (viewport - sidebar - inset - 16.0)
        .max(1.0)
        .min(crate::reading::scale::body_max_width(row.wide_body))
}
