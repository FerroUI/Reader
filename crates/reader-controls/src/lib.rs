//! Reusable Google Reader-era controls layered on FerroUI's Material theme.
//! Native control templates retain keyboard, focus, automation and input semantics.
mod buttons;
mod checks;
mod icons;
mod inputs;
mod menus;
mod preferences;
mod read_state;
mod templates;
mod theme;
mod touch;
pub use buttons::*;
pub use icons::{icon, Icon};
pub use preferences::{preference_check_theme, preference_choice_theme};
pub use theme::install_theme;
ferroui_markup_xaml::include_compiled_xaml!();
