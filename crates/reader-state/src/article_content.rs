//! On-demand article requests stay independent from feed refresh and stream navigation.
mod requests;
use crate::{AppViewModel, HostRequest};
use reader_domain::{Action, FullArticle, Id};
pub(crate) use requests::ArticleLoads;
use requests::{ArticleError, Request};

impl AppViewModel {
    pub fn article_loading(&self, id: Id) -> bool {
        self.article_loads.pending.borrow().contains_key(&id)
    }

    pub fn article_error(&self, id: Id) -> Option<String> {
        self.article_loads
            .errors
            .borrow()
            .get(&id)
            .map(|error| error.message.clone())
    }

    /// A saved copy opens immediately. Network work starts only for an explicit
    /// request or reload; selecting an ordinary feed article never prefetches it.
    pub fn load_full_article(&self, id: Id, reload: bool) -> bool {
        if self.article_loading(id) {
            return false;
        }
        let Some(article) = self.article_shared(id) else {
            return false;
        };
        if !reload && article.has_full_content() {
            self.article_loads.errors.borrow_mut().remove(&id);
            self.dispatch(Action::ShowFullArticle { id, show: true });
            self.notify();
            return false;
        }
        if !self.has_services.get() {
            self.set_article_error(
                id,
                "Article downloading is not available in this host.".into(),
            );
            return false;
        }
        let Some(url) = reader_domain::safe_url(&article.url, None)
            .filter(|url| reader_domain::subscription_url(url).is_some())
        else {
            self.set_article_error(
                id,
                "This item has no valid HTTP or HTTPS article link.".into(),
            );
            return false;
        };
        let Some(request_id) = self.article_loads.next.get().checked_add(1) else {
            self.set_article_error(
                id,
                "Reopen Reader before starting another article download.".into(),
            );
            return false;
        };
        self.article_loads.next.set(request_id);
        let generation = self.library_generation();
        let (proxy, connection) = {
            let state = self.state();
            (
                state.settings.feed_proxy.clone(),
                state.settings.feed_connection,
            )
        };
        self.article_loads.errors.borrow_mut().remove(&id);
        self.article_loads.pending.borrow_mut().insert(
            id,
            Request {
                request_id,
                generation,
                url: url.clone(),
            },
        );
        self.request(HostRequest::FetchArticle {
            id,
            request_id,
            generation,
            url,
            proxy,
            connection,
        });
        self.notify();
        true
    }

    pub fn cancel_full_article(&self, id: Id) {
        let pending = self.article_loads.pending.borrow_mut().remove(&id);
        self.article_loads.errors.borrow_mut().remove(&id);
        if let Some(request) = pending {
            self.request(HostRequest::CancelArticle {
                request_id: request.request_id,
            });
        }
        self.notify();
    }

    pub fn show_feed_content(&self, id: Id) {
        self.cancel_full_article(id);
        self.dispatch(Action::ShowFullArticle { id, show: false });
    }

    pub fn show_full_content(&self, id: Id) {
        self.load_full_article(id, false);
    }

    /// A response cannot affect another article or library, even if an import
    /// reuses its numeric identifiers and URL. False means a stale response.
    pub fn receive_article(
        &self,
        id: Id,
        request_id: u64,
        generation: u64,
        url: &str,
        html: impl Into<String>,
        base_url: &str,
    ) -> Result<bool, String> {
        if !self.take_article_request(id, request_id, generation, url) {
            return Ok(false);
        }
        let content = match FullArticle::new(html.into(), url, base_url, self.now()) {
            Ok(content) => content,
            Err(error) => {
                self.set_article_error(id, error.clone());
                return Err(error);
            }
        };
        self.article_loads.errors.borrow_mut().remove(&id);
        self.dispatch(Action::CacheFullArticle { id, content });
        self.notify();
        Ok(true)
    }

    pub fn article_error_response(
        &self,
        id: Id,
        request_id: u64,
        generation: u64,
        url: &str,
        message: &str,
    ) -> bool {
        if !self.take_article_request(id, request_id, generation, url) {
            return false;
        }
        self.set_article_error(id, message.to_string());
        true
    }

    fn set_article_error(&self, id: Id, message: String) {
        // An unbounded upstream error page never becomes a native status label.
        let message: String = message.chars().take(500).collect();
        let url = self
            .state()
            .article(id)
            .map(|article| article.url.clone())
            .unwrap_or_default();
        self.article_loads
            .errors
            .borrow_mut()
            .insert(id, ArticleError { url, message });
        self.notify();
    }
}
