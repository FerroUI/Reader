use crate::{AppViewModel, HostRequest};
use reader_domain::{Action, Id, View};

impl AppViewModel {
    pub fn navigate(&self, view: View) {
        *self.view.borrow_mut() = view;
        self.restore_stream_settings();
        self.selected.set(None);
        self.cursor.set(None);
        self.sidebar.set(false);
        self.rebuild_query();
        self.notify();
    }
    pub fn select(&self, id: Option<Id>) {
        let id = id.filter(|&id| self.state().article(id).is_some());
        let previous = self.selected.replace(id);
        if let Some(id) = id {
            self.cursor.set(Some(id));
            self.kept_unread.borrow_mut().remove(&id);
        }
        let mut changed = previous.into_iter().collect::<Vec<_>>();
        changed.extend(id);
        self.update_query(&changed);
        if let Some(id) = id {
            self.dispatch(Action::SetRead { id, read: true });
        }
        self.notify();
    }
    pub(crate) fn restore_stream_settings(&self) {
        let mut state = self.document.borrow_mut();
        let display = state
            .stream_settings
            .get(&self.view().storage_key())
            .copied()
            .unwrap_or_default();
        display.apply_to(&mut state.settings);
    }
    pub fn next_article(&self, delta: i32) {
        let ids = self.visible_ids.borrow();
        if ids.is_empty() {
            return;
        }
        let current = self
            .cursor()
            .and_then(|id| ids.iter().position(|candidate| *candidate == id));
        let index = match current {
            Some(index) => (index as i64 + delta as i64).clamp(0, ids.len() as i64 - 1) as usize,
            None => {
                if delta < 0 {
                    ids.len() - 1
                } else {
                    0
                }
            }
        };
        let id = ids[index];
        drop(ids);
        self.select(Some(id));
        self.request(HostRequest::ScrollToArticle { id });
    }
    /// Reader's n/p cursor moves independently of opening (j/k) and read state.
    pub fn scan_article(&self, delta: i32) {
        let ids = self.visible_ids.borrow();
        if ids.is_empty() {
            return;
        }
        let current = self
            .cursor()
            .and_then(|id| ids.iter().position(|candidate| *candidate == id));
        let index = current
            .map(|i| (i as i64 + delta as i64).clamp(0, ids.len() as i64 - 1) as usize)
            .unwrap_or_else(|| if delta < 0 { ids.len() - 1 } else { 0 });
        let id = ids[index];
        drop(ids);
        self.cursor.set(Some(id));
        self.notify();
        self.request(HostRequest::ScrollToArticle { id });
    }
    pub fn keep_unread(&self, id: Id) {
        self.kept_unread.borrow_mut().insert(id);
        self.dispatch(Action::SetRead { id, read: false });
    }
    pub fn is_kept_unread(&self, id: Id) -> bool {
        self.kept_unread.borrow().contains(&id)
    }
    pub fn select_from_scroll(&self, id: Id) {
        let needs_update = self.selected() != Some(id)
            || self
                .state()
                .article(id)
                .is_some_and(|article| !article.read);
        if needs_update && self.state().settings.mark_read_on_scroll && !self.is_kept_unread(id) {
            self.select(Some(id));
        }
    }
    pub fn set_viewport(&self, width: f64) {
        self.set_viewport_size(width, self.viewport_height());
    }
    pub fn set_touch_mode(&self, value: bool) {
        if self.touch_mode.replace(value) != value {
            self.notify();
        }
    }
    pub fn set_viewport_size(&self, width: f64, height: f64) {
        let width_changed =
            width.is_finite() && width > 0.0 && (self.viewport.get() - width).abs() > 0.5;
        let height_changed =
            height.is_finite() && height > 0.0 && (self.viewport_height.get() - height).abs() > 0.5;
        if width_changed {
            self.viewport.set(width);
        }
        if height_changed {
            self.viewport_height.set(height);
        }
        if width_changed || height_changed {
            self.notify();
        }
    }
    pub fn toggle_sidebar(&self) {
        self.sidebar.set(!self.sidebar.get());
        self.notify();
    }
    pub fn set_sidebar_open(&self, open: bool) {
        if self.sidebar.replace(open) != open {
            self.notify();
        }
    }
}
