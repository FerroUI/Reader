use super::{layout, ImageLease, ImageState, CACHE};
use ferroui_base::layout::{HorizontalAlignment as H, VerticalAlignment as V};
use ferroui_base::media::{Stretch, TextWrapping};
use ferroui_view::prelude::*;
use reader_state::{AppViewModel, HostRequest};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

pub fn with_metadata(
    vm: Rc<AppViewModel>,
    metadata: reader_content::projection::Image,
    body_key: String,
    available: f64,
) -> Element {
    let key = format!("image-{body_key}-{:016x}", available.to_bits());
    // A keyed component owns the lease. Parent row patches keep the same hook,
    // while another article/body/image or width remounts both metadata and the
    // mount-only native viewport subscription together.
    let component = rerender(move || {
        let lease = use_hook(|| mount_lease(&metadata.url));
        render(vm.clone(), &metadata, available, lease)
    });
    ferroui_view::element::with_key(component, key.into())
}

fn mount_lease(url: &str) -> Rc<ImageLease> {
    let url = url.to_owned();
    let lease = Rc::new(ImageLease {
        state: RefCell::new(ImageState::Waiting),
        active: Cell::new(false),
        natural: Cell::new(None),
        revision: signal(0),
    });
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
    lease
}

fn render(
    vm: Rc<AppViewModel>,
    metadata: &reader_content::projection::Image,
    available: f64,
    lease: Rc<ImageLease>,
) -> Element {
    let url = metadata.url.clone();
    let viewport_lease = lease.clone();
    let contents: Element = {
        lease.revision.get();
        let active = lease.active.get();
        let (bitmap, failed, request) = CACHE.with(|cache| {
            let mut cache = cache.borrow_mut();
            cache.clock = cache.clock.wrapping_add(1);
            let used = cache.clock;
            let cached = cache.images.get_mut(&url).map(|entry| {
                entry.used = used;
                entry.image.clone()
            });
            if let Some(image) = cached {
                let size = image.size();
                lease.natural.set(Some((size.width, size.height)));
                *lease.state.borrow_mut() = if active {
                    ImageState::Ready(image)
                } else {
                    ImageState::Waiting
                };
            } else if cache.failed.iter().any(|failed| failed == &url) {
                *lease.state.borrow_mut() = ImageState::Failed;
            } else if !active {
                *lease.state.borrow_mut() = ImageState::Waiting;
            }
            let bitmap = match &*lease.state.borrow() {
                ImageState::Ready(image) => Some(image.clone()),
                _ => None,
            };
            let failed = matches!(*lease.state.borrow(), ImageState::Failed);
            let request =
                active && bitmap.is_none() && !failed && cache.pending.insert(url.clone());
            (bitmap, failed, request)
        });
        if request {
            vm.request(HostRequest::FetchImage { url: url.clone() });
        }
        let (width, height) = layout::size(
            available,
            metadata.width.zip(metadata.height),
            lease.natural.get(),
        );
        let open = vm.clone();
        let target = metadata.link.as_ref().unwrap_or(&url).clone();
        let contents: Element = if let Some(source) = bitmap {
            image()
                .source(Some(source))
                .stretch(Stretch::Uniform)
                .width(width)
                .height(height)
                .automation_properties_name(if metadata.alt.is_empty() {
                    "Article image"
                } else {
                    &metadata.alt
                })
                .on_tapped(move |_| {
                    open.request(HostRequest::OpenUrl {
                        url: target.clone(),
                    })
                })
                .into()
        } else if !active || width < 96.0 || height < 64.0 {
            // Small inline icons and publisher tracking pixels must not grow a
            // large loading label beyond their reserved native image geometry.
            border().into()
        } else {
            let label = if failed {
                "Image unavailable — view original"
            } else {
                "Loading image…"
            };
            stack_panel()
                .spacing(8.0)
                .horizontal_alignment(H::Center)
                .vertical_alignment(V::Center)
                .max_width((width - 24.0).max(60.0))
                .child(
                    text_block()
                        .text(label)
                        .foreground("#777777")
                        .text_wrapping(TextWrapping::Wrap),
                )
                .child(
                    button()
                        .class("reader-link")
                        .horizontal_content_alignment(H::Center)
                        .automation_properties_name("View original image")
                        .on_click(move |_| {
                            open.request(HostRequest::OpenUrl {
                                url: target.clone(),
                            })
                        })
                        .child(
                            text_block()
                                .text(if metadata.alt.is_empty() {
                                    "View image"
                                } else {
                                    &metadata.alt
                                })
                                .text_wrapping(TextWrapping::Wrap),
                        ),
                )
                .into()
        };
        border()
            .width(width)
            .height(height)
            .horizontal_alignment(H::Left)
            .background("#F7F7F7")
            .child(contents)
            .into()
    };
    border()
        .name("ArticleImage")
        .horizontal_alignment(H::Left)
        .max_width(available)
        .with(move |container| {
            let weak = container.downgrade();
            let lease = viewport_lease.clone();
            let token = container.effective_viewport_changed(move |event| {
                let Some(container) = weak.upgrade() else {
                    return;
                };
                let viewport = event.effective_viewport();
                let active = viewport.width > 0.0
                    && layout::near_viewport(
                        viewport.y,
                        viewport.height,
                        container.bounds().height,
                    );
                if lease.active.replace(active) != active {
                    lease
                        .revision
                        .update(|revision| *revision = revision.wrapping_add(1));
                }
            });
            on_cleanup(move || token.dispose());
        })
        .child(contents)
        .into()
}
