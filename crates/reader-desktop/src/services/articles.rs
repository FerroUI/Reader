use crate::{background, transport};
use ferroui_base::threading::Dispatcher;
use reader_state::{AppViewModel, HostRequest};
use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

#[derive(Default)]
pub(super) struct ArticleRequests {
    pending: RefCell<BTreeMap<u64, transport::DownloadCancel>>,
}

impl ArticleRequests {
    pub fn cancel_all(&self) {
        for (_, cancel) in self.pending.take() {
            cancel.cancel();
        }
    }

    /// Register before yielding to the dispatcher so cancellation also covers
    /// work that has not yet reached one of the four blocking-I/O workers.
    pub fn handle(self: &Rc<Self>, vm: &Rc<AppViewModel>, request: &HostRequest) -> bool {
        match request {
            HostRequest::CancelArticle { request_id } => {
                if let Some(cancel) = self.pending.borrow_mut().remove(request_id) {
                    cancel.cancel();
                }
            }
            HostRequest::FetchArticle {
                id,
                request_id,
                generation,
                url,
                ..
            } => {
                let (id, request_id, generation, url) =
                    (*id, *request_id, *generation, url.clone());
                let cancel = transport::DownloadCancel::default();
                self.pending.borrow_mut().insert(request_id, cancel.clone());
                let operations = self.clone();
                let vm = Rc::downgrade(vm);
                Dispatcher::ui_thread().spawn_local(async move {
                    let source = url.clone();
                    let result =
                        background::run(move || transport::article(&source, &cancel)).await;
                    if operations
                        .pending
                        .borrow_mut()
                        .remove(&request_id)
                        .is_none()
                    {
                        return;
                    }
                    let Some(vm) = vm.upgrade() else {
                        return;
                    };
                    match result {
                        Ok(content) => {
                            // Invalid extracted content becomes the article's inline
                            // error; stale replies never affect its page or read state.
                            let _ = vm.receive_article(
                                id,
                                request_id,
                                generation,
                                &url,
                                content.html,
                                &content.base_url,
                            );
                        }
                        Err(error) => {
                            vm.article_error_response(id, request_id, generation, &url, &error);
                        }
                    }
                });
            }
            _ => return false,
        }
        true
    }
}

impl Drop for ArticleRequests {
    fn drop(&mut self) {
        self.cancel_all();
    }
}
