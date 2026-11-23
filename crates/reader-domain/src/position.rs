use crate::{AppState, Id, SubscriptionNode, SubscriptionSort};

impl AppState {
    /// Restore the legacy document-order fallback without visiting articles.
    /// Sources added since the snapshot retain their relative order at the end.
    pub fn restore_feed_order(&mut self, order: &[Id]) -> bool {
        let ranks: std::collections::BTreeMap<_, _> = order
            .iter()
            .copied()
            .enumerate()
            .map(|(index, id)| (id, index))
            .collect();
        let previous: Vec<_> = self.feeds.iter().map(|feed| feed.id).collect();
        self.feeds
            .sort_by_key(|feed| ranks.get(&feed.id).copied().unwrap_or(usize::MAX));
        let changed = self.feeds.iter().map(|feed| feed.id).ne(previous);
        if changed {
            self.reindex_feeds();
        }
        changed
    }

    /// Compatibility command for callers without a captured source occurrence.
    /// An explicit folder uses its independent order; None changes only the
    /// legacy document-order fallback. New sidebar code uses occurrence moves.
    pub(crate) fn position_feed(
        &mut self,
        id: Id,
        target: Id,
        before: bool,
        folder: Option<String>,
    ) -> Result<(bool, bool), String> {
        let source_index = *self
            .feed_index
            .get(&id)
            .ok_or("This subscription no longer exists.")?;
        let target_index = *self
            .feed_index
            .get(&target)
            .ok_or("The drop target no longer exists.")?;
        if self.feeds[source_index].archived || self.feeds[target_index].archived {
            return Err("This subscription is no longer active.".into());
        }
        if id == target {
            return Ok((false, false));
        }
        let folder = folder.map(|name| name.trim().to_owned());
        if let Some(folder) = &folder {
            let target = &self.feeds[target_index];
            let valid = if folder.is_empty() {
                target.folder_names().is_empty()
            } else {
                target.in_folder(folder)
            };
            if !valid {
                return Err("The target subscription is no longer in this folder.".into());
            }
        }
        if let Some(folder) = folder {
            let source = &self.feeds[source_index];
            let same_group = if folder.is_empty() {
                source.folder_names().is_empty()
            } else {
                source.in_folder(&folder)
            };
            let membership = if same_group {
                false
            } else {
                self.set_feed_folders(id, vec![folder.clone()])?
            };
            let (_, order) = self.position_feed_occurrence(
                id,
                folder.clone(),
                folder,
                Some(SubscriptionNode::Feed(target)),
                before,
            )?;
            return Ok((membership, order));
        }
        if self.subscription_order.sort != SubscriptionSort::Manual {
            return Ok((false, false));
        }
        let mut destination = target_index + usize::from(!before);
        if source_index < destination {
            destination -= 1;
        }
        if source_index != destination {
            let source = self.feeds.remove(source_index);
            self.feeds.insert(destination, source);
            self.reindex_feeds();
        }
        Ok((false, source_index != destination))
    }
}
