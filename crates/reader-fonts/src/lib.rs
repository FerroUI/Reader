//! Shared metrics-compatible Arial substitute, kept outside the browser WASM module.
use ferroui_base::media::fonts::{FontCollectionBase, FontCollectionBaseImpl};
use ferroui_base::media::{FontManager, FontManagerOptions};
use std::{io::Cursor, rc::Rc};
use url::Url;

pub const SANS: &str = "fonts:Reader#Liberation Sans";
pub const SERIF: &str = "fonts:Reader#Liberation Serif";

pub fn font_options() -> FontManagerOptions {
    FontManagerOptions {
        default_family_name: Some(SANS.into()),
        ..Default::default()
    }
}

pub fn install_fonts(manager: &FontManager, fonts: &[&[u8]]) -> Result<(), String> {
    let collection = Rc::new(ReaderFonts {
        base: FontCollectionBase::new(),
    });
    for bytes in fonts {
        FontCollectionBase::try_add_glyph_typeface_from_stream(
            &*collection,
            &mut Cursor::new(bytes),
        )
        .ok_or_else(|| "The Reader font could not be loaded.".to_owned())?;
    }
    manager.add_font_collection(collection);
    Ok(())
}

struct ReaderFonts {
    base: FontCollectionBase,
}
impl FontCollectionBaseImpl for ReaderFonts {
    fn base(this: &Self) -> &FontCollectionBase {
        &this.base
    }
    fn key(_: &Self) -> Url {
        Url::parse("fonts:Reader").expect("static font collection URI")
    }
}

#[cfg(feature = "embedded")]
pub const BUNDLED_FONTS: &[&[u8]] = &[
    include_bytes!("../assets/fonts/LiberationSans-Regular.ttf"),
    include_bytes!("../assets/fonts/LiberationSans-Bold.ttf"),
    include_bytes!("../assets/fonts/LiberationSans-Italic.ttf"),
    include_bytes!("../assets/fonts/LiberationSans-BoldItalic.ttf"),
    include_bytes!("../assets/fonts/LiberationSerif-Regular.ttf"),
    include_bytes!("../assets/fonts/LiberationMono-Regular.ttf"),
];
