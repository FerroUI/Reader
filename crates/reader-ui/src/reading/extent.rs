//! Measured pixel offsets for non-uniform rows. Updates and offset lookups are
//! logarithmic; scrolling never scans the feed or averages the current window.
#[derive(Default)]
pub(super) struct Extents {
    heights: Vec<f64>,
    tree: Vec<f64>,
}

impl Extents {
    pub fn reset(&mut self, heights: impl IntoIterator<Item = f64>) {
        self.heights = heights.into_iter().map(valid_height).collect();
        self.tree = vec![0.0; self.heights.len() + 1];
        // Linear Fenwick construction, including large import/reorder batches.
        for index in 1..self.tree.len() {
            self.tree[index] += self.heights[index - 1];
            let parent = index + (index & index.wrapping_neg());
            if parent < self.tree.len() {
                self.tree[parent] += self.tree[index];
            }
        }
    }

    pub fn len(&self) -> usize {
        self.heights.len()
    }

    pub fn height(&self, index: usize) -> f64 {
        self.heights.get(index).copied().unwrap_or(0.0)
    }

    pub fn start(&self, index: usize) -> f64 {
        let mut index = index.min(self.len());
        let mut result = 0.0;
        while index > 0 {
            result += self.tree[index];
            index &= index - 1;
        }
        result
    }

    pub fn total(&self) -> f64 {
        self.start(self.len())
    }

    pub fn measure(&mut self, index: usize, height: f64) {
        let Some(previous) = self.heights.get_mut(index) else {
            return;
        };
        let height = valid_height(height);
        let delta = height - *previous;
        if delta.abs() < 0.01 {
            return;
        }
        *previous = height;
        let mut index = index + 1;
        while index < self.tree.len() {
            self.tree[index] += delta;
            index += index & index.wrapping_neg();
        }
    }

    /// Finds the row containing this pixel. Exact boundaries belong to the next
    /// row; outside offsets clamp to the closest row, including overscroll.
    pub fn at(&self, position: f64) -> Option<(usize, f64)> {
        if self.heights.is_empty() {
            return None;
        }
        let position = if position.is_nan() {
            0.0
        } else {
            position.max(0.0)
        };
        let mut index = 0;
        let mut prefix = 0.0;
        let mut step = 1usize << (usize::BITS - self.len().leading_zeros() - 1);
        while step != 0 {
            let next = index + step;
            if next < self.tree.len() && prefix + self.tree[next] <= position {
                prefix += self.tree[next];
                index = next;
            }
            step >>= 1;
        }
        if index == self.len() {
            index -= 1;
            prefix = self.start(index);
        }
        Some((index, prefix))
    }
}

fn valid_height(height: f64) -> f64 {
    if height.is_finite() {
        height.max(1.0)
    } else {
        1.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mixed_articles_keep_exact_offsets_when_a_large_body_expands() {
        let mut extents = Extents::default();
        extents.reset([28.0, 18_000.0, 38.0, 4_000.0, 28.0]);
        assert_eq!(extents.at(18_028.0), Some((2, 18_028.0)));
        extents.measure(1, 35_000.0);
        assert_eq!(extents.start(2), 35_028.0);
        assert_eq!(extents.total(), 39_094.0);
        assert_eq!(extents.at(34_999.0), Some((1, 28.0)));
        extents.measure(1, 28.0);
        assert_eq!(extents.at(56.0), Some((2, 56.0)));
    }

    #[test]
    fn all_boundaries_and_updates_match_a_linear_reference() {
        let mut extents = Extents::default();
        let mut reference: Vec<_> = (0..16_384)
            .map(|n| 27.0 + ((n * 7919) % 37_000) as f64)
            .collect();
        extents.reset(reference.iter().copied());
        for index in (0..reference.len()).step_by(97) {
            reference[index] = 31.0 + (index % 19) as f64;
            extents.measure(index, reference[index]);
        }
        let mut start = 0.0;
        for (index, height) in reference.into_iter().enumerate() {
            assert_eq!(extents.start(index), start);
            assert_eq!(extents.at(start), Some((index, start)));
            assert_eq!(extents.at(start + height - 0.5), Some((index, start)));
            start += height;
        }
        assert_eq!(extents.total(), start);
    }

    #[test]
    fn empty_overscroll_and_invalid_measurements_remain_finite() {
        let mut extents = Extents::default();
        assert_eq!(extents.at(0.0), None);
        extents.reset([f64::NAN, f64::INFINITY, -1.0, 27.0]);
        assert_eq!(extents.total(), 30.0);
        assert_eq!(extents.at(-500.0), Some((0, 0.0)));
        assert_eq!(extents.at(f64::NAN), Some((0, 0.0)));
        assert_eq!(extents.at(f64::INFINITY), Some((3, 3.0)));
        assert_eq!(extents.at(30.0), Some((3, 3.0)));
        extents.measure(99, 800.0);
        assert_eq!(extents.total(), 30.0);
    }
}
