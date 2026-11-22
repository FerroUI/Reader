use super::Row;
use ferroui_base::{collections::FerroList, BoxedValue};
use std::collections::HashSet;
/// Common mutations patch only changed rows; reordered sources preserve recycled controls.
pub(super) fn reconcile(
    items: &FerroList<Option<BoxedValue>>,
    mut previous: Vec<Row>,
    next: &[Row],
) {
    if previous.len() == next.len()
        && previous
            .iter()
            .zip(next)
            .all(|(a, b)| a.article.id == b.article.id)
    {
        for (index, (old, row)) in previous.iter().zip(next).enumerate() {
            if old != row {
                items.set(index, row.boxed());
            }
        }
        return;
    }
    let retained: HashSet<_> = next.iter().map(|r| r.article.id).collect();
    let mut end = previous.len();
    while end > 0 {
        if retained.contains(&previous[end - 1].article.id) {
            end -= 1;
            continue;
        }
        let mut start = end - 1;
        while start > 0 && !retained.contains(&previous[start - 1].article.id) {
            start -= 1;
        }
        previous.drain(start..end);
        items.remove_range(start, end - start);
        end = start;
    }
    if previous.is_empty() {
        items.add_range(next.iter().map(Row::boxed));
        return;
    }
    let displaced = previous
        .iter()
        .zip(next)
        .filter(|(a, b)| a.article.id != b.article.id)
        .take(65)
        .count();
    if next.len().max(previous.len()) > 256 && displaced > 64 {
        items.clear();
        items.add_range(next.iter().map(Row::boxed));
        return;
    }
    for (index, row) in next.iter().enumerate() {
        if previous
            .get(index)
            .is_some_and(|r| r.article.id == row.article.id)
        {
            if previous[index] != *row {
                previous[index] = row.clone();
                items.set(index, row.boxed());
            }
        } else if let Some(position) = previous
            .iter()
            .enumerate()
            .skip(index)
            .find_map(|(i, r)| (r.article.id == row.article.id).then_some(i))
        {
            let old = previous.remove(position);
            previous.insert(index, old);
            items.move_item(position, index);
            if previous[index] != *row {
                previous[index] = row.clone();
                items.set(index, row.boxed());
            }
        } else {
            previous.insert(index, row.clone());
            items.insert(index, row.boxed());
        }
    }
    if previous.len() > next.len() {
        items.remove_range(next.len(), previous.len() - next.len());
    }
}
