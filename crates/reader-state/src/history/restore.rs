use super::{UndoChange, UndoEntry};
use reader_domain::{AppState, Changes};

impl UndoEntry {
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
                UndoChange::FeedOrder(order) => {
                    let ranks: std::collections::HashMap<_, _> = order
                        .into_iter()
                        .enumerate()
                        .map(|(index, id)| (id, index))
                        .collect();
                    state
                        .feeds
                        .sort_by_key(|feed| ranks.get(&feed.id).copied().unwrap_or(usize::MAX));
                    changed.feeds = true;
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
        if entry.snapshot.is_some() {
            self.library_generation
                .set(self.library_generation.get().wrapping_add(1));
            self.refreshing.set(false);
            self.selected.set(None);
            self.cursor.set(None);
            self.kept_unread.borrow_mut().clear();
        }
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
