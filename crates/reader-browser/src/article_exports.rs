//! Async article callbacks use the same guarded identity as native host replies.
use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name = receiveArticle)]
pub fn receive_article(
    id: f64,
    request_id: f64,
    generation: f64,
    url: &str,
    html: &str,
    base_url: &str,
) -> bool {
    crate::app::view_model().is_some_and(|vm| {
        vm.receive_article(
            id as u64,
            request_id as u64,
            generation as u64,
            url,
            html,
            base_url,
        )
        .unwrap_or(false)
    })
}

#[wasm_bindgen(js_name = articleError)]
pub fn article_error(id: f64, request_id: f64, generation: f64, url: &str, message: &str) -> bool {
    crate::app::view_model().is_some_and(|vm| {
        vm.article_error_response(
            id as u64,
            request_id as u64,
            generation as u64,
            url,
            message,
        )
    })
}
