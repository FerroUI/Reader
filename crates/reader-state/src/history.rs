mod restore;
use reader_domain::{Action, AppState, Article, Changes, Feed, Id, Settings};

pub(crate) enum UndoChange {
    Read(Id, bool, Option<i64>),
    Star(Id, bool),
    Tags(Id, Vec<String>),
    Note(Id, String),
    Title(Id, String),
    FeedFolders(Id, Vec<String>),
    FeedOrder(Vec<Id>),
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
            Action::PositionFeed { id, folder, .. } => {
                changes.push(UndoChange::FeedOrder(
                    state.feeds.iter().map(|feed| feed.id).collect(),
                ));
                if folder.is_some() {
                    if let Some(feed) = state.feed(*id) {
                        changes.push(UndoChange::FeedFolders(*id, feed.folder_names()));
                    }
                    changes.push(UndoChange::Folders(state.folders.clone()));
                }
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
            Action::MergeFeed { .. }
            | Action::FeedError { .. }
            | Action::CacheFullArticle { .. }
            | Action::ShowFullArticle { .. } => return None,
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
}
