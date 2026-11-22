//! Portable feed document, parsing, queries and mutations. No platform or UI dependencies.
mod action;
mod article_content;
mod connection;
mod document;
mod feed;
mod folders;
mod html;
mod legacy_samples;
mod merge;
mod model;
mod mutations;
mod opml;
mod position;
mod query;
#[cfg(feature = "test-support")]
mod sample;
mod starter;
mod stream;
mod trends;

pub use action::{Action, Changes};
pub use article_content::{FullArticle, MAX_ARTICLE_CACHE_BYTES, MAX_ARTICLE_HTML_BYTES};
pub use connection::{FeedConnection, subscription_url, validate_feed_proxy};
pub use feed::{ParsedArticle, ParsedFeed, parse_feed};
pub use html::{plain_text, safe_url};
pub use model::*;
pub use opml::{OpmlSubscription, parse_opml};
pub use query::{matches_view, search_matches};
pub use starter::{DEFAULT_FEEDS, DefaultFeed};
pub use stream::StreamSettings;
pub use trends::{FeedTrend, Trends};
