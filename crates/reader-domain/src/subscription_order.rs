//! Optional tree ordering overlays. Missing metadata preserves the original
//! document-order feeds followed by alphabetically ordered folder projection.
use crate::{AppState, Id};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum SubscriptionSort {
    #[default]
    Manual,
    Alphabetical,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SubscriptionNode {
    Feed(Id),
    Folder(String),
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct SubscriptionOrder {
    pub sort: SubscriptionSort,
    pub root: Vec<SubscriptionNode>,
    pub folders: BTreeMap<String, Vec<Id>>,
}

impl SubscriptionOrder {
    pub fn is_default(&self) -> bool {
        self.sort == SubscriptionSort::Manual && self.root.is_empty() && self.folders.is_empty()
    }
}

impl AppState {
    /// A complete projection for rendering and keyboard traversal. The saved
    /// manual positions survive a temporary switch to alphabetical display.
    pub fn subscription_tree(&self) -> SubscriptionOrder {
        let mut tree = self.manual_subscription_tree();
        if tree.sort == SubscriptionSort::Alphabetical {
            self.sort_subscription_nodes(&mut tree.root);
            for ids in tree.folders.values_mut() {
                self.sort_subscription_ids(ids);
            }
        }
        tree
    }

    pub(crate) fn manual_subscription_tree(&self) -> SubscriptionOrder {
        let mut tree = self.default_subscription_tree();
        tree.sort = self.subscription_order.sort;
        tree.root = overlay(tree.root, &self.subscription_order.root);
        for (folder, ids) in &mut tree.folders {
            if let Some(order) = self.subscription_order.folders.get(folder) {
                *ids = overlay(std::mem::take(ids), order);
            }
        }
        tree
    }

    fn default_subscription_tree(&self) -> SubscriptionOrder {
        let mut tree = SubscriptionOrder::default();
        for folder in &self.folders {
            tree.folders.entry(folder.clone()).or_default();
        }
        for feed in self.feeds.iter().filter(|feed| !feed.archived) {
            let folders = feed.folder_names();
            if folders.is_empty() {
                tree.root.push(SubscriptionNode::Feed(feed.id));
            }
            for folder in folders {
                tree.folders.entry(folder).or_default().push(feed.id);
            }
        }
        tree.root
            .extend(tree.folders.keys().cloned().map(SubscriptionNode::Folder));
        tree
    }

    /// Remove invalid references without filling sparse overrides. New sources
    /// and memberships append through the default projection, never replacing
    /// another occurrence's manual position.
    pub fn normalize_subscription_order(&mut self) -> bool {
        if self.subscription_order.root.is_empty() && self.subscription_order.folders.is_empty() {
            return false;
        }
        let tree = self.default_subscription_tree();
        let root: BTreeSet<_> = tree.root.into_iter().collect();
        let mut seen = BTreeSet::new();
        let size = |order: &SubscriptionOrder| {
            order.root.len()
                + order.folders.len()
                + order.folders.values().map(Vec::len).sum::<usize>()
        };
        let previous = size(&self.subscription_order);
        self.subscription_order
            .root
            .retain(|node| root.contains(node) && seen.insert(node.clone()));
        self.subscription_order.folders.retain(|folder, ids| {
            let Some(members) = tree.folders.get(folder) else {
                return false;
            };
            let members: BTreeSet<_> = members.iter().copied().collect();
            let mut seen = BTreeSet::new();
            ids.retain(|id| members.contains(id) && seen.insert(*id));
            !ids.is_empty()
        });
        previous != size(&self.subscription_order)
    }

    pub(crate) fn rename_subscription_order(&mut self, from: &str, to: &str) -> bool {
        let mut changed = false;
        for node in &mut self.subscription_order.root {
            if let SubscriptionNode::Folder(name) = node {
                if name == from {
                    *name = to.to_owned();
                    changed = true;
                }
            }
        }
        if let Some(ids) = self.subscription_order.folders.remove(from) {
            self.subscription_order.folders.insert(to.to_owned(), ids);
            changed = true;
        }
        changed
    }

    pub(crate) fn sort_subscription_nodes(&self, nodes: &mut [SubscriptionNode]) {
        nodes.sort_by_cached_key(|node| match node {
            SubscriptionNode::Feed(id) => (
                1,
                self.feed(*id)
                    .map(|feed| feed.title.to_lowercase())
                    .unwrap_or_default(),
            ),
            SubscriptionNode::Folder(name) => (0, name.to_lowercase()),
        });
    }

    pub(crate) fn sort_subscription_ids(&self, ids: &mut [Id]) {
        ids.sort_by_cached_key(|id| {
            self.feed(*id)
                .map(|feed| feed.title.to_lowercase())
                .unwrap_or_default()
        });
    }
}

fn overlay<T: Clone + Ord>(default: Vec<T>, saved: &[T]) -> Vec<T> {
    let mut remaining: BTreeSet<_> = default.iter().cloned().collect();
    let mut result = Vec::with_capacity(default.len());
    result.extend(saved.iter().filter(|item| remaining.remove(*item)).cloned());
    result.extend(default.into_iter().filter(|item| remaining.remove(item)));
    result
}
