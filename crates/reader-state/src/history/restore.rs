use super::{aliases::Aliases, reconcile, UndoChange, UndoEntry};
use reader_domain::{Action, AppState, Changes, View};

impl UndoEntry {
    fn is_order_only(&self) -> bool {
        self.snapshot.is_none()
            && !self.changes.is_empty()
            && self.changes.iter().all(|change| {
                matches!(
                    change,
                    UndoChange::FeedOrder(_) | UndoChange::SubscriptionOrder(_)
                )
            })
    }

    pub(super) fn restore(&self, state: &mut AppState) -> Result<(Changes, Aliases), String> {
        if let Some(snapshot) = &self.snapshot {
            let mut restored = (**snapshot).clone();
            restored.normalize()?;
            let mut changes = difference(state, &restored);
            changes.changed = true;
            changes.feeds = true;
            changes.settings = true;
            *state = restored;
            return Ok((changes, Aliases::default()));
        }
        if self.is_order_only() {
            // Ordering changes only subscription metadata. Preserve the article
            // allocation, derived indexes and reactive snapshots in place.
            let previous = state.subscription_order.clone();
            let mut reordered = false;
            for change in &self.changes {
                match change {
                    UndoChange::SubscriptionOrder(order) => {
                        state.subscription_order = order.clone()
                    }
                    UndoChange::FeedOrder(order) => reordered |= state.restore_feed_order(order),
                    _ => unreachable!("only ordering history is restored here"),
                }
            }
            state.normalize_subscription_order();
            let order = reordered || previous != state.subscription_order;
            return Ok((
                Changes {
                    changed: order,
                    order,
                    ..Changes::default()
                },
                Aliases::default(),
            ));
        }
        if self.changes.iter().all(is_scalar) {
            // Read/star/note/title undo cannot change identities or references.
            // Keep these frequent edits narrow rather than copying every body.
            let mut changes = Changes::default();
            for change in &self.changes {
                restore_scalar(change, state, &mut changes);
            }
            changes.changed = !changes.articles.is_empty() || changes.feeds;
            return Ok((changes, Aliases::default()));
        }
        // Structural history can overlap later network results. Stage it until
        // URL/GUID uniqueness, all references and derived indexes validate.
        let mut restored = state.clone();
        let mut pending = Vec::new();
        let mut aliases = Aliases::default();
        let mut index = reconcile::Index::new(&restored);
        for change in &self.changes {
            if is_scalar(change) {
                restore_scalar(change, &mut restored, &mut Changes::default());
                continue;
            }
            match change {
                UndoChange::FeedFolders(id, folders) => {
                    if let Some(feed) = restored.feeds.iter_mut().find(|feed| feed.id == *id) {
                        feed.folder = folders.first().cloned().unwrap_or_default();
                        feed.folders = folders.clone();
                    }
                }
                UndoChange::FeedOrder(order) => {
                    restored.restore_feed_order(order);
                }
                UndoChange::SubscriptionOrder(order) => restored.subscription_order = order.clone(),
                UndoChange::Folders(folders) => restored.folders = folders.clone(),
                UndoChange::RestoreFeed(feed, position, articles) => {
                    reconcile::restore_feed(
                        &mut restored,
                        feed,
                        *position,
                        articles,
                        &mut pending,
                        &mut aliases,
                        &mut index,
                    )?;
                }
                UndoChange::RemoveFeed(id) => {
                    // Undoing an add/import also respects protected saved items.
                    restored.apply(Action::RemoveFeed(*id), 0)?;
                }
                UndoChange::Settings(settings) => restored.settings = settings.clone(),
                UndoChange::Stream(stream, display) => {
                    if let Some(display) = display {
                        restored.stream_settings.insert(stream.clone(), *display);
                    } else {
                        restored.stream_settings.remove(stream);
                    }
                }
                UndoChange::FolderRoute { .. } => {}
                _ => unreachable!("scalar history handled above"),
            }
        }
        // A bulk undo may interleave articles from several feeds. Insert the
        // missing rows in their original document order, not grouped by source.
        pending.sort_by_key(|(position, _)| *position);
        for (position, article) in pending {
            restored
                .articles
                .insert(position.min(restored.articles.len()), article);
        }
        aliases.settings(&mut restored.settings);
        aliases.subscription_order(&mut restored.subscription_order);
        restored.normalize()?;
        let changes = difference(state, &restored);
        *state = restored;
        Ok((changes, aliases))
    }

    fn restored_view(&self, current: View, state: &AppState) -> View {
        let folders = state.folders();
        let view = self
            .changes
            .iter()
            .find_map(|change| {
                if let UndoChange::FolderRoute { original, renamed } = change {
                    if current == View::Folder(renamed.clone()) && folders.contains(original) {
                        return Some(View::Folder(original.clone()));
                    }
                }
                None
            })
            .unwrap_or(current);
        match &view {
            View::Feed(id) if state.feed(*id).is_none_or(|feed| feed.archived) => View::All,
            View::Folder(name) if !folders.contains(name) => View::All,
            View::Tag(name) if !state.tags().contains(name) => View::All,
            _ => view,
        }
    }
}

fn is_scalar(change: &UndoChange) -> bool {
    matches!(
        change,
        UndoChange::Read(..)
            | UndoChange::Star(..)
            | UndoChange::Tags(..)
            | UndoChange::Note(..)
            | UndoChange::Title(..)
    )
}

fn restore_scalar(change: &UndoChange, state: &mut AppState, changed: &mut Changes) {
    match change {
        UndoChange::Read(id, read, at) => {
            if let Some(article) = state.articles.iter_mut().find(|article| article.id == *id) {
                article.read = *read;
                article.read_at = *at;
                changed.articles.push(*id);
            }
        }
        UndoChange::Star(id, starred) => {
            if let Some(article) = state.articles.iter_mut().find(|article| article.id == *id) {
                article.starred = *starred;
                changed.articles.push(*id);
            }
        }
        UndoChange::Tags(id, tags) => {
            if let Some(article) = state.articles.iter_mut().find(|article| article.id == *id) {
                article.tags = tags.clone();
                article.rebuild_search();
                changed.articles.push(*id);
            }
        }
        UndoChange::Note(id, note) => {
            if let Some(article) = state.articles.iter_mut().find(|article| article.id == *id) {
                article.note = note.clone();
                article.rebuild_search();
                changed.articles.push(*id);
            }
        }
        UndoChange::Title(id, title) => {
            if let Some(feed) = state.feeds.iter_mut().find(|feed| feed.id == *id) {
                feed.title = title.clone();
                changed.feeds = true;
            }
        }
        _ => unreachable!("only scalar history is restored in place"),
    }
}

fn difference(before: &AppState, after: &AppState) -> Changes {
    let articles = after
        .articles
        .iter()
        .filter(|article| before.article(article.id) != Some(article))
        .map(|article| article.id)
        .collect::<Vec<_>>();
    let removed = before
        .articles
        .iter()
        .filter(|article| after.article(article.id).is_none())
        .map(|article| article.id)
        .collect::<Vec<_>>();
    let feeds = before.feeds != after.feeds || before.folders != after.folders;
    let settings =
        before.settings != after.settings || before.stream_settings != after.stream_settings;
    let order = before.subscription_order != after.subscription_order;
    Changes {
        changed: feeds || settings || order || !articles.is_empty() || !removed.is_empty(),
        articles,
        removed,
        feeds,
        order,
        settings,
        created_feed: None,
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
        let order_only = entry.is_order_only();
        self.applying.set(true);
        let restored = entry.restore(&mut self.document.borrow_mut());
        match restored {
            Ok((mut changes, aliases)) => {
                if entry.snapshot.is_some() {
                    self.library_generation
                        .set(self.library_generation.get().wrapping_add(1));
                    self.reset_article_loads();
                    self.refreshing.set(false);
                    self.selected.set(None);
                    self.cursor.set(None);
                    self.kept_unread.borrow_mut().clear();
                }
                if !aliases.is_empty() {
                    for older in self.undo.borrow_mut().iter_mut() {
                        older.remap(&aliases);
                    }
                }
                if !order_only {
                    let route = entry.restored_view(self.view(), &self.state());
                    // Unlike navigate(), this keeps a still-valid reading cursor
                    // and selection through a folder rename and its inverse.
                    changes.settings |= self.view() != route;
                    *self.view.borrow_mut() = route.clone();
                    {
                        let state = self.state();
                        let belongs = |id| {
                            state.article(id).is_some_and(|article| {
                                reader_domain::matches_view(article, &state, &route)
                            })
                        };
                        if self.selected().is_some_and(|id| !belongs(id)) {
                            self.selected.set(None);
                        }
                        if self.cursor().is_some_and(|id| !belongs(id)) {
                            self.cursor.set(None);
                        }
                    }
                    self.restore_stream_settings();
                }
                if !order_only || changes.changed {
                    self.publish(&changes);
                }
                *self.status.borrow_mut() = "Change undone.".into();
                // Report persistence errors after success text, never hide them.
                if changes.changed {
                    self.schedule_save();
                }
            }
            Err(error) => {
                self.undo.borrow_mut().push_back(entry);
                *self.status.borrow_mut() =
                    format!("Could not undo this change: {error} Your library was kept unchanged.");
            }
        }
        self.applying.set(false);
        self.notify();
        self.apply(None);
    }
}
