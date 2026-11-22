use crate::{AppViewModel, HostRequest};
use reader_domain::{Action, Id, View};

impl AppViewModel {
    pub fn subscribe(&self, url: String) {
        let Some(url) = reader_domain::safe_url(&url, None) else {
            self.report_error("Enter a full http or https RSS or Atom feed URL.".into());
            return;
        };
        let existing = self
            .state()
            .feeds
            .iter()
            .find(|f| f.url == url)
            .map(|f| f.id);
        if let Some(id) = existing {
            self.navigate(View::Feed(id));
            self.set_status("You are already subscribed to this feed.".into());
            return;
        }
        self.refreshing.set(true);
        self.set_status("Subscribing…".into());
        let proxy = self.state().settings.feed_proxy.clone();
        self.request(HostRequest::Subscribe { url, proxy });
    }
    pub fn refresh(&self) {
        if self.refreshing.replace(true) {
            return;
        }
        let state = self.state();
        let feeds = state
            .feeds
            .iter()
            .filter(|f| match self.view() {
                View::Feed(id) => f.id == id,
                View::Folder(ref folder) => f.in_folder(folder),
                _ => true,
            })
            .cloned()
            .collect();
        let proxy = state.settings.feed_proxy.clone();
        drop(state);
        self.set_status("Refreshing feeds…".into());
        self.request(HostRequest::Refresh { feeds, proxy });
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
    /// Position in the shared subscription order, optionally moving to the
    /// target sidebar group. Reordering within an existing label retains others.
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
        self.dispatch(Action::RemoveFeed(id));
        self.set_status("Subscription removed. You can undo this change.".into());
    }
    pub fn create_folder(&self, name: String) {
        self.dispatch(Action::CreateFolder(name));
    }
    pub fn rename_folder(&self, from: String, to: String) {
        let selected = self.view() == View::Folder(from.clone());
        self.dispatch(Action::RenameFolder {
            from,
            to: to.clone(),
        });
        if selected && self.folders().contains(&to) {
            self.navigate(View::Folder(to));
        }
    }
    pub fn delete_folder(&self, name: String) {
        let selected = self.view() == View::Folder(name.clone());
        self.dispatch(Action::DeleteFolder(name));
        if selected {
            self.navigate(View::All);
        }
    }
}
