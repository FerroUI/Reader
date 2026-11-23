//! Transient subscription lifetimes fence responses started before unsubscribe.
//! Reusing a source ID for its saved articles must not reuse its network lifetime.
use reader_domain::{AppState, Id};
use std::collections::BTreeMap;

pub(crate) struct FeedEpochs {
    active: BTreeMap<Id, (String, u64)>,
    next: u64,
    library_generation: u64,
}

impl FeedEpochs {
    pub fn new(state: &AppState, library_generation: u64) -> Self {
        let mut result = Self {
            active: BTreeMap::new(),
            next: 1,
            library_generation,
        };
        result.reconcile(state, library_generation);
        result
    }

    pub fn reconcile(&mut self, state: &AppState, library_generation: u64) {
        if self.library_generation != library_generation {
            self.active.clear();
            self.library_generation = library_generation;
        }
        self.active.retain(|id, (url, _)| {
            state
                .feed(*id)
                .is_some_and(|feed| !feed.archived && feed.url == *url)
        });
        for feed in state.feeds.iter().filter(|feed| !feed.archived) {
            if self.active.contains_key(&feed.id) {
                continue;
            }
            // Never recycle a lifetime token, even in an exhausted counter.
            // A missing token makes the host reject that response safely.
            let Some(next) = self.next.checked_add(1) else {
                continue;
            };
            self.active.insert(feed.id, (feed.url.clone(), self.next));
            self.next = next;
        }
    }

    pub fn get(&self, id: Id) -> Option<u64> {
        self.active.get(&id).map(|(_, epoch)| *epoch)
    }

    pub fn accepts(&self, id: Id, url: &str, epoch: u64) -> bool {
        self.active
            .get(&id)
            .is_some_and(|(active_url, active_epoch)| *active_epoch == epoch && active_url == url)
    }
}

impl crate::AppViewModel {
    pub fn subscription_epoch(&self, id: Id) -> Option<u64> {
        self.feed_epochs.borrow().get(id)
    }

    pub fn accepts_feed_result(&self, id: Id, url: &str, epoch: u64) -> bool {
        self.feed_epochs.borrow().accepts(id, url, epoch)
    }
}
