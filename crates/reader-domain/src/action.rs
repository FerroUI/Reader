use crate::{FullArticle, Id, ParsedFeed, Settings, SubscriptionNode, SubscriptionSort};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Action {
    CacheFullArticle {
        id: Id,
        content: FullArticle,
    },
    ShowFullArticle {
        id: Id,
        show: bool,
    },
    SetRead {
        id: Id,
        read: bool,
    },
    ToggleStar(Id),
    SetTags {
        id: Id,
        tags: Vec<String>,
    },
    SetNote {
        id: Id,
        note: String,
    },
    RenameFeed {
        id: Id,
        title: String,
    },
    MoveFeed {
        id: Id,
        folder: String,
    },
    PositionFeed {
        id: Id,
        target: Id,
        before: bool,
        /// None keeps membership; a target folder moves only if not already a member.
        folder: Option<String>,
    },
    PositionRoot {
        item: SubscriptionNode,
        target: SubscriptionNode,
        before: bool,
    },
    PositionFeedOccurrence {
        id: Id,
        from_folder: String,
        to_folder: String,
        target: Option<SubscriptionNode>,
        before: bool,
    },
    /// None sorts the mixed root; Some sorts only the named folder's children.
    SortSubscriptions(Option<String>),
    SetSubscriptionSort(SubscriptionSort),
    SetFeedFolders {
        id: Id,
        folders: Vec<String>,
    },
    RemoveFeed(Id),
    /// Unsubscribe an exact set in one transaction. Missing/inactive identities
    /// reject the entire operation; repeated identities are removed only once.
    RemoveFeeds(Vec<Id>),
    CreateFolder(String),
    RenameFolder {
        from: String,
        to: String,
    },
    DeleteFolder(String),
    MarkRead {
        ids: Vec<Id>,
    },
    SetSettings(Settings),
    SetStreamSettings {
        stream: String,
        settings: Settings,
    },
    AddSubscription {
        title: String,
        url: String,
        site_url: String,
        folder: String,
    },
    MergeFeed {
        url: String,
        feed: ParsedFeed,
    },
    FeedError {
        url: String,
        message: String,
    },
}

#[derive(Clone, Debug, Default)]
pub struct Changes {
    pub changed: bool,
    pub articles: Vec<Id>,
    pub removed: Vec<Id>,
    pub feeds: bool,
    /// Sidebar projection only: no article-query, count or source-epoch work.
    pub order: bool,
    pub settings: bool,
    pub created_feed: Option<Id>,
}
