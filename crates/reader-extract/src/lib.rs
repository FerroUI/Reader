//! Small, lazy Web Worker bridge. No UI framework, rendering, or network code.
use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name = extractArticle)]
pub fn extract_article(html: &str, url: &str) -> Result<String, JsValue> {
    let article =
        reader_content::extract(html, url).map_err(|message| JsValue::from_str(&message))?;
    Ok(serde_json::json!({ "html": article.html, "base_url": article.base_url }).to_string())
}
