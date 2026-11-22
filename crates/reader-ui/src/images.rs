mod info;
mod layout;
mod view;
pub(crate) use layout::available_width;
pub use view::with_metadata;

use ferroui_base::media::{
    imaging::{Bitmap, BitmapInterpolationMode},
    IImage,
};
use ferroui_view::prelude::*;
use std::{
    cell::{Cell, RefCell},
    collections::{HashMap, HashSet, VecDeque},
    io::Cursor,
    rc::{Rc, Weak},
};

const CACHE_BYTES: usize = 32 * 1024 * 1024;
struct Entry {
    image: Rc<dyn IImage>,
    bytes: usize,
    used: u64,
}
enum ImageState {
    Waiting,
    Ready(Rc<dyn IImage>),
    Failed,
}
struct ImageLease {
    state: RefCell<ImageState>,
    active: Cell<bool>,
    natural: Cell<Option<(f64, f64)>>,
    revision: Signal<u64>,
}
struct Cache {
    images: HashMap<String, Entry>,
    pending: HashSet<String>,
    failed: VecDeque<String>,
    leases: HashMap<String, Vec<Weak<ImageLease>>>,
    bytes: usize,
    clock: u64,
}
ferroui_base::ferro_thread_local! {
    static CACHE: RefCell<Cache> = RefCell::new(Cache { images:HashMap::new(), pending:HashSet::new(),
        failed:VecDeque::new(), leases:HashMap::new(), bytes:0, clock:0 });
}
// Reactive leases belong to each mounted image, so completion does not invalidate
// unrelated images or every article in the stream.
pub fn initialize() {}

/// Host completion is delivered on the UI thread; dimensions are checked before native decoding.
pub fn register_image(url: &str, bytes: Vec<u8>) -> Result<(), String> {
    let decoded = if bytes.is_empty() {
        Ok(None)
    } else {
        decode(bytes).map(Some)
    };
    let error = decoded.as_ref().err().cloned();
    let revisions = CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        cache.pending.remove(url);
        let mut resolved = None;
        match decoded {
            Ok(Some((bitmap, bytes))) => {
                if let Some(previous) = cache.images.remove(url) {
                    cache.bytes -= previous.bytes;
                }
                while cache.bytes + bytes > CACHE_BYTES {
                    let key = cache
                        .images
                        .iter()
                        .min_by_key(|(_, item)| item.used)
                        .map(|(key, _)| key.clone());
                    let Some(key) = key else {
                        break;
                    };
                    if let Some(entry) = cache.images.remove(&key) {
                        cache.bytes -= entry.bytes;
                    }
                    // Evicted images can be requested again on revisit.
                }
                cache.clock = cache.clock.wrapping_add(1);
                let used = cache.clock;
                cache.bytes += bytes;
                cache.failed.retain(|failed| failed != url);
                let image: Rc<dyn IImage> = Rc::new(bitmap);
                resolved = Some(image.clone());
                cache
                    .images
                    .insert(url.to_owned(), Entry { image, bytes, used });
            }
            _ => {
                if !cache.failed.iter().any(|failed| failed == url) {
                    cache.failed.push_back(url.to_owned());
                    if cache.failed.len() > 256 {
                        cache.failed.pop_front();
                    }
                }
            }
        }
        let mut revisions = Vec::new();
        if let Some(leases) = cache.leases.get_mut(url) {
            leases.retain(|weak| {
                let Some(lease) = weak.upgrade() else {
                    return false;
                };
                *lease.state.borrow_mut() = resolved.as_ref().map_or(ImageState::Failed, |image| {
                    let size = image.size();
                    lease.natural.set(Some((size.width, size.height)));
                    if lease.active.get() {
                        ImageState::Ready(image.clone())
                    } else {
                        ImageState::Waiting
                    }
                });
                revisions.push(lease.revision);
                true
            });
        }
        revisions
    });
    for revision in revisions {
        revision.update(|r| *r = r.wrapping_add(1));
    }
    error.map_or(Ok(()), Err)
}

fn decode(bytes: Vec<u8>) -> Result<(Bitmap, usize), String> {
    let size = info::dimensions(&bytes)?;
    let mut stream = Cursor::new(bytes);
    // Fit a 2x reading viewport on both axes, never enlarge a small source image.
    let bitmap = if u64::from(size.height) * 1300 > u64::from(size.width) * 1800 {
        Bitmap::decode_to_height(
            &mut stream,
            size.height.min(1800) as i32,
            BitmapInterpolationMode::HighQuality,
        )
    } else {
        Bitmap::decode_to_width(
            &mut stream,
            size.width.min(1300) as i32,
            BitmapInterpolationMode::HighQuality,
        )
    }
    .map_err(|error| error.to_string())?;
    let pixels = bitmap.pixel_size();
    let bytes = (pixels.width as usize)
        .checked_mul(pixels.height as usize)
        .and_then(|n| n.checked_mul(4))
        .filter(|&bytes| bytes > 0 && bytes <= CACHE_BYTES)
        .ok_or("Decoded image exceeds the reading cache budget.")?;
    Ok((bitmap, bytes))
}
