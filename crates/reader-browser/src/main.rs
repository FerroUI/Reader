#![cfg_attr(target_os = "emscripten", no_main)]
mod app;
mod article_exports;
mod exports;
mod host;
mod input_hints;
mod inspection;
mod startup;

use ferroui_browser::{BrowserAccessibilityMode, BrowserAppBuilder, BrowserPlatformOptions};
use ferroui_controls::AppBuilder;
use ferroui_fonts_inter::AppBuilderExtension;
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name = runMain)]
pub fn run_main(
    regular: Vec<u8>,
    bold: Vec<u8>,
    italic: Vec<u8>,
    bold_italic: Vec<u8>,
    serif: Vec<u8>,
    mono: Vec<u8>,
    japanese: Vec<u8>,
) -> Result<(), JsValue> {
    let bytes = RefCell::new(Some(vec![regular, bold, italic, bold_italic, serif, mono, japanese]));
    let font_error = Rc::new(RefCell::new(None));
    let result = font_error.clone();
    AppBuilder::configure::<app::App>()
        .with_inter_font()
        .with(Rc::new(reader_fonts::font_options()))
        .configure_fonts(move |manager| {
            if let Some(fonts) = bytes.borrow_mut().take() {
                let fonts = fonts.iter().map(Vec::as_slice).collect::<Vec<_>>();
                *result.borrow_mut() = reader_fonts::install_fonts(manager, &fonts).err();
            }
        })
        .start_browser_app(
            "out",
            Some(BrowserPlatformOptions {
                accessibility: BrowserAccessibilityMode::Enabled,
                ..Default::default()
            }),
        );
    let error = font_error.borrow_mut().take();
    error.map_or(Ok(()), |e| Err(JsValue::from_str(&e)))
}

#[cfg(not(target_os = "emscripten"))]
fn main() {
    println!("Build Reader for the browser with scripts/build-browser.sh");
}
