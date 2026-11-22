use crate::{AppViewModel, HostRequest};
use reader_domain::Id;
use std::{
    cell::{Cell, RefCell},
    collections::BTreeMap,
};

pub(super) struct Request {
    pub request_id: u64,
    pub generation: u64,
    pub url: String,
}

pub(super) struct ArticleError {
    pub url: String,
    pub message: String,
}

#[derive(Default)]
pub(crate) struct ArticleLoads {
    pub(super) next: Cell<u64>,
    pub(super) pending: RefCell<BTreeMap<Id, Request>>,
    pub(super) errors: RefCell<BTreeMap<Id, ArticleError>>,
}

impl AppViewModel {
    pub(super) fn take_article_request(
        &self,
        id: Id,
        request_id: u64,
        generation: u64,
        url: &str,
    ) -> bool {
        if generation != self.library_generation()
            || self
                .state()
                .article(id)
                .is_none_or(|article| article.url != url)
        {
            return false;
        }
        let mut pending = self.article_loads.pending.borrow_mut();
        if !pending.get(&id).is_some_and(|request| {
            request.request_id == request_id
                && request.generation == generation
                && request.url == url
        }) {
            return false;
        }
        pending.remove(&id);
        true
    }

    /// Called before publishing changed feed/article identities. Cancel transport
    /// work after dropping map/document borrows; hosts may complete synchronously.
    pub(crate) fn prune_article_loads(&self) {
        let mut cancelled = Vec::new();
        {
            let state = self.state();
            let generation = self.library_generation();
            self.article_loads
                .pending
                .borrow_mut()
                .retain(|id, request| {
                    let keep = request.generation == generation
                        && state
                            .article(*id)
                            .is_some_and(|article| article.url == request.url);
                    if !keep {
                        cancelled.push(request.request_id);
                    }
                    keep
                });
            self.article_loads.errors.borrow_mut().retain(|id, error| {
                state
                    .article(*id)
                    .is_some_and(|article| article.url == error.url)
            });
        }
        for request_id in cancelled {
            self.request(HostRequest::CancelArticle { request_id });
        }
    }

    pub(crate) fn reset_article_loads(&self) {
        // A synchronous host can complete during a reactive notification, when
        // dispatch is queued. Such accepted work still belongs to the old library.
        self.pending.borrow_mut().retain(|action| {
            !matches!(
                action,
                reader_domain::Action::CacheFullArticle { .. }
                    | reader_domain::Action::ShowFullArticle { .. }
            )
        });
        let pending = self.article_loads.pending.take();
        self.article_loads.errors.borrow_mut().clear();
        for (_, request) in pending {
            self.request(HostRequest::CancelArticle {
                request_id: request.request_id,
            });
        }
    }
}
