//! First-use subscriptions contain publisher metadata, never invented articles.
use crate::{AppState, Feed, Id};

#[derive(Clone, Copy, Debug)]
pub struct DefaultFeed {
    pub title: &'static str,
    pub url: &'static str,
    pub site_url: &'static str,
    pub folder: &'static str,
    pub description: &'static str,
}

pub const DEFAULT_FEEDS: &[DefaultFeed] = &[
    DefaultFeed {
        title: "Hacker News",
        url: "https://news.ycombinator.com/rss",
        site_url: "https://news.ycombinator.com/",
        folder: "Technology",
        description: "Technology and ideas from the Hacker News community.",
    },
    DefaultFeed {
        title: "Ars Technica",
        url: "https://feeds.arstechnica.com/arstechnica/index",
        site_url: "https://arstechnica.com/",
        folder: "Technology",
        description: "Technology, science and policy reporting.",
    },
    DefaultFeed {
        title: "The Verge",
        url: "https://www.theverge.com/rss/index.xml",
        site_url: "https://www.theverge.com/",
        folder: "Technology",
        description: "Technology, products and the future of the web.",
    },
    DefaultFeed {
        title: "Rust Blog",
        url: "https://blog.rust-lang.org/feed.xml",
        site_url: "https://blog.rust-lang.org/",
        folder: "Development",
        description: "Updates from the Rust programming language team.",
    },
    DefaultFeed {
        title: "GitHub Blog",
        url: "https://github.blog/feed/",
        site_url: "https://github.blog/",
        folder: "Development",
        description: "Developer tools, product updates and software engineering.",
    },
    DefaultFeed {
        title: "BBC News",
        url: "https://feeds.bbci.co.uk/news/rss.xml",
        site_url: "https://www.bbc.com/news",
        folder: "News",
        description: "Headlines and reporting from BBC News.",
    },
];

impl DefaultFeed {
    pub(crate) fn subscription(self, id: Id) -> Feed {
        Feed {
            id,
            title: self.title.into(),
            url: self.url.into(),
            site_url: self.site_url.into(),
            folder: self.folder.into(),
            folders: vec![self.folder.into()],
            description: self.description.into(),
            ..Feed::default()
        }
    }
}

impl AppState {
    /// Hosts call this only when no saved library exists. An intentionally empty
    /// saved library must remain empty, and removed defaults are never restored.
    pub fn with_default_feeds() -> Self {
        let mut state = Self::new();
        for source in DEFAULT_FEEDS {
            let id = state.allocate();
            state.feeds.push(source.subscription(id));
        }
        state.folders = crate::document::clean_names(
            DEFAULT_FEEDS
                .iter()
                .map(|source| source.folder.into())
                .collect(),
        );
        state.reindex();
        state
    }
}
