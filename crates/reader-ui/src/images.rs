use ferroui_base::layout::HorizontalAlignment as H;
use ferroui_base::media::{
    imaging::{Bitmap, BitmapInterpolationMode},
    IImage, Stretch,
};
use ferroui_view::prelude::*;
use reader_state::{AppViewModel, HostRequest};
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    io::Cursor,
    rc::Rc,
};
struct Cache {
    images: HashMap<String, Rc<dyn IImage>>,
    requested: HashSet<String>,
    revision: Option<Signal<u64>>,
}
ferroui_base::ferro_thread_local! {
    static CACHE: RefCell<Cache> = RefCell::new(Cache{images:HashMap::new(),requested:HashSet::new(),revision:None});
}
pub fn initialize() {
    CACHE.with(|cache| cache.borrow_mut().revision = Some(signal(0)));
}
/// Host completion is always delivered on the FerroUI UI thread.
pub fn register_image(url: &str, bytes: Vec<u8>) -> Result<(), String> {
    let decoded = if bytes.is_empty() {
        None
    } else {
        Some(
            Bitmap::decode_to_width(
                &mut Cursor::new(bytes),
                1300,
                BitmapInterpolationMode::HighQuality,
            )
            .map_err(|e| e.to_string())?,
        )
    };
    let revision = CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        if let Some(bitmap) = decoded {
            if cache.images.len() >= 32 {
                if let Some(key) = cache.images.keys().next().cloned() {
                    cache.images.remove(&key);
                }
            }
            cache.images.insert(url.to_owned(), Rc::new(bitmap));
        }
        cache.revision
    });
    if let Some(revision) = revision {
        revision.update(|r| *r = r.wrapping_add(1));
    }
    Ok(())
}
pub fn view(vm: Rc<AppViewModel>, url: String, alt: String) -> Element {
    rerender(move || {
        let (bitmap, revision, request) = CACHE.with(|cache| {
            let mut cache = cache.borrow_mut();
            let request = cache.requested.insert(url.clone());
            (cache.images.get(&url).cloned(), cache.revision, request)
        });
        if let Some(revision) = revision {
            revision.get();
        }
        if request {
            vm.request(HostRequest::FetchImage { url: url.clone() });
        }
        if let Some(source) = bitmap {
            let open = vm.clone();
            let target = url.clone();
            image()
                .source(Some(source))
                .stretch(Stretch::Uniform)
                .max_width(650.0)
                .max_height(900.0)
                .horizontal_alignment(H::Left)
                .automation_properties_name(if alt.is_empty() {
                    "Article image"
                } else {
                    &alt
                })
                .on_tapped(move |_| {
                    open.request(HostRequest::OpenUrl {
                        url: target.clone(),
                    })
                })
                .into()
        } else {
            let open = vm.clone();
            let target = url.clone();
            reader_controls::link(
                if alt.is_empty() { "View image" } else { &alt },
                move || {
                    open.request(HostRequest::OpenUrl {
                        url: target.clone(),
                    })
                },
            )
        }
    })
}
