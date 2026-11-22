use wasm_bindgen::prelude::*;

#[wasm_bindgen(raw_module = "./host.js")]
extern "C" {
    #[wasm_bindgen(catch)]
    pub fn load() -> Result<String, JsValue>;
    #[wasm_bindgen(catch)]
    pub fn save(contents: &str) -> Result<(), JsValue>;
    pub fn now() -> f64;
    pub fn width() -> f64;
    pub fn request(json: &str);
    #[wasm_bindgen(js_name = preserveRecovery)]
    pub fn preserve_recovery(contents: &str) -> bool;
}

pub fn error_message(error: JsValue) -> String {
    error.as_string().unwrap_or_else(|| {
        "Browser storage is unavailable. Export a backup to keep your library.".into()
    })
}
