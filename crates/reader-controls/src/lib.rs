//! Reusable Google Reader-era controls layered on FerroUI's Material theme.
//! Native control templates retain keyboard, focus, automation and input semantics.
mod buttons;
mod icons;
mod templates;
mod theme;
pub use buttons::*;
pub use icons::{icon, Icon};
pub use theme::install_theme;
ferroui_markup_xaml::include_compiled_xaml!();
