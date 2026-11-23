//! Inspect the active native font manager, so a machine's installed fonts cannot
//! mask a missing bundled browser fallback in the rendered verification suite.
use ferroui_base::media::{FontFamily, FontManager, FontStretch, FontStyle, FontWeight, Typeface};
use serde_json::json;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name = inspectGlyphs)]
pub fn inspect_glyphs(text: &str, family: &str, bold: bool, italic: bool) -> String {
    let manager = FontManager::current();
    let family = FontFamily::new(family);
    let style = if italic {
        FontStyle::Italic
    } else {
        FontStyle::Normal
    };
    let weight = if bold {
        FontWeight::Bold
    } else {
        FontWeight::Normal
    };
    let primary = manager.try_get_glyph_typeface(&Typeface::with_style(
        family.clone(),
        style,
        weight,
        FontStretch::Normal,
    ));
    let glyphs = text.chars().take(256).map(|character| {
        let codepoint = character as i32;
        // Formatting first uses its requested face and asks the manager for a
        // fallback only when that face cannot represent the character.
        let face = primary.as_ref()
            .filter(|face| face.character_to_glyph_map().try_get_glyph(codepoint).is_some_and(|g| g != 0))
            .cloned()
            .or_else(|| manager.try_match_character(codepoint, style, weight,
                FontStretch::Normal, Some(&family), None)
                .and_then(|matched| manager.try_get_glyph_typeface(&matched)));
        json!({"character":character.to_string(),
            "family":face.as_ref().map(|face| face.family_name()),
            "glyph":face.as_ref().and_then(|face| face.character_to_glyph_map().try_get_glyph(codepoint)),
            "advance":face.as_ref().and_then(|face| face.character_to_glyph_map().try_get_glyph(codepoint)
                .and_then(|glyph| face.try_get_horizontal_glyph_advance(glyph)))})
    }).collect::<Vec<_>>();
    serde_json::to_string(&glyphs).expect("font inspection is serializable")
}
