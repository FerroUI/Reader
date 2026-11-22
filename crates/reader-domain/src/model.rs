use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub type Id = u64;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Feed {
    pub id: Id,
    pub title: String,
    pub url: String,
    pub site_url: String,
    pub folder: String,
    /// Reader subscriptions may carry several folder labels; `folder` is the primary compatibility label.
    pub folders: Vec<String>,
    pub description: String,
    pub fetched_at: i64,
    pub error: Option<String>,
    pub sample: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Article {
    pub id: Id,
    pub feed_id: Id,
    pub guid: String,
    pub title: String,
    pub url: String,
    pub author: String,
    pub content: String,
    /// Effective document/XML Base for relative URLs inside the article body.
    pub content_base: String,
    pub summary: String,
    pub published: i64,
    pub read: bool,
    pub starred: bool,
    pub tags: Vec<String>,
    pub note: String,
    pub read_at: Option<i64>,
    /// Derived once on import/update, never persisted in the backup document.
    #[serde(skip)]
    pub search_text: String,
}

impl Article {
    pub fn content_base_url(&self) -> &str {
        if self.content_base.is_empty() {
            &self.url
        } else {
            &self.content_base
        }
    }
    pub fn rebuild_search(&mut self) {
        self.search_text = format!(
            "{} {} {} {} {}",
            self.title,
            self.author,
            crate::plain_text(&self.content),
            self.tags.join(" "),
            self.note
        )
        .to_lowercase();
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum View {
    #[default]
    Home,
    All,
    Starred,
    Trends,
    Explore,
    Feed(Id),
    Folder(String),
    Tag(String),
    Search(String),
    Settings,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViewMode {
    #[default]
    Expanded,
    List,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum SortOrder {
    #[default]
    Newest,
    Oldest,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Density {
    #[default]
    Comfortable,
    Cozy,
    Compact,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub view_mode: ViewMode,
    pub sort: SortOrder,
    pub unread_only: bool,
    pub show_all_subscriptions: bool,
    pub mark_read_on_scroll: bool,
    pub confirm_mark_all: bool,
    pub font_size: f64,
    pub density: Density,
    pub start_page: View,
    pub feed_connection: crate::FeedConnection,
    /// Optional trusted CORS endpoint; `{url}` is replaced with the encoded feed URL.
    pub feed_proxy: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            view_mode: ViewMode::Expanded,
            sort: SortOrder::Newest,
            unread_only: false,
            show_all_subscriptions: true,
            mark_read_on_scroll: true,
            confirm_mark_all: true,
            font_size: 13.0,
            density: Density::Comfortable,
            start_page: View::Home,
            feed_connection: crate::FeedConnection::Automatic,
            feed_proxy: String::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AppState {
    pub version: u32,
    pub feeds: Vec<Feed>,
    pub articles: Vec<Article>,
    pub folders: Vec<String>,
    pub settings: Settings,
    pub stream_settings: BTreeMap<String, crate::StreamSettings>,
    pub next_id: Id,
    #[serde(skip)]
    pub(crate) article_index: BTreeMap<Id, usize>,
    #[serde(skip)]
    pub(crate) feed_index: BTreeMap<Id, usize>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            version: 1,
            feeds: Vec::new(),
            articles: Vec::new(),
            folders: Vec::new(),
            settings: Settings::default(),
            stream_settings: BTreeMap::new(),
            next_id: 1,
            article_index: BTreeMap::new(),
            feed_index: BTreeMap::new(),
        }
    }
}
