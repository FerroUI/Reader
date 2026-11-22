use super::{
    html::{attr, dimension},
    model::{Image, MAX_IMAGES},
    state::State,
};

pub(super) fn append(state: &mut State, raw: &str, base: &str) {
    let Some(url) = ["data-src", "data-original", "data-lazy-src", "src"]
        .iter()
        .find_map(|key| attr(raw, key).and_then(|src| reader_domain::safe_url(&src, Some(base))))
        .or_else(|| image_source_set(raw, base))
    else {
        return;
    };
    if state.images >= MAX_IMAGES {
        state.media("View additional image", url);
        return;
    }
    state.images += 1;
    state.flush();
    state.block.image = Some(Image {
        url,
        alt: attr(raw, "alt").unwrap_or_default(),
        width: dimension(raw, "width"),
        height: dimension(raw, "height"),
        link: state.hrefs.last().cloned().flatten(),
    });
    state.flush();
}

fn image_source_set(raw: &str, base: &str) -> Option<String> {
    let sources = attr(raw, "data-srcset").or_else(|| attr(raw, "srcset"))?;
    if sources.contains("data:") {
        return None;
    }
    // Prefer a 2x reading-width source, without fetching every responsive image.
    let mut candidates: Vec<_> = sources
        .split(',')
        .take(128)
        .filter_map(|candidate| {
            let mut parts = candidate.split_ascii_whitespace();
            let url = reader_domain::safe_url(parts.next()?, Some(base))?;
            let descriptor = parts.next().unwrap_or("1x");
            let width = descriptor
                .strip_suffix('w')
                .and_then(|value| value.parse::<f64>().ok())
                .or_else(|| {
                    descriptor
                        .strip_suffix('x')
                        .and_then(|value| value.parse::<f64>().ok())
                        .map(|scale| scale * 650.0)
                })
                .filter(|width| width.is_finite() && *width > 0.0)
                .unwrap_or(650.0);
            Some((width, url))
        })
        .collect();
    candidates.sort_by(|a, b| a.0.total_cmp(&b.0));
    let index = candidates
        .iter()
        .position(|(width, _)| *width >= 1300.0)
        .unwrap_or_else(|| candidates.len().saturating_sub(1));
    (!candidates.is_empty()).then(|| candidates.swap_remove(index).1)
}
