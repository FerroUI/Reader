use crate::AppViewModel;
use ferroui_dynamic_data::IObservableCache;
use reader_domain::{AppState, Article, Id, SortOrder};
use std::rc::Rc;

impl AppViewModel {
    fn accepts(&self, article: &Article, state: &AppState) -> bool {
        reader_domain::matches_view(article, state, &self.view())
            && (!state.settings.unread_only || !article.read || self.selected() == Some(article.id))
    }
    fn compare(state: &AppState, a: Id, b: Id) -> std::cmp::Ordering {
        let first = state
            .article(a)
            .map(|a| (a.published, a.id))
            .unwrap_or_default();
        let second = state
            .article(b)
            .map(|a| (a.published, a.id))
            .unwrap_or_default();
        match state.settings.sort {
            SortOrder::Newest => second.cmp(&first),
            SortOrder::Oldest => first.cmp(&second),
        }
    }
    pub(crate) fn rebuild_query(&self) {
        let (ids, updates) = {
            let state = self.state();
            let mut ids = state
                .articles
                .iter()
                .filter(|a| self.accepts(a, &state))
                .map(|a| a.id)
                .collect::<Vec<_>>();
            ids.sort_unstable_by(|a, b| Self::compare(&state, *a, *b));
            let updates = ids
                .iter()
                .filter_map(|id| self.articles.lookup(id))
                .collect::<Vec<_>>();
            (ids, updates)
        };
        let set = ids.iter().copied().collect();
        let old = self.visible_set.replace(set);
        let removed = old
            .difference(&self.visible_set.borrow())
            .copied()
            .collect::<Vec<_>>();
        *self.visible_ids.borrow_mut() = ids;
        self.visible.edit(|cache| {
            for id in removed {
                cache.remove_key(&id);
            }
            for article in updates {
                cache.add_or_update_with(article, &Rc::ptr_eq);
            }
        });
    }
    pub(crate) fn update_query(&self, changed: &[Id]) {
        let mut updates = Vec::new();
        let mut removed = Vec::new();
        {
            let state = self.state();
            let mut ids = self.visible_ids.borrow_mut();
            let mut members = self.visible_set.borrow_mut();
            for &id in changed {
                if members.remove(&id) {
                    ids.retain(|existing| *existing != id);
                }
                match state.article(id).filter(|a| self.accepts(a, &state)) {
                    Some(_) => {
                        let index = ids
                            .binary_search_by(|existing| Self::compare(&state, *existing, id))
                            .unwrap_or_else(|i| i);
                        ids.insert(index, id);
                        members.insert(id);
                        if let Some(article) = self.articles.lookup(&id) {
                            updates.push(article);
                        }
                    }
                    None => removed.push(id),
                }
            }
        }
        self.visible.edit(|cache| {
            for id in removed {
                cache.remove_key(&id);
            }
            for article in updates {
                cache.add_or_update_with(article, &Rc::ptr_eq);
            }
        });
    }
    pub fn visible_ids(&self) -> Vec<Id> {
        self.visible_ids.borrow().clone()
    }
    pub fn visible_len(&self) -> usize {
        self.visible_ids.borrow().len()
    }
    pub fn visible_page(&self, offset: usize, count: usize) -> Vec<Article> {
        let state = self.state();
        self.visible_ids
            .borrow()
            .iter()
            .skip(offset)
            .take(count)
            .filter_map(|&id| state.article(id).cloned())
            .collect()
    }
    pub fn visible_articles(&self) -> Vec<Article> {
        self.visible_page(0, self.visible_len())
    }
    /// Immutable snapshots shared with DynamicData and recycled view rows.
    pub fn article_shared(&self, id: Id) -> Option<Rc<Article>> {
        self.articles.lookup(&id)
    }
    pub fn visible_shared(&self) -> Vec<Rc<Article>> {
        self.visible_ids
            .borrow()
            .iter()
            .filter_map(|id| self.articles.lookup(id))
            .collect()
    }
}
