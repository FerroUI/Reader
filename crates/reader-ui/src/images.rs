mod info;

use ferroui_base::layout::HorizontalAlignment as H;
use ferroui_base::media::{
    imaging::{Bitmap, BitmapInterpolationMode},
    IImage, Stretch,
};
use ferroui_view::prelude::*;
use reader_state::{AppViewModel, HostRequest};
use std::{
    cell::RefCell,
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
type ImageLease = RefCell<ImageState>;
struct Cache {
    images: HashMap<String, Entry>,
    pending: HashSet<String>,
    failed: VecDeque<String>,
    leases: HashMap<String, Vec<Weak<ImageLease>>>,
    bytes: usize,
    clock: u64,
    revision: Option<Signal<u64>>,
}
ferroui_base::ferro_thread_local! {
    static CACHE: RefCell<Cache> = RefCell::new(Cache { images:HashMap::new(), pending:HashSet::new(),
        failed:VecDeque::new(), leases:HashMap::new(), bytes:0, clock:0, revision:None });
}
pub fn initialize() {
    CACHE.with(|cache| cache.borrow_mut().revision = Some(signal(0)));
}

/// Host completion is delivered on the UI thread; dimensions are checked before native decoding.
pub fn register_image(url: &str, bytes: Vec<u8>) -> Result<(), String> {
    let decoded = if bytes.is_empty() {
        Ok(None)
    } else {
        decode(bytes).map(Some)
    };
    let error = decoded.as_ref().err().cloned();
    let revision = CACHE.with(|cache| {
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
        if let Some(leases) = cache.leases.get_mut(url) {
            leases.retain(|weak| {
                let Some(lease) = weak.upgrade() else {
                    return false;
                };
                *lease.borrow_mut() = resolved
                    .as_ref()
                    .map_or(ImageState::Failed, |image| ImageState::Ready(image.clone()));
                true
            });
        }
        cache.revision
    });
    if let Some(revision) = revision {
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

pub fn view(vm: Rc<AppViewModel>, url: String, alt: String) -> Element {
    // A mounted image keeps its decoded source when the reusable cache evicts it.
    // Completion resolves leases before batching a render, so many simultaneous images cannot
    // evict each other into a permanent reload loop.
    let lease = Rc::new(RefCell::new(ImageState::Waiting));
    CACHE.with(|cache| {
        cache
            .borrow_mut()
            .leases
            .entry(url.clone())
            .or_default()
            .push(Rc::downgrade(&lease))
    });
    let cleanup_url = url.clone();
    let cleanup_lease = Rc::downgrade(&lease);
    on_cleanup(move || {
        CACHE.with(|cache| {
            let mut cache = cache.borrow_mut();
            if let Some(leases) = cache.leases.get_mut(&cleanup_url) {
                leases.retain(|weak| weak.strong_count() > 0 && !weak.ptr_eq(&cleanup_lease));
                if leases.is_empty() {
                    cache.leases.remove(&cleanup_url);
                }
            }
        })
    });
    rerender(move || -> Element {
        let (bitmap, revision, request) = CACHE.with(|cache| {
            let mut cache = cache.borrow_mut();
            cache.clock = cache.clock.wrapping_add(1);
            let used = cache.clock;
            let cached = cache.images.get_mut(&url).map(|entry| {
                entry.used = used;
                entry.image.clone()
            });
            if let Some(image) = cached {
                *lease.borrow_mut() = ImageState::Ready(image);
            } else if cache.failed.iter().any(|failed| failed == &url) {
                *lease.borrow_mut() = ImageState::Failed;
            }
            let bitmap = match &*lease.borrow() {
                ImageState::Ready(image) => Some(image.clone()),
                _ => None,
            };
            let failed = matches!(*lease.borrow(), ImageState::Failed);
            let request = bitmap.is_none()
                && !failed
                && !cache.failed.iter().any(|failed| failed == &url)
                && cache.pending.insert(url.clone());
            (bitmap, cache.revision, request)
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
            .into()
        }
    })
}
