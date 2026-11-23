use crate::{AppState, Id};

impl AppState {
    /// Sidebar projections share a persisted subscription order. Validate every
    /// identity and group before mutating either the order or folder membership.
    pub(crate) fn position_feed(
        &mut self,
        id: Id,
        target: Id,
        before: bool,
        folder: Option<String>,
    ) -> Result<bool, String> {
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
            return Ok(false);
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
        let mut destination = target_index + usize::from(!before);
        if source_index < destination {
            destination -= 1;
        }
        let membership_changed = if let Some(folder) = folder {
            let source = &self.feeds[source_index];
            let same_group = if folder.is_empty() {
                source.folder_names().is_empty()
            } else {
                source.in_folder(&folder)
            };
            if same_group {
                false
            } else {
                self.set_feed_folders(id, vec![folder])?
            }
        } else {
            false
        };
        if source_index != destination {
            let source = self.feeds.remove(source_index);
            self.feeds.insert(destination, source);
            self.reindex_feeds();
        }
        Ok(membership_changed || source_index != destination)
    }
}
