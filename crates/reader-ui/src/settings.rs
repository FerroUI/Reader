//! Small, independently composed Reader settings pages.
mod connection;
mod controls;
mod import_export;
mod preferences;
mod send_to;
mod send_to_editor;
mod subscriptions;
pub use import_export::view as import_export;
pub use preferences::view as preferences;
pub use send_to::view as send_to;
pub use send_to_editor::view as edit_send_to;
pub use subscriptions::view as subscriptions;
