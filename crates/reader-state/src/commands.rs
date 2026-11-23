use crate::{history::UndoEntry, AppViewModel};
use reader_domain::{Action, Changes, Density, Id, Settings, SortOrder, View, ViewMode};
use std::rc::Rc;

impl AppViewModel {
    pub(crate) fn apply(&self, action: Option<Action>) {
        if let Some(action) = action {
            self.pending.borrow_mut().push_back(action);
        }
        if self.applying.replace(true) {
            return;
        }
        loop {
            let action = self.pending.borrow_mut().pop_front();
            let Some(action) = action else {
                break;
            };
            let mut history = UndoEntry::capture(&self.document.borrow(), &action);
            let result = self.document.borrow_mut().apply(action, self.now());
            match result {
                Ok(changes) if changes.changed => {
                    if let Some(entry) = &mut history {
                        entry.finish(&changes);
                    }
                    if let Some(entry) = history {
                        let mut undo = self.undo.borrow_mut();
                        undo.push_back(entry);
                        if undo.len() > 30 {
                            undo.pop_front();
                        }
                    }
                    self.publish(&changes);
                    self.schedule_save();
                    self.notify();
                }
                Ok(_) => {}
                Err(error) => self.report_error(error),
            }
        }
        self.applying.set(false);
    }

    pub(crate) fn publish(&self, changes: &Changes) {
        if !changes.changed {
            return;
        }
        if changes.order
            && !changes.feeds
            && !changes.settings
            && changes.articles.is_empty()
            && changes.removed.is_empty()
        {
            return;
        }
        if changes.feeds {
            self.feed_epochs
                .borrow_mut()
                .reconcile(&self.state(), self.library_generation());
        }
        self.prune_article_loads();
        let updates = {
            let state = self.document.borrow();
            if self
                .selected()
                .is_some_and(|id| state.article(id).is_none())
            {
                self.selected.set(None);
            }
            if self.cursor().is_some_and(|id| state.article(id).is_none()) {
                self.cursor.set(None);
            }
            self.kept_unread
                .borrow_mut()
                .retain(|id| state.article(*id).is_some_and(|article| !article.read));
            if matches!(self.view(), View::Feed(id) if state.feed(id).is_none()) {
                *self.view.borrow_mut() = View::All;
            }
            changes
                .articles
                .iter()
                .filter_map(|&id| state.article(id).cloned().map(Rc::new))
                .collect::<Vec<_>>()
        };
        {
            let mut counts = self.counts.borrow_mut();
            for &id in &changes.removed {
                counts.remove(id);
            }
            for article in &updates {
                counts.update(article);
            }
            if changes.feeds {
                counts.reconcile_sources(&self.state());
            }
        }
        self.articles.edit(|cache| {
            for id in &changes.removed {
                cache.remove_key(id);
            }
            for article in updates {
                cache.add_or_update(article);
            }
        });
        if changes.feeds || changes.settings || changes.articles.len() > 64 {
            self.rebuild_query();
        } else {
            let mut changed = changes.articles.clone();
            changed.extend(&changes.removed);
            self.update_query(&changed);
        }
    }

    pub fn toggle_star(&self, id: Id) {
        self.dispatch(Action::ToggleStar(id));
    }
    pub fn toggle_read(&self, id: Id) {
        let read = self.state().article(id).map(|a| !a.read);
        if let Some(read) = read {
            self.set_read(id, read);
        }
    }
    pub fn set_read(&self, id: Id, read: bool) {
        if read {
            self.kept_unread.borrow_mut().remove(&id);
        } else {
            self.kept_unread.borrow_mut().insert(id);
        }
        self.dispatch(Action::SetRead { id, read });
    }
    pub fn set_tags(&self, id: Id, tags: Vec<String>) {
        self.dispatch(Action::SetTags { id, tags });
    }
    pub fn set_note(&self, id: Id, note: String) {
        self.dispatch(Action::SetNote { id, note });
    }
    /// Reader asked only for large streams. The count belongs to the current
    /// stream, including for an age-limited mark operation, not the global inbox.
    pub fn should_confirm_mark_all(&self) -> bool {
        self.state().settings.confirm_mark_all && self.count(&self.view()) > 50
    }
    pub fn mark_all_read(&self, older_than: Option<i64>) {
        let state = self.state();
        let ids = state
            .articles
            .iter()
            .filter(|a| {
                !a.read
                    && reader_domain::matches_view(a, &state, &self.view())
                    && older_than.is_none_or(|cutoff| a.published < cutoff)
            })
            .map(|a| a.id)
            .collect::<Vec<_>>();
        let count = ids.len();
        drop(state);
        self.dispatch(Action::MarkRead { ids });
        self.set_status(format!("Marked {count} items as read."));
    }
    pub fn set_settings(&self, settings: Settings) {
        self.dispatch(Action::SetStreamSettings {
            stream: self.view().storage_key(),
            settings,
        });
    }
    pub fn set_view_mode(&self, mode: ViewMode) {
        let mut settings = self.state().settings.clone();
        settings.view_mode = mode;
        self.set_settings(settings);
    }
    pub fn set_sort(&self, sort: SortOrder) {
        let mut settings = self.state().settings.clone();
        settings.sort = sort;
        self.set_settings(settings);
    }
    pub fn set_unread_only(&self, unread: bool) {
        let mut settings = self.state().settings.clone();
        settings.unread_only = unread;
        self.set_settings(settings);
    }
    pub fn set_density(&self, density: Density) {
        let mut settings = self.state().settings.clone();
        settings.density = density;
        self.set_settings(settings);
    }
}
