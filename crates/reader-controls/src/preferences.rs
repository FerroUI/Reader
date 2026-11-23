//! Reusable settings themes; native controls own input, selection and automation.
mod check;
mod choice;
mod presenter;
pub use check::theme as preference_check_theme;
pub use choice::theme as preference_choice_theme;
