use reader_domain::View;
use std::rc::Rc;
use wasm_bindgen::prelude::*;

fn vm() -> Result<Rc<reader_state::AppViewModel>, JsValue> {
    crate::app::view_model().ok_or_else(|| JsValue::from_str("Reader is still starting"))
}
fn error(message: String) -> JsValue {
    JsValue::from_str(&message)
}

#[wasm_bindgen(js_name = exportData)]
pub fn export_data() -> Result<String, JsValue> {
    vm()?.export_json().map_err(error)
}
#[wasm_bindgen(js_name = flushSave)]
pub fn flush_save() -> Result<(), JsValue> {
    vm()?.flush().map_err(error)
}
#[wasm_bindgen(js_name = receiveFeed)]
pub fn receive_feed(
    url: &str,
    xml: &str,
    open: bool,
    expected_id: f64,
    base_url: &str,
    expected_epoch: f64,
) -> Result<(), JsValue> {
    let vm = vm()?;
    if expected_id > 0.0 && !vm.accepts_feed_result(expected_id as u64, url, expected_epoch as u64)
    {
        return Ok(()); // An unsubscribed/replaced feed must not reappear when its request finishes.
    }
    let id = vm.receive_feed_from(url, xml, base_url).map_err(error)?;
    if open {
        vm.navigate(View::Feed(id));
    }
    Ok(())
}
#[wasm_bindgen(js_name = feedError)]
pub fn feed_error(url: &str, message: &str, expected_id: f64, expected_epoch: f64) {
    if let Ok(vm) = vm() {
        if expected_id > 0.0
            && !vm.accepts_feed_result(expected_id as u64, url, expected_epoch as u64)
        {
            return;
        }
        vm.feed_error(url, message);
    }
}
#[wasm_bindgen(js_name = subscriptionEpoch)]
pub fn subscription_epoch(id: f64) -> f64 {
    vm().ok()
        .and_then(|vm| vm.subscription_epoch(id as u64))
        .unwrap_or_default() as f64
}
#[wasm_bindgen(js_name = finishRefresh)]
pub fn finish_refresh() {
    if let Ok(vm) = vm() {
        vm.finish_refresh();
    }
}
#[wasm_bindgen(js_name = refreshInitialFeeds)]
pub fn refresh_initial_feeds() -> bool {
    vm().is_ok_and(|vm| vm.refresh_initial_feeds())
}
#[wasm_bindgen(js_name = importOpml)]
pub fn import_opml(contents: &str) -> Result<u32, JsValue> {
    vm()?.import_opml(contents).map(|n| n as u32).map_err(error)
}
#[wasm_bindgen(js_name = importData)]
pub fn import_data(contents: &str) -> Result<(), JsValue> {
    vm()?.import_json(contents).map_err(error)
}
#[wasm_bindgen(js_name = hostError)]
pub fn host_error(message: &str) {
    if let Ok(vm) = vm() {
        vm.report_error(message.into());
    }
}
#[wasm_bindgen(js_name = storageError)]
pub fn storage_error(message: &str) {
    if let Ok(vm) = vm() {
        vm.persistence_failed(message.into());
    }
}
#[wasm_bindgen(js_name = hostStatus)]
pub fn host_status(message: &str) {
    if let Ok(vm) = vm() {
        vm.set_status(message.into());
    }
}
#[wasm_bindgen(js_name = appUpdateStatus)]
pub fn app_update_status(message: &str, applying: bool) {
    if let Ok(vm) = vm() {
        vm.set_update_notice(message.into(), applying);
    }
}
#[wasm_bindgen(js_name = resizeWindow)]
pub fn resize_window(width: f64) {
    if width.is_finite() && width > 0.0 {
        if let Ok(vm) = vm() {
            vm.set_viewport(width);
        }
    }
}
#[wasm_bindgen(js_name = resizeViewport)]
pub fn resize_viewport(width: f64, height: f64) {
    if let Ok(vm) = vm() {
        vm.set_viewport_size(width, height);
    }
}
#[wasm_bindgen(js_name = setTouchMode)]
pub fn set_touch_mode(enabled: bool) {
    if let Ok(vm) = vm() {
        vm.set_touch_mode(enabled);
    }
}
#[wasm_bindgen(js_name = tick)]
pub fn tick(now: f64) {
    if now.is_finite() {
        if let Ok(vm) = vm() {
            vm.set_now(now as i64);
        }
    }
}
#[wasm_bindgen(js_name = receiveImage)]
pub fn receive_image(url: &str, bytes: Vec<u8>) -> Result<(), JsValue> {
    reader_ui::register_image(url, bytes).map_err(error)
}
#[wasm_bindgen(js_name = focusSearch)]
pub fn focus_search() {
    reader_ui::focus_search();
}
#[wasm_bindgen(js_name = scrollToArticle)]
pub fn scroll_to_article(id: f64) {
    if id.is_finite() && id > 0.0 {
        reader_ui::scroll_to_article(id as u64);
    }
}
#[wasm_bindgen(js_name = currentView)]
pub fn current_view() -> String {
    crate::app::view_model()
        .and_then(|v| serde_json::to_string(&v.view()).ok())
        .unwrap_or_default()
}
#[wasm_bindgen(js_name = libraryGeneration)]
pub fn library_generation() -> f64 {
    vm().map(|vm| vm.library_generation() as f64)
        .unwrap_or_default()
}
#[wasm_bindgen(js_name = navigate)]
pub fn navigate(view: &str) -> Result<(), JsValue> {
    let view = serde_json::from_str::<View>(view).map_err(|e| error(e.to_string()))?;
    for _ in 0..8 {
        if !dismiss_overlay() {
            break;
        }
    }
    vm()?.navigate(view);
    Ok(())
}
#[wasm_bindgen(js_name = hasOverlay)]
pub fn has_overlay() -> bool {
    reader_ui::has_transient_ui() || vm().is_ok_and(|vm| vm.viewport() < 700.0 && vm.sidebar_open())
}
#[wasm_bindgen(js_name = dismissOverlay)]
pub fn dismiss_overlay() -> bool {
    if reader_ui::dismiss_transient_ui() {
        return true;
    }
    let mut dismissed = false;
    if let Ok(vm) = vm() {
        if vm.viewport() < 700.0 && vm.sidebar_open() {
            vm.set_sidebar_open(false);
            dismissed = true;
        }
    }
    dismissed
}
