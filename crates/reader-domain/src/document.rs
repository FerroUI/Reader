use crate::{AppState, Article, Feed, Id};
use std::collections::BTreeSet;

impl AppState {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn feed(&self, id: Id) -> Option<&Feed> {
        self.feed_index.get(&id).and_then(|&i| self.feeds.get(i))
    }
    pub fn article(&self, id: Id) -> Option<&Article> {
        self.article_index
            .get(&id)
            .and_then(|&i| self.articles.get(i))
    }
    pub(crate) fn article_mut(&mut self, id: Id) -> Option<&mut Article> {
        self.article_index
            .get(&id)
            .copied()
            .and_then(|i| self.articles.get_mut(i))
    }
    pub(crate) fn feed_mut(&mut self, id: Id) -> Option<&mut Feed> {
        self.feed_index
            .get(&id)
            .copied()
            .and_then(|i| self.feeds.get_mut(i))
    }
    pub fn folders(&self) -> Vec<String> {
        let mut result = self.folders.iter().cloned().collect::<BTreeSet<_>>();
        result.extend(self.feeds.iter().flat_map(Feed::folder_names));
        result.into_iter().collect()
    }
    pub fn tags(&self) -> Vec<String> {
        self.articles
            .iter()
            .flat_map(|a| a.tags.iter().cloned())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }
    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string(self).map_err(|e| e.to_string())
    }
    pub fn from_json(json: &str) -> Result<Self, String> {
        if json.len() > 64 * 1024 * 1024 {
            return Err("Reader backup exceeds the 64 MB limit.".into());
        }
        let value: serde_json::Value =
            serde_json::from_str(json).map_err(|e| format!("Invalid Reader backup: {e}"))?;
        if !value["feeds"].is_array() || !value["articles"].is_array() {
            return Err("This is not a Reader backup. Use OPML import for subscriptions.".into());
        }
        let mut state: Self = serde_json::from_value(value).map_err(|e| e.to_string())?;
        state.normalize()?;
        Ok(state)
    }
    /// Validate references and restore all derived indexes before accepting imported data.
    pub fn normalize(&mut self) -> Result<(), String> {
        if self.version != 1 {
            return Err(format!(
                "Reader backup version {} is not supported.",
                self.version
            ));
        }
        let mut used = BTreeSet::new();
        let mut urls = BTreeSet::new();
        for feed in &mut self.feeds {
            if feed.id == 0 || !used.insert(feed.id) {
                return Err("Duplicate or invalid feed identifiers.".into());
            }
            feed.url = crate::safe_url(&feed.url, None)
                .ok_or("A subscription contains an invalid URL.")?;
            if !urls.insert(feed.url.clone()) {
                return Err("The backup contains duplicate subscriptions.".into());
            }
            feed.site_url = crate::safe_url(&feed.site_url, None).unwrap_or_default();
            feed.folder = feed.folder.trim().to_string();
            feed.folders = feed.folder_names();
            if feed.folder.is_empty() {
                feed.folder = feed.folders.first().cloned().unwrap_or_default();
            }
        }
        let feeds = used.clone();
        let mut guids = BTreeSet::new();
        for article in &mut self.articles {
            if article.id == 0 || !used.insert(article.id) {
                return Err("Duplicate or invalid article identifiers.".into());
            }
            if !feeds.contains(&article.feed_id) {
                return Err("An article refers to a missing subscription.".into());
            }
            if !guids.insert((article.feed_id, article.guid.clone())) {
                return Err("The backup contains duplicate article GUIDs.".into());
            }
            article.url = crate::safe_url(&article.url, None).unwrap_or_default();
            article.content_base = crate::safe_url(&article.content_base, None).unwrap_or_default();
            article.tags = clean_names(std::mem::take(&mut article.tags));
            article.rebuild_search();
        }
        self.folders = clean_names(std::mem::take(&mut self.folders));
        self.settings.font_size = if self.settings.font_size.is_finite() {
            self.settings.font_size.clamp(11.0, 26.0)
        } else {
            13.0
        };
        self.next_id = self.next_id.max(
            used.last()
                .copied()
                .unwrap_or(0)
                .checked_add(1)
                .ok_or("Reader identifiers are exhausted.")?,
        );
        if self.next_id == u64::MAX {
            return Err("Reader identifiers are exhausted.".into());
        }
        self.reindex();
        Ok(())
    }
    pub(crate) fn reindex(&mut self) {
        self.feed_index = self
            .feeds
            .iter()
            .enumerate()
            .map(|(i, f)| (f.id, i))
            .collect();
        self.article_index = self
            .articles
            .iter()
            .enumerate()
            .map(|(i, a)| (a.id, i))
            .collect();
    }
    pub(crate) fn allocate(&mut self) -> Id {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
}

pub(crate) fn clean_names(names: Vec<String>) -> Vec<String> {
    names
        .into_iter()
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}
