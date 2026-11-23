use crate::{AppViewModel, HostRequest};
use reader_domain::{Action, Id, View};
use std::collections::BTreeSet;

impl AppViewModel {
    pub fn subscribe(&self, url: String) {
        let Some(url) = reader_domain::subscription_url(&url) else {
            self.report_error(
                "Enter an HTTP or HTTPS feed or website address without embedded credentials."
                    .into(),
            );
            return;
        };
        let existing = self
            .state()
            .feeds
            .iter()
            .find(|f| !f.archived && f.url == url)
            .map(|f| f.id);
        if let Some(id) = existing {
            self.navigate(View::Feed(id));
            self.set_status("You are already subscribed to this feed.".into());
            return;
        }
        self.refreshing.set(true);
        self.set_status("Subscribing…".into());
        let (proxy, connection) = {
            let state = self.state();
            (
                state.settings.feed_proxy.clone(),
                state.settings.feed_connection,
            )
        };
        self.request(HostRequest::Subscribe {
            url,
            proxy,
            connection,
        });
    }
    /// Start only subscriptions that have never completed a download. Hosts call
    /// this after startup and once connectivity returns; it neither adds feeds
    /// nor follows the current stream filter, so removed defaults stay removed.
    pub fn refresh_initial_feeds(&self) -> bool {
        let feeds = self
            .state()
            .feeds
            .iter()
            .filter(|feed| !feed.archived && feed.fetched_at == 0)
            .cloned()
            .collect();
        self.refresh_feeds(feeds)
    }
    pub fn refresh(&self) {
        self.refresh_view(self.view());
    }
    /// Refresh a context-menu target without changing the open reading stream.
    pub fn refresh_view(&self, view: View) -> bool {
        let state = self.state();
        let feeds = state
            .feeds
            .iter()
            .filter(|f| {
                !f.archived
                    && match &view {
                        View::Feed(id) => f.id == *id,
                        View::Folder(folder) => f.in_folder(folder),
                        _ => true,
                    }
            })
            .cloned()
            .collect();
        drop(state);
        self.refresh_feeds(feeds)
    }
    fn refresh_feeds(&self, feeds: Vec<reader_domain::Feed>) -> bool {
        if feeds.is_empty() || !self.has_services.get() || self.refreshing.replace(true) {
            return false;
        }
        let epochs: std::collections::BTreeMap<_, _> = feeds
            .iter()
            .filter_map(|feed| {
                self.subscription_epoch(feed.id)
                    .map(|epoch| (feed.id, epoch))
            })
            .collect();
        if epochs.len() != feeds.len() {
            self.refreshing.set(false);
            return false;
        }
        let state = self.state();
        let proxy = state.settings.feed_proxy.clone();
        let connection = state.settings.feed_connection;
        drop(state);
        self.set_status("Refreshing feeds…".into());
        self.request(HostRequest::Refresh {
            feeds,
            epochs,
            proxy,
            connection,
        });
        true
    }
    pub fn finish_refresh(&self) {
        self.refreshing.set(false);
        let status = self.status();
        if status.starts_with("Refreshing") || status.starts_with("Subscribing") {
            *self.status.borrow_mut() = "Refresh complete.".into();
        }
        self.notify();
    }
    pub fn receive_feed(&self, url: &str, source: &str) -> Result<Id, String> {
        self.receive_feed_from(url, source, url)
    }
    /// A redirect changes the document's link base, while the subscription retains its identity.
    pub fn receive_feed_from(
        &self,
        identity_url: &str,
        source: &str,
        base_url: &str,
    ) -> Result<Id, String> {
        let url = reader_domain::safe_url(identity_url, None).ok_or("Invalid feed URL.")?;
        let base = reader_domain::safe_url(base_url, None).ok_or("Invalid feed base URL.")?;
        let parsed = reader_domain::parse_feed(source, &base, self.now())?;
        self.dispatch(Action::MergeFeed {
            url: url.clone(),
            feed: parsed,
        });
        let id = self
            .state()
            .feeds
            .iter()
            .find(|f| f.url == url)
            .map(|f| f.id)
            .ok_or("The feed could not be added.")?;
        Ok(id)
    }
    pub fn feed_error(&self, url: &str, message: &str) {
        self.dispatch(Action::FeedError {
            url: url.into(),
            message: message.into(),
        });
        self.report_error(message.into());
    }
    pub fn rename_feed(&self, id: Id, title: String) {
        self.dispatch(Action::RenameFeed { id, title });
    }
    pub fn move_feed(&self, id: Id, folder: String) {
        self.dispatch(Action::MoveFeed { id, folder });
    }
    pub fn reorder_feed(&self, id: Id, target: Id, before: bool) {
        self.position_feed(id, target, before, None);
    }
    /// Compatibility API: an explicit folder has independent order. Call
    /// position_feed_occurrence when a drag captures its source folder as well.
    pub fn position_feed(&self, id: Id, target: Id, before: bool, folder: Option<String>) {
        self.dispatch(Action::PositionFeed {
            id,
            target,
            before,
            folder,
        });
    }
    pub fn set_feed_folders(&self, id: Id, folders: Vec<String>) {
        self.dispatch(Action::SetFeedFolders { id, folders });
    }
    pub fn unsubscribe(&self, id: Id) {
        self.unsubscribe_many(vec![id]);
    }
    /// Apply the exact confirmed selection as one undoable operation. A stale
    /// subscription rejects the whole set instead of partially unsubscribing.
    pub fn unsubscribe_many(&self, ids: Vec<Id>) -> bool {
        let ids: BTreeSet<_> = ids.into_iter().collect();
        if ids.is_empty() {
            return false;
        }
        if ids.iter().any(|id| self.feed(*id).is_none()) {
            self.report_error(
                "One or more subscriptions have changed. Review the selection and try again."
                    .into(),
            );
            return false;
        }
        let was_feed = matches!(self.view(), View::Feed(id) if ids.contains(&id));
        *self.status.borrow_mut() = if ids.len() == 1 {
            "Subscription removed. Saved items remain in your library. You can undo this change."
                .into()
        } else {
            format!("{} subscriptions removed. Saved items remain in your library. You can undo this change.", ids.len())
        };
        self.dispatch(Action::RemoveFeeds(ids.iter().copied().collect()));
        if was_feed {
            self.navigate(View::All);
        } else {
            let state = self.state();
            if self.selected().is_some_and(|id| {
                state.article(id).is_none_or(|article| {
                    !reader_domain::matches_view(article, &state, &self.view())
                })
            }) {
                self.selected.set(None);
            }
            if self.cursor().is_some_and(|id| {
                state.article(id).is_none_or(|article| {
                    !reader_domain::matches_view(article, &state, &self.view())
                })
            }) {
                self.cursor.set(None);
            }
        }
        // Keep any synchronous persistence error raised by dispatch visible.
        self.notify();
        true
    }
    pub fn validate_folder_name(
        &self,
        current: Option<&str>,
        input: &str,
    ) -> Result<String, String> {
        self.state().validate_folder_name(current, input)
    }
    pub fn create_folder(&self, name: String) -> bool {
        let name = match self.validate_folder_name(None, &name) {
            Ok(name) => name,
            Err(error) => {
                self.report_error(error);
                return false;
            }
        };
        self.dispatch(Action::CreateFolder(name));
        true
    }
    pub fn rename_folder(&self, from: String, to: String) -> bool {
        let to = match self.validate_folder_name(Some(&from), &to) {
            Ok(name) => name,
            Err(error) => {
                self.report_error(error);
                return false;
            }
        };
        if from == to {
            return true;
        }
        let selected = self.view() == View::Folder(from.clone());
        self.dispatch(Action::RenameFolder {
            from,
            to: to.clone(),
        });
        if selected && self.folders().contains(&to) {
            *self.view.borrow_mut() = View::Folder(to);
            self.restore_stream_settings();
            self.rebuild_query();
            self.notify();
        }
        true
    }
    pub fn delete_folder(&self, name: String) -> bool {
        if !self.folders().contains(&name) {
            self.report_error("This folder no longer exists.".into());
            return false;
        }
        let selected = self.view() == View::Folder(name.clone());
        *self.status.borrow_mut() = "Folder removed. Your subscriptions and saved items are unchanged. You can undo this change.".into();
        self.dispatch(Action::DeleteFolder(name));
        if selected {
            self.navigate(View::All);
        }
        true
    }
}
