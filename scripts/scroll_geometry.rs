//! The production index/cache without the renderer, for the existing fast CI step.
#![allow(dead_code)]
#[path = "../crates/reader-ui/src/reading/extent.rs"]
mod extent;
#[path = "../crates/reader-ui/src/reading/geometry.rs"]
mod geometry;
#[path = "../crates/reader-ui/src/reading/page_policy.rs"]
mod page_policy;
#[path = "../crates/reader-ui/src/reading/scale_policy.rs"]
mod scale_policy;
#[path = "../crates/reader-ui/src/reading/viewport.rs"]
mod viewport;
