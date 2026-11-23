//! One ordering snapshot is shared by every menu in a rendered tree. Commands
//! validate their captured siblings before applying a relative move.
use super::tree::TreeNode;
use crate::{menus, session::Session};
use ferroui_view::prelude::Element;
use reader_domain::{Id, SubscriptionNode, SubscriptionOrder, SubscriptionSort, View};
use reader_state::AppViewModel;
use std::{collections::HashMap, rc::Rc};

pub(crate) struct Context {
    pub order: SubscriptionOrder,
    generation: u64,
    urls: HashMap<Id, String>,
    neighbors: HashMap<TreeNode, (Option<SubscriptionNode>, Option<SubscriptionNode>)>,
}
impl Context {
    pub fn capture(vm: &AppViewModel) -> Rc<Self> {
        let order = vm.subscription_tree();
        let show_all = vm.state().settings.show_all_subscriptions;
        let mut neighbors = HashMap::new();
        let root = order
            .root
            .iter()
            .filter(|node| {
                show_all
                    || vm.count(&match node {
                        SubscriptionNode::Feed(id) => View::Feed(*id),
                        SubscriptionNode::Folder(name) => View::Folder(name.clone()),
                    }) > 0
            })
            .cloned()
            .collect::<Vec<_>>();
        add_neighbors(&mut neighbors, &root, "");
        for (folder, children) in &order.folders {
            let children = children
                .iter()
                .filter(|id| show_all || vm.count(&View::Feed(**id)) > 0)
                .map(|id| SubscriptionNode::Feed(*id))
                .collect::<Vec<_>>();
            add_neighbors(&mut neighbors, &children, folder);
        }
        Rc::new(Self {
            order,
            generation: vm.library_generation(),
            urls: vm
                .state()
                .feeds
                .iter()
                .filter(|feed| !feed.archived)
                .map(|feed| (feed.id, feed.url.clone()))
                .collect(),
            neighbors,
        })
    }

    fn valid(&self, vm: &AppViewModel, folder: &str) -> bool {
        if self.generation != vm.library_generation() {
            return false;
        }
        let current = vm.subscription_tree();
        if current.sort != self.order.sort {
            return false;
        }
        if folder.is_empty() {
            current.root == self.order.root
                && self.order.root.iter().all(|node| match node {
                    SubscriptionNode::Feed(id) => self.same_feed(vm, *id),
                    SubscriptionNode::Folder(_) => true,
                })
        } else {
            let Some(ids) = self.order.folders.get(folder) else {
                return false;
            };
            current.folders.get(folder) == Some(ids) && ids.iter().all(|id| self.same_feed(vm, *id))
        }
    }

    fn same_feed(&self, vm: &AppViewModel, id: Id) -> bool {
        vm.feed(id)
            .is_some_and(|feed| self.urls.get(&id) == Some(&feed.url))
    }
}

fn add_neighbors(
    neighbors: &mut HashMap<TreeNode, (Option<SubscriptionNode>, Option<SubscriptionNode>)>,
    siblings: &[SubscriptionNode],
    folder: &str,
) {
    for (index, node) in siblings.iter().enumerate() {
        let key = match node {
            SubscriptionNode::Feed(id) => TreeNode::Feed {
                id: *id,
                folder: folder.to_owned(),
            },
            SubscriptionNode::Folder(name) => TreeNode::Folder(name.clone()),
        };
        neighbors.insert(
            key,
            (
                index.checked_sub(1).and_then(|i| siblings.get(i)).cloned(),
                siblings.get(index + 1).cloned(),
            ),
        );
    }
}

pub(super) fn move_items(
    vm: Rc<AppViewModel>,
    ui: Rc<Session>,
    node: TreeNode,
    context: Rc<Context>,
) -> Vec<Element> {
    [true, false]
        .into_iter()
        .map(|before| {
            let target = context.neighbors.get(&node).and_then(|(up, down)| {
                if before {
                    up.clone()
                } else {
                    down.clone()
                }
            });
            let enabled = context.order.sort == SubscriptionSort::Manual && target.is_some();
            let vm = vm.clone();
            let ui = ui.clone();
            let node = node.clone();
            let context = context.clone();
            menus::item_enabled(
                if before { "Move up" } else { "Move down" },
                enabled,
                move || {
                    let folder = match &node {
                        TreeNode::Feed { folder, .. } => folder.as_str(),
                        _ => "",
                    };
                    if !enabled || !context.valid(&vm, folder) {
                        return;
                    }
                    let Some(target) = target.clone() else { return };
                    let changed = match &node {
                        TreeNode::Folder(name) => {
                            vm.position_root(SubscriptionNode::Folder(name.clone()), target, before)
                        }
                        TreeNode::Feed { id, folder } => vm.position_feed_occurrence(
                            *id,
                            folder.clone(),
                            folder.clone(),
                            Some(target),
                            before,
                        ),
                        TreeNode::Tag(_) => false,
                    };
                    if changed {
                        super::cursor::reveal(&ui);
                    }
                },
            )
        })
        .collect()
}

pub(super) fn sort_item(vm: Rc<AppViewModel>, folder: &str, context: Rc<Context>) -> Element {
    let folder = folder.to_owned();
    let enabled = context.order.sort == SubscriptionSort::Manual
        && context
            .order
            .folders
            .get(&folder)
            .is_some_and(|ids| ids.len() > 1);
    menus::item_enabled("Sort subscriptions alphabetically", enabled, move || {
        if enabled && context.valid(&vm, &folder) {
            vm.sort_subscriptions(Some(folder.clone()));
        }
    })
}
