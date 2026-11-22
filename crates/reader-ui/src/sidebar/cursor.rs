//! Keyboard navigation over the same visible occurrences rendered by the sidebar.
use super::tree::{self, TreeNode};
use crate::session::Session;
use ferroui_base::threading::{Dispatcher, DispatcherPriority};
use ferroui_controls::Control;
use reader_domain::View;
use reader_state::AppViewModel;

pub(crate) fn visible(vm: &AppViewModel, ui: &Session) -> Vec<TreeNode> {
    let feeds = vm.feeds();
    let show_all = vm.state().settings.show_all_subscriptions;
    let include = |id| show_all || vm.count(&View::Feed(id)) > 0;
    let mut nodes: Vec<_> = feeds
        .iter()
        .filter(|feed| feed.folder_names().is_empty() && include(feed.id))
        .map(|feed| TreeNode::Feed {
            id: feed.id,
            folder: String::new(),
        })
        .collect();
    let collapsed = ui.collapsed.get_untracked();
    for folder in vm.folders() {
        let children: Vec<_> = feeds
            .iter()
            .filter(|feed| feed.in_folder(&folder) && include(feed.id))
            .collect();
        if !show_all && children.is_empty() {
            continue;
        }
        nodes.push(TreeNode::Folder(folder.clone()));
        if !collapsed.contains(&folder) {
            nodes.extend(children.into_iter().map(|feed| TreeNode::Feed {
                id: feed.id,
                folder: folder.clone(),
            }));
        }
    }
    nodes.extend(
        vm.tags()
            .into_iter()
            .filter(|tag| show_all || vm.count(&View::Tag(tag.clone())) > 0)
            .map(TreeNode::Tag),
    );
    nodes
}

pub(crate) fn current(nodes: &[TreeNode], ui: &Session) -> Option<TreeNode> {
    tree::resolve(nodes, ui.tree_cursor.get().as_ref())
}

fn view(node: &TreeNode) -> View {
    match node {
        TreeNode::Feed { id, .. } => View::Feed(*id),
        TreeNode::Folder(name) => View::Folder(name.clone()),
        TreeNode::Tag(name) => View::Tag(name.clone()),
    }
}

fn opened(vm: &AppViewModel, ui: &Session, nodes: &[TreeNode]) -> Option<TreeNode> {
    let route = vm.view();
    let remembered = ui.tree_opened.borrow();
    if let Some(node) = remembered.as_ref().filter(|node| view(node) == route) {
        if let Some(node) = tree::resolve(nodes, Some(node)) {
            return Some(node);
        }
    }
    nodes.iter().find(|node| view(node) == route).cloned()
}

pub(crate) fn move_cursor(vm: &AppViewModel, ui: &Session, forward: bool) {
    let nodes = visible(vm, ui);
    let opened = opened(vm, ui, &nodes);
    let next = tree::step(
        &nodes,
        ui.tree_cursor.get_untracked().as_ref(),
        opened.as_ref(),
        forward,
    );
    ui.tree_cursor.set(next.clone());
    if next.is_some() {
        if vm.viewport() < 700.0 {
            vm.set_sidebar_open(true);
        }
        reveal(ui);
    }
}

pub(crate) fn open_cursor(vm: &AppViewModel, ui: &Session) {
    if let Some(node) = current(&visible(vm, ui), ui) {
        open(vm, ui, node);
        ui.restore_article_focus();
    }
}

pub(crate) fn open(vm: &AppViewModel, ui: &Session, node: TreeNode) {
    // Pointer navigation follows an already active keyboard cursor, as Reader did.
    if ui.tree_cursor.get_untracked().is_some() {
        ui.tree_cursor.set(Some(node.clone()));
    }
    *ui.tree_opened.borrow_mut() = Some(node.clone());
    let drawer_open = vm.sidebar_open();
    let target = view(&node);
    if vm.view() == target {
        vm.refresh();
    } else {
        vm.navigate(target);
    }
    if drawer_open {
        vm.set_sidebar_open(false);
        ui.restore_article_focus();
    }
}

pub(crate) fn toggle(vm: &AppViewModel, ui: &Session) {
    let nodes = visible(vm, ui);
    let node = current(&nodes, ui).or_else(|| opened(vm, ui, &nodes));
    let folder = match node {
        Some(TreeNode::Folder(folder)) => Some(folder),
        Some(TreeNode::Feed { folder, .. }) if !folder.is_empty() => Some(folder),
        _ => None,
    };
    if let Some(folder) =
        folder.filter(|folder| vm.feeds().iter().any(|feed| feed.in_folder(folder)))
    {
        ui.toggle_folder(&folder);
        let next = current(&visible(vm, ui), ui);
        ui.tree_cursor.set(next);
        reveal(ui);
    }
}

fn reveal(ui: &Session) {
    let scroll = ui.navigation_scroll;
    Dispatcher::ui_thread().post_local(
        move || {
            if let Some(scroll) = scroll
                .get_untracked()
                .filter(|s| s.is_attached_to_visual_tree())
            {
                if let Some(control) = scroll
                    .get_visual_descendants()
                    .find(|v| v.classes().contains("keyboard-cursor"))
                    .and_then(|v| v.cast::<Control>())
                {
                    control.bring_into_view();
                }
            }
        },
        DispatcherPriority::BACKGROUND,
    );
}
