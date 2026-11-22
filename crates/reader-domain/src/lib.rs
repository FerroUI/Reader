//! Portable feed document, parsing, queries and mutations. No platform or UI dependencies.
mod action;
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
pub use connection::{subscription_url, validate_feed_proxy, FeedConnection};
pub use feed::{parse_feed, ParsedArticle, ParsedFeed};
pub use html::{plain_text, safe_url};
pub use model::*;
pub use opml::{parse_opml, OpmlSubscription};
pub use query::{matches_view, search_matches};
pub use starter::{DefaultFeed, DEFAULT_FEEDS};
pub use stream::StreamSettings;
pub use trends::{FeedTrend, Trends};
