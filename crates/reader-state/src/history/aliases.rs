//! Keep older undo commands attached to their logical feed/article when a
//! removed source was subscribed again with newly allocated identities.
use super::{UndoChange, UndoEntry};
use reader_domain::{Id, Settings, SubscriptionNode, SubscriptionOrder, View};
use std::collections::BTreeMap;

#[derive(Default)]
pub(super) struct Aliases {
    pub feeds: BTreeMap<Id, Id>,
    pub articles: BTreeMap<Id, Id>,
}

impl Aliases {
    fn feed(&self, id: &mut Id) {
        if let Some(next) = self.feeds.get(id) {
            *id = *next;
        }
    }
    fn article(&self, id: &mut Id) {
        if let Some(next) = self.articles.get(id) {
            *id = *next;
        }
    }
    pub fn settings(&self, settings: &mut Settings) {
        if let View::Feed(id) = &mut settings.start_page {
            self.feed(id);
        }
    }
    pub fn subscription_order(&self, order: &mut SubscriptionOrder) {
        for node in &mut order.root {
            if let SubscriptionNode::Feed(id) = node {
                self.feed(id);
            }
        }
        for ids in order.folders.values_mut() {
            for id in ids {
                self.feed(id);
            }
        }
        // Membership validation and duplicate removal belong to restoration:
        // earlier entries may refer to folders that a later undo recreates.
    }
    fn stream(&self, key: &mut String) {
        if let Some(mut id) = key.strip_prefix("feed:").and_then(|id| id.parse().ok()) {
            self.feed(&mut id);
            *key = View::Feed(id).storage_key();
        }
    }
    pub fn is_empty(&self) -> bool {
        self.feeds.is_empty() && self.articles.is_empty()
    }
}

impl UndoEntry {
    pub(super) fn remap(&mut self, aliases: &Aliases) {
        // A library-replacement snapshot is a separate, self-contained identity
        // space. Its own feeds, article references and settings stay together.
        for change in &mut self.changes {
            match change {
                UndoChange::Read(id, ..)
                | UndoChange::Star(id, ..)
                | UndoChange::Tags(id, ..)
                | UndoChange::Note(id, ..) => aliases.article(id),
                UndoChange::Title(id, ..)
                | UndoChange::FeedFolders(id, ..)
                | UndoChange::RemoveFeed(id) => aliases.feed(id),
                UndoChange::FeedOrder(order) => {
                    for id in order {
                        aliases.feed(id);
                    }
                }
                UndoChange::SubscriptionOrder(order) => aliases.subscription_order(order),
                UndoChange::RestoreFeed(feed, _, articles) => {
                    aliases.feed(&mut feed.id);
                    for (_, article) in articles {
                        aliases.feed(&mut article.feed_id);
                        aliases.article(&mut article.id);
                    }
                }
                UndoChange::Settings(settings) => aliases.settings(settings),
                UndoChange::Stream(key, _) => aliases.stream(key),
                UndoChange::Folders(_) | UndoChange::FolderRoute { .. } => {}
            }
        }
    }
}
