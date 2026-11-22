use super::geometry::GeometryCache;
use crate::rows::Row;
use ferroui_controls::ItemsSourceView;
use reader_domain::Density;
use std::ops::{Deref, DerefMut};

/// Cache lifetimes follow the source collection, never the realized window.
pub(super) struct Metrics {
    cache: GeometryCache<RowGeometry>,
    pub dirty: bool,
}

impl Default for Metrics {
    fn default() -> Self {
        Self {
            cache: GeometryCache::default(),
            dirty: true,
        }
    }
}

impl Deref for Metrics {
    type Target = GeometryCache<RowGeometry>;
    fn deref(&self) -> &Self::Target {
        &self.cache
    }
}
impl DerefMut for Metrics {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.cache
    }
}

impl Metrics {
    pub fn synchronize(&mut self, items: &ItemsSourceView, width: f64) {
        if !self.dirty && (width - self.width).abs() <= 0.5 && self.extents.len() == items.count() {
            return;
        }
        self.cache.reset(
            width,
            (0..items.count()).map(|index| {
                let item = items.get_at(index);
                let row = item
                    .as_ref()
                    .and_then(|value| value.downcast_ref::<Row>())
                    .expect("Reader's article panel receives projected article rows")
                    .clone();
                let estimate = estimate(&row, width);
                (row.article.id, RowGeometry(row), estimate)
            }),
        );
        self.dirty = false;
    }

    pub fn replace(&mut self, index: usize, row: Row) {
        let estimate = estimate(&row, self.width);
        if !self
            .cache
            .replace(index, row.article.id, RowGeometry(row), estimate)
        {
            self.dirty = true;
        }
    }
}

/// Read/star/cursor changes preserve geometry. Full body, reading controls,
/// heading context, annotations and presentation settings can change its size.
pub(super) struct RowGeometry(Row);
impl PartialEq for RowGeometry {
    fn eq(&self, other: &Self) -> bool {
        let (old, new) = (&self.0, &other.0);
        old.expanded == new.expanded
            && old.density == new.density
            && old.font_size == new.font_size
            && old.narrow == new.narrow
            && old.touch == new.touch
            && (!new.expanded
                || (old.source == new.source
                    && old.single_source == new.single_source
                    && old.article_loading == new.article_loading
                    && old.article_error == new.article_error
                    && old.article.has_full_content() == new.article.has_full_content()
                    && old.article.show_full_content == new.article.show_full_content
                    && old.article.title == new.article.title
                    && old.article.author == new.article.author
                    && old.article.reading_content() == new.article.reading_content()
                    && old.article.reading_base_url() == new.article.reading_base_url()
                    && old.article.tags == new.article.tags
                    && old.article.note == new.article.note))
    }
}

fn estimate(row: &Row, width: f64) -> f64 {
    if !row.expanded {
        return 1.0
            + if row.touch {
                44.0
            } else {
                match row.density {
                    Density::Comfortable => 37.0,
                    Density::Cozy => 31.0,
                    Density::Compact => 27.0,
                }
            };
    }
    let text_width = (width - if row.narrow { 24.0 } else { 48.0 }).clamp(160.0, 650.0);
    let characters_per_line = (text_width / (row.font_size * 0.49)).max(12.0);
    // Initial guesses only: replace each visited row with its actual measurement,
    // without changing estimates for the other thousands of unseen articles.
    let characters = row.article.reading_content().len() as f64 * 0.58;
    let lines = (characters / characters_per_line).ceil().max(2.0);
    (184.0 + lines * row.font_size * 1.55).clamp(216.0, 48_000.0)
}
