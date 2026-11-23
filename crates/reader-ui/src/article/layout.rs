use crate::rows::Row;
use reader_domain::Density;

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
