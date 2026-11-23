use crate::{AppState, Id, SubscriptionNode, SubscriptionSort};

impl AppState {
    pub(crate) fn position_root(
        &mut self,
        item: SubscriptionNode,
        target: SubscriptionNode,
        before: bool,
    ) -> Result<bool, String> {
        let mut root = self.manual_subscription_tree().root;
        if !root.contains(&item) || !root.contains(&target) {
            return Err("This subscription-tree item is no longer at the root.".into());
        }
        if self.subscription_order.sort != SubscriptionSort::Manual {
            return Ok(false);
        }
        if position(&mut root, &item, Some(&target), before) {
            self.subscription_order.root = root;
            return Ok(true);
        }
        Ok(false)
    }

    /// Moves one visible occurrence. Folder-to-folder removes only the source
    /// label; other labels and their positions remain independent. A move to
    /// root removes that label, so a still-labelled source does not gain a root
    /// occurrence. Validate source, destination and anchor before any mutation.
    pub(crate) fn position_feed_occurrence(
        &mut self,
        id: Id,
        from_folder: String,
        to_folder: String,
        target: Option<SubscriptionNode>,
        before: bool,
    ) -> Result<(bool, bool), String> {
        self.validate_feed_occurrence(id, &from_folder)?;
        let folders = self.folders();
        if !to_folder.is_empty() && !folders.contains(&to_folder) {
            return Err("The destination folder no longer exists.".into());
        }
        if let Some(target) = &target {
            match target {
                SubscriptionNode::Feed(target) => {
                    self.validate_feed_occurrence(*target, &to_folder)?
                }
                SubscriptionNode::Folder(name)
                    if to_folder.is_empty() && folders.contains(name) => {}
                _ => return Err("The drop target is no longer in this folder.".into()),
            }
        }
        let feed = self.feed(id).unwrap();
        let was_member = if to_folder.is_empty() {
            feed.folder_names().is_empty()
        } else {
            feed.in_folder(&to_folder)
        };
        let membership = if from_folder != to_folder {
            let mut labels = feed.folder_names();
            labels.retain(|name| name != &from_folder);
            if !to_folder.is_empty() && !labels.contains(&to_folder) {
                labels.push(to_folder.clone());
            }
            self.set_feed_folders(id, labels)?
        } else {
            false
        };
        if self.subscription_order.sort != SubscriptionSort::Manual
            || (to_folder.is_empty() && !self.feed(id).unwrap().folder_names().is_empty())
            || (from_folder != to_folder && was_member && target.is_none())
        {
            return Ok((membership, false));
        }
        let mut tree = self.manual_subscription_tree();
        let changed = if to_folder.is_empty() {
            let changed = position(
                &mut tree.root,
                &SubscriptionNode::Feed(id),
                target.as_ref(),
                before,
            );
            if changed {
                self.subscription_order.root = tree.root;
            }
            changed
        } else {
            let ids = tree.folders.get_mut(&to_folder).unwrap();
            let target = match target {
                Some(SubscriptionNode::Feed(id)) => Some(id),
                _ => None,
            };
            let changed = position(ids, &id, target.as_ref(), before);
            if changed {
                self.subscription_order
                    .folders
                    .insert(to_folder, std::mem::take(ids));
            }
            changed
        };
        Ok((membership, changed))
    }

    pub(crate) fn sort_subscriptions(&mut self, folder: Option<String>) -> Result<bool, String> {
        let mut tree = self.manual_subscription_tree();
        if folder
            .as_ref()
            .is_some_and(|name| !tree.folders.contains_key(name))
        {
            return Err("This folder no longer exists.".into());
        }
        if self.subscription_order.sort != SubscriptionSort::Manual {
            return Ok(false);
        }
        if let Some(folder) = folder {
            let ids = tree.folders.get_mut(&folder).unwrap();
            let previous = ids.clone();
            self.sort_subscription_ids(ids);
            if previous == *ids {
                return Ok(false);
            }
            self.subscription_order
                .folders
                .insert(folder, std::mem::take(ids));
        } else {
            let previous = tree.root.clone();
            self.sort_subscription_nodes(&mut tree.root);
            if previous == tree.root {
                return Ok(false);
            }
            self.subscription_order.root = tree.root;
        }
        Ok(true)
    }

    fn validate_feed_occurrence(&self, id: Id, folder: &str) -> Result<(), String> {
        let feed = self
            .feed(id)
            .filter(|feed| !feed.archived)
            .ok_or("This subscription no longer exists.")?;
        let valid = if folder.is_empty() {
            feed.folder_names().is_empty()
        } else {
            feed.in_folder(folder)
        };
        if !valid {
            return Err("This subscription is no longer in the captured folder.".into());
        }
        Ok(())
    }
}

/// Source and optional anchor have already been validated in the same scope.
fn position<T: Clone + PartialEq>(
    items: &mut Vec<T>,
    item: &T,
    target: Option<&T>,
    before: bool,
) -> bool {
    if target == Some(item) {
        return false;
    }
    let source = items.iter().position(|node| node == item).unwrap();
    let mut destination = target.map_or(items.len(), |target| {
        items.iter().position(|node| node == target).unwrap() + usize::from(!before)
    });
    if source < destination {
        destination -= 1;
    }
    if source == destination {
        return false;
    }
    let item = items.remove(source);
    items.insert(destination, item);
    true
}
