use reader_domain::{Action, AppState, Article, Changes, Feed, Id, Settings};

pub(crate) enum UndoChange {
    Read(Id, bool, Option<i64>),
    Star(Id, bool),
    Tags(Id, Vec<String>),
    Note(Id, String),
    Title(Id, String),
    FeedFolders(Id, Vec<String>),
    Folders(Vec<String>),
    RestoreFeed(Feed, usize, Vec<(usize, Article)>),
    RemoveFeed(Id),
    Settings(Settings),
    Stream(String, Option<reader_domain::StreamSettings>),
}
pub(crate) struct UndoEntry {
    changes: Vec<UndoChange>,
    snapshot: Option<Box<AppState>>,
}

impl UndoEntry {
    pub fn capture(state: &AppState, action: &Action) -> Option<Self> {
        let mut changes = Vec::new();
        match action {
            Action::SetRead { id, .. } => {
                if let Some(a) = state.article(*id) {
                    changes.push(UndoChange::Read(*id, a.read, a.read_at));
                }
            }
            Action::MarkRead { ids } => {
                for id in ids {
                    if let Some(a) = state.article(*id) {
                        if !a.read {
                            changes.push(UndoChange::Read(*id, a.read, a.read_at));
                        }
                    }
                }
            }
            Action::ToggleStar(id) => {
                if let Some(a) = state.article(*id) {
                    changes.push(UndoChange::Star(*id, a.starred));
                }
            }
            Action::SetTags { id, .. } => {
                if let Some(a) = state.article(*id) {
                    changes.push(UndoChange::Tags(*id, a.tags.clone()));
                }
            }
            Action::SetNote { id, .. } => {
                if let Some(a) = state.article(*id) {
                    changes.push(UndoChange::Note(*id, a.note.clone()));
                }
            }
            Action::RenameFeed { id, .. } => {
                if let Some(f) = state.feed(*id) {
                    changes.push(UndoChange::Title(*id, f.title.clone()));
                }
            }
            Action::MoveFeed { id, .. } | Action::SetFeedFolders { id, .. } => {
                if let Some(f) = state.feed(*id) {
                    changes.push(UndoChange::FeedFolders(*id, f.folder_names()));
                }
                changes.push(UndoChange::Folders(state.folders.clone()));
            }
            Action::RemoveFeed(id) => {
                if let Some(f) = state.feed(*id) {
                    changes.push(UndoChange::RestoreFeed(
                        f.clone(),
                        state.feeds.iter().position(|f| f.id == *id).unwrap(),
                        state
                            .articles
                            .iter()
                            .enumerate()
                            .filter(|(_, a)| a.feed_id == *id)
                            .map(|(i, a)| (i, a.clone()))
                            .collect(),
                    ));
                }
            }
            Action::CreateFolder(_) => changes.push(UndoChange::Folders(state.folders.clone())),
            Action::RenameFolder { from, .. } | Action::DeleteFolder(from) => {
                changes.push(UndoChange::Folders(state.folders.clone()));
                for f in state.feeds.iter().filter(|f| f.in_folder(from)) {
                    changes.push(UndoChange::FeedFolders(f.id, f.folder_names()));
                }
                if let Action::RenameFolder { to, .. } = action {
                    for name in [from, to] {
                        let key = format!("folder:{name}");
                        changes.push(UndoChange::Stream(
                            key.clone(),
                            state.stream_settings.get(&key).copied(),
                        ));
                    }
                }
            }
            Action::SetSettings(_) => changes.push(UndoChange::Settings(state.settings.clone())),
            Action::SetStreamSettings { stream, .. } => {
                changes.push(UndoChange::Settings(state.settings.clone()));
                changes.push(UndoChange::Stream(
                    stream.clone(),
                    state.stream_settings.get(stream).copied(),
                ));
            }
            Action::AddSubscription { .. } => {}
            Action::MergeFeed { .. } | Action::FeedError { .. } => return None,
        }
        Some(Self {
            changes,
            snapshot: None,
        })
    }
    pub fn snapshot(state: AppState) -> Self {
        Self {
            changes: Vec::new(),
            snapshot: Some(Box::new(state)),
        }
    }
    pub fn subscriptions(
        ids: Vec<Id>,
        folders: Vec<String>,
        memberships: Vec<(Id, Vec<String>)>,
    ) -> Self {
        let mut changes = ids
            .into_iter()
            .map(UndoChange::RemoveFeed)
            .collect::<Vec<_>>();
        changes.push(UndoChange::Folders(folders));
        changes.extend(
            memberships
                .into_iter()
                .map(|(id, folders)| UndoChange::FeedFolders(id, folders)),
        );
        Self {
            changes,
            snapshot: None,
        }
    }
    pub fn finish(&mut self, changes: &Changes) {
        if let Some(id) = changes.created_feed {
            self.changes.push(UndoChange::RemoveFeed(id));
        }
    }
    pub fn restore(self, state: &mut AppState) -> Changes {
        if let Some(snapshot) = self.snapshot {
            let removed = state
                .articles
                .iter()
                .filter(|a| snapshot.article(a.id).is_none())
                .map(|a| a.id)
                .collect();
            let articles = snapshot.articles.iter().map(|a| a.id).collect();
            *state = *snapshot;
            return Changes {
                changed: true,
                articles,
                removed,
                feeds: true,
                settings: true,
                created_feed: None,
            };
        }
        let mut changed = Changes {
            changed: true,
            ..Changes::default()
        };
        for change in self.changes {
            match change {
                UndoChange::Read(id, read, at) => {
                    if let Some(a) = state.articles.iter_mut().find(|a| a.id == id) {
                        a.read = read;
                        a.read_at = at;
                        changed.articles.push(id);
                    }
                }
                UndoChange::Star(id, starred) => {
                    if let Some(a) = state.articles.iter_mut().find(|a| a.id == id) {
                        a.starred = starred;
                        changed.articles.push(id);
                    }
                }
                UndoChange::Tags(id, tags) => {
                    if let Some(a) = state.articles.iter_mut().find(|a| a.id == id) {
                        a.tags = tags;
                        a.rebuild_search();
                        changed.articles.push(id);
                    }
                }
                UndoChange::Note(id, note) => {
                    if let Some(a) = state.articles.iter_mut().find(|a| a.id == id) {
                        a.note = note;
                        a.rebuild_search();
                        changed.articles.push(id);
                    }
                }
                UndoChange::Title(id, title) => {
                    if let Some(f) = state.feeds.iter_mut().find(|f| f.id == id) {
                        f.title = title;
                        changed.feeds = true;
                    }
                }
                UndoChange::FeedFolders(id, folders) => {
                    if let Some(f) = state.feeds.iter_mut().find(|f| f.id == id) {
                        f.folder = folders.first().cloned().unwrap_or_default();
                        f.folders = folders;
                        changed.feeds = true;
                    }
                }
                UndoChange::Folders(folders) => {
                    state.folders = folders;
                    changed.feeds = true;
                }
                UndoChange::RestoreFeed(feed, position, articles) => {
                    state.feeds.insert(position.min(state.feeds.len()), feed);
                    changed.articles.extend(articles.iter().map(|(_, a)| a.id));
                    for (position, article) in articles {
                        state
                            .articles
                            .insert(position.min(state.articles.len()), article);
                    }
                    changed.feeds = true;
                }
                UndoChange::RemoveFeed(id) => {
                    state.feeds.retain(|f| f.id != id);
                    state.articles.retain(|a| {
                        if a.feed_id == id {
                            changed.removed.push(a.id);
                            false
                        } else {
                            true
                        }
                    });
                    changed.feeds = true;
                }
                UndoChange::Settings(settings) => {
                    state.settings = settings;
                    changed.settings = true;
                }
                UndoChange::Stream(stream, display) => {
                    if let Some(display) = display {
                        state.stream_settings.insert(stream, display);
                    } else {
                        state.stream_settings.remove(&stream);
                    }
                    changed.settings = true;
                }
            }
        }
        // Restoring existing, previously validated data cannot introduce bad references.
        let _ = state.normalize();
        changed
    }
}

impl crate::AppViewModel {
    pub fn can_undo(&self) -> bool {
        !self.undo.borrow().is_empty()
    }
    pub fn undo(&self) {
        if self.applying.get() {
            return;
        }
        let entry = self.undo.borrow_mut().pop_back();
        let Some(entry) = entry else {
            return;
        };
        self.applying.set(true);
        let changes = entry.restore(&mut self.document.borrow_mut());
        self.restore_stream_settings();
        self.publish(&changes);
        self.schedule_save();
        *self.status.borrow_mut() = "Change undone.".into();
        self.applying.set(false);
        self.notify();
        self.apply(None);
    }
}
