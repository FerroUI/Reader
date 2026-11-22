//! Platform-independent article projection and optional publisher extraction.
//! Extraction stays out of the browser UI binary and runs in its own worker.
pub mod projection;

#[cfg(feature = "extraction")]
mod extraction;
#[cfg(feature = "extraction")]
pub use extraction::{extract, ExtractedArticle, MAX_DOCUMENT_BYTES};
