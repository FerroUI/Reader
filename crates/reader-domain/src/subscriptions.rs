use crate::{AppState, Changes, Id, View};
use std::collections::BTreeSet;

impl AppState {
    /// Validate the entire captured selection before changing any subscription.
    /// Source metadata for annotated articles stays in the portable document,
    /// while ordinary cached articles and active folder memberships are removed.
    pub(crate) fn remove_feeds(&mut self, ids: Vec<Id>) -> Result<Changes, String> {
        let ids: BTreeSet<_> = ids.into_iter().collect();
        if ids
            .iter()
            .any(|&id| self.feed(id).is_none_or(|feed| feed.archived))
        {
            return Err(
                "One or more subscriptions have changed. Review the selection and try again."
                    .into(),
            );
        }
        if ids.is_empty() {
            return Ok(Changes::default());
        }

        let mut saved_sources = BTreeSet::new();
        let mut changes = Changes {
            changed: true,
            feeds: true,
            ..Changes::default()
        };
        self.articles.retain(|article| {
            if !ids.contains(&article.feed_id) {
                return true;
            }
            if article.has_user_annotations() {
                saved_sources.insert(article.feed_id);
                true
            } else {
                changes.removed.push(article.id);
                false
            }
        });
        self.feeds.retain_mut(|feed| {
            if !ids.contains(&feed.id) {
                return true;
            }
            if !saved_sources.contains(&feed.id) {
                return false;
            }
            feed.archived = true;
            feed.folder.clear();
            feed.folders.clear();
            feed.error = None;
            true
        });
        if matches!(self.settings.start_page, View::Feed(id) if ids.contains(&id)) {
            self.settings.start_page = View::Home;
            changes.settings = true;
        }
        self.reindex();
        Ok(changes)
    }
}
