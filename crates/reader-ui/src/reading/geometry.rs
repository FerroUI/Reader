//! Stable article identities and measured geometry, independent of UI objects.
use super::extent::Extents;
use std::collections::HashMap;

struct Entry<G> {
    id: u64,
    geometry: G,
    measured: Option<f64>,
}

pub(super) struct GeometryCache<G> {
    entries: Vec<Entry<G>>,
    indices: HashMap<u64, usize>,
    pub extents: Extents,
    pub width: f64,
}

impl<G> Default for GeometryCache<G> {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            indices: HashMap::new(),
            extents: Extents::default(),
            width: 0.0,
        }
    }
}

impl<G: PartialEq> GeometryCache<G> {
    pub fn reset(&mut self, width: f64, rows: impl IntoIterator<Item = (u64, G, f64)>) {
        let width_changed = (width - self.width).abs() > 0.5;
        let mut previous: HashMap<_, _> = self
            .entries
            .drain(..)
            .map(|entry| (entry.id, entry))
            .collect();
        self.width = width;
        self.indices.clear();
        let mut heights = Vec::new();
        for (id, geometry, estimate) in rows {
            let measured = previous
                .remove(&id)
                .filter(|old| !width_changed && old.geometry == geometry)
                .and_then(|old| old.measured);
            heights.push(measured.unwrap_or(estimate));
            self.indices.insert(id, self.entries.len());
            self.entries.push(Entry {
                id,
                geometry,
                measured,
            });
        }
        self.extents.reset(heights);
    }

    pub fn replace(&mut self, index: usize, id: u64, geometry: G, estimate: f64) -> bool {
        let Some(entry) = self.entries.get_mut(index).filter(|entry| entry.id == id) else {
            return false;
        };
        if entry.geometry != geometry {
            entry.measured = None;
            self.extents.measure(index, estimate);
        }
        entry.geometry = geometry;
        true
    }

    pub fn measure(&mut self, index: usize, height: f64) {
        if height.is_finite() && height > 0.0 {
            if let Some(entry) = self.entries.get_mut(index) {
                entry.measured = Some(height);
                self.extents.measure(index, height);
            }
        }
    }

    pub fn id(&self, index: usize) -> Option<u64> {
        self.entries.get(index).map(|entry| entry.id)
    }
    pub fn index(&self, id: u64) -> Option<usize> {
        self.indices.get(&id).copied()
    }

    pub fn restore(&self, id: u64, within: f64) -> Option<f64> {
        self.index(id).map(|index| {
            self.extents.start(index)
                + within
                    .max(0.0)
                    .min((self.extents.height(index) - 1.0).max(0.0))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refresh_and_reorder_keep_measurements_by_identity() {
        let mut cache = GeometryCache::default();
        cache.reset(800.0, [(10, "body-a", 200.0), (20, "body-b", 200.0)]);
        cache.measure(0, 14_000.0);
        cache.measure(1, 300.0);
        assert!(cache.replace(0, 10, "body-a", 100.0));
        assert_eq!(
            cache.extents.height(0),
            14_000.0,
            "same article body retains its real height"
        );
        cache.reset(
            800.0,
            [
                (30, "new", 400.0),
                (20, "body-b", 200.0),
                (10, "body-a", 200.0),
            ],
        );
        assert_eq!(cache.extents.start(2), 700.0);
        assert_eq!(cache.extents.height(2), 14_000.0);
        assert_eq!(cache.extents.total(), 14_700.0);
    }

    #[test]
    fn expansion_full_body_and_width_change_invalidate_only_relevant_geometry() {
        let mut cache = GeometryCache::default();
        cache.reset(800.0, [(10, 0, 28.0), (20, 0, 28.0)]);
        cache.measure(0, 28.0);
        cache.measure(1, 31.0);
        assert!(cache.replace(0, 10, 1, 4_000.0));
        assert_eq!(cache.extents.height(1), 31.0);
        cache.measure(0, 18_000.0);
        cache.reset(799.8, [(10, 1, 4_000.0), (20, 0, 28.0)]);
        assert_eq!(
            cache.extents.height(0),
            18_000.0,
            "subpixel noise does not invalidate"
        );
        cache.reset(390.0, [(10, 1, 8_000.0), (20, 0, 44.0)]);
        assert_eq!(cache.extents.height(0), 8_000.0);
        assert_eq!(cache.extents.height(1), 44.0);
        assert!(!cache.replace(0, 99, 1, 5_000.0));
    }

    #[test]
    fn anchor_survives_prepend_reorder_and_shrinking_content() {
        let mut cache = GeometryCache::default();
        cache.reset(800.0, [(10, 0, 28.0), (20, 0, 5_000.0)]);
        assert_eq!(cache.restore(20, 900.0), Some(928.0));
        cache.reset(800.0, [(30, 0, 3_000.0), (10, 0, 28.0), (20, 0, 5_000.0)]);
        assert_eq!(cache.restore(20, 900.0), Some(3_928.0));
        cache.reset(800.0, [(20, 0, 5_000.0), (30, 0, 3_000.0)]);
        assert_eq!(cache.restore(20, 900.0), Some(900.0));
        cache.replace(0, 20, 1, 28.0);
        assert_eq!(cache.restore(20, 900.0), Some(27.0));
        assert_eq!(
            cache.restore(10, 0.0),
            None,
            "deleted anchors never seek another article"
        );
    }

    #[test]
    fn far_seek_uses_each_article_height_not_the_last_realized_average() {
        let mut cache = GeometryCache::default();
        cache.reset(
            800.0,
            (0..100_000).map(|index| (index, 0, if index % 10 == 0 { 12_000.0 } else { 28.0 })),
        );
        let index = 99_999;
        let expected = 10_000.0 * 12_000.0 + 89_999.0 * 28.0;
        assert_eq!(cache.extents.start(index), expected);
        assert_eq!(cache.extents.at(expected + 12.0), Some((index, expected)));
        cache.measure(99_998, 40_000.0);
        assert_eq!(cache.extents.start(index), expected + 39_972.0);
        assert_eq!(
            cache.extents.height(0),
            12_000.0,
            "one measure never rewrites unrelated estimates"
        );
    }
}
