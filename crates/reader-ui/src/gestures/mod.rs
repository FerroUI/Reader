//! Input arbitration leaves scrolling and native controls in charge of capture.
//! Only a completed, single-contact tap may activate a command after touch input.
mod guard;
pub(crate) mod motion;
mod tap;

pub(crate) use guard::{attach, cancel_active};
pub(crate) use tap::{attach_article_tap, attach_headline_tap};
