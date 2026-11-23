use reader_domain::{AppState, Article, Id, View};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Default)]
struct Count {
    total: usize,
    unread: usize,
}
impl Count {
    fn update(&mut self, read: bool, add: bool) {
        if add {
            self.total += 1;
            self.unread += usize::from(!read);
        } else {
            self.total = self.total.saturating_sub(1);
            self.unread = self.unread.saturating_sub(usize::from(!read));
        }
    }
    fn get(&self, unread: bool) -> usize {
        if unread {
            self.unread
        } else {
            self.total
        }
    }
}

struct Entry {
    feed: Id,
    read: bool,
    starred: bool,
    tags: Vec<String>,
}

#[derive(Default)]
pub(crate) struct CountIndex {
    all: Count,
    archived: Count,
    archived_sources: BTreeSet<Id>,
    starred: Count,
    feeds: BTreeMap<Id, Count>,
    tags: BTreeMap<String, Count>,
    entries: BTreeMap<Id, Entry>,
}

impl CountIndex {
    pub fn new(state: &AppState) -> Self {
        let mut index = Self {
            archived_sources: state
                .feeds
                .iter()
                .filter(|feed| feed.archived)
                .map(|feed| feed.id)
                .collect(),
            ..Self::default()
        };
        for article in &state.articles {
            index.update(article);
        }
        index
    }
    /// Subscription transitions are infrequent. Reconcile their aggregate once
    /// so ordinary read/star updates keep All/Home counts independent of size.
    pub fn reconcile_sources(&mut self, state: &AppState) {
        self.archived_sources = state
            .feeds
            .iter()
            .filter(|feed| feed.archived)
            .map(|feed| feed.id)
            .collect();
        self.archived = self
            .archived_sources
            .iter()
            .filter_map(|id| self.feeds.get(id))
            .fold(Count::default(), |mut total, count| {
                total.total += count.total;
                total.unread += count.unread;
                total
            });
    }
    pub fn remove(&mut self, id: Id) {
        if let Some(entry) = self.entries.remove(&id) {
            self.adjust(&entry, false);
        }
    }
    pub fn update(&mut self, article: &Article) {
        self.remove(article.id);
        let entry = Entry {
            feed: article.feed_id,
            read: article.read,
            starred: article.starred,
            tags: article.tags.clone(),
        };
        self.adjust(&entry, true);
        self.entries.insert(article.id, entry);
    }
    fn adjust(&mut self, entry: &Entry, add: bool) {
        self.all.update(entry.read, add);
        if self.archived_sources.contains(&entry.feed) {
            self.archived.update(entry.read, add);
        }
        if entry.starred {
            self.starred.update(entry.read, add);
        }
        self.feeds
            .entry(entry.feed)
            .or_default()
            .update(entry.read, add);
        for tag in &entry.tags {
            self.tags
                .entry(tag.clone())
                .or_default()
                .update(entry.read, add);
        }
    }
    pub fn count(&self, view: &View, state: &AppState, unread: bool) -> usize {
        match view {
            View::All | View::Home | View::Trends => self
                .all
                .get(unread)
                .saturating_sub(self.archived.get(unread)),
            View::Starred => self.starred.get(unread),
            View::Feed(id) => self.feeds.get(id).copied().unwrap_or_default().get(unread),
            View::Folder(folder) => state
                .feeds
                .iter()
                .filter(|f| !f.archived && f.in_folder(folder))
                .map(|f| {
                    self.feeds
                        .get(&f.id)
                        .copied()
                        .unwrap_or_default()
                        .get(unread)
                })
                .sum(),
            View::Tag(tag) => self.tags.get(tag).copied().unwrap_or_default().get(unread),
            View::Search(_) => state
                .articles
                .iter()
                .filter(|a| (!unread || !a.read) && reader_domain::matches_view(a, state, view))
                .count(),
            View::Explore | View::Settings => 0,
        }
    }
}
