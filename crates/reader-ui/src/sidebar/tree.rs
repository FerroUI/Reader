//! Stable occurrences in Reader's subscription tree, independent of the open stream.
//! A feed may appear under several folders; its occurrence includes that parent.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum TreeNode {
    Feed { id: u64, folder: String },
    Folder(String),
    Tag(String),
}

impl TreeNode {
    pub(crate) fn parent(&self) -> Option<Self> {
        match self {
            Self::Feed { folder, .. } if !folder.is_empty() => Some(Self::Folder(folder.clone())),
            _ => None,
        }
    }
}

/// Rebuilding a collapsed/filtered tree retains the nearest surviving ancestor.
pub(crate) fn resolve(nodes: &[TreeNode], current: Option<&TreeNode>) -> Option<TreeNode> {
    let mut candidate = current.cloned();
    while let Some(node) = candidate {
        if nodes.contains(&node) {
            return Some(node);
        }
        candidate = node.parent();
    }
    None
}

/// No wrapping. With no cursor, move relative to the opened occurrence. With
/// neither, next selects the first node and previous leaves the cursor unset.
pub(crate) fn step(
    nodes: &[TreeNode],
    cursor: Option<&TreeNode>,
    opened: Option<&TreeNode>,
    forward: bool,
) -> Option<TreeNode> {
    let cursor = resolve(nodes, cursor);
    let start = cursor.clone().or_else(|| resolve(nodes, opened));
    let index = start
        .as_ref()
        .and_then(|node| nodes.iter().position(|n| n == node));
    let next = match (index, forward) {
        (Some(index), true) => index.checked_add(1),
        (Some(index), false) => index.checked_sub(1),
        (None, true) => Some(0),
        (None, false) => None,
    };
    next.and_then(|index| nodes.get(index)).cloned().or(cursor)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn feed(id: u64, folder: &str) -> TreeNode {
        TreeNode::Feed {
            id,
            folder: folder.into(),
        }
    }
    fn nodes() -> Vec<TreeNode> {
        vec![
            feed(1, ""),
            TreeNode::Folder("News".into()),
            feed(2, "News"),
            TreeNode::Folder("Tech".into()),
            feed(2, "Tech"),
            TreeNode::Tag("saved".into()),
        ]
    }
    #[test]
    fn starts_only_forward_without_an_open_subscription() {
        let n = nodes();
        assert_eq!(step(&n, None, None, true), Some(n[0].clone()));
        assert_eq!(step(&n, None, None, false), None);
        assert_eq!(step(&[], None, None, true), None);
    }
    #[test]
    fn opened_occurrence_anchors_initial_movement_without_becoming_cursor_at_boundary() {
        let n = nodes();
        assert_eq!(step(&n, None, Some(&n[2]), true), Some(n[3].clone()));
        assert_eq!(step(&n, None, Some(&n[2]), false), Some(n[1].clone()));
        assert_eq!(step(&n, None, n.last(), true), None);
    }
    #[test]
    fn cursor_is_independent_of_route_and_never_wraps() {
        let n = nodes();
        assert_eq!(step(&n, Some(&n[0]), Some(&n[4]), true), Some(n[1].clone()));
        assert_eq!(step(&n, Some(&n[0]), None, false), Some(n[0].clone()));
        assert_eq!(step(&n, n.last(), None, true), n.last().cloned());
    }
    #[test]
    fn collapsing_parent_preserves_occurrence_ancestry() {
        let mut n = nodes();
        let cursor = n.remove(2);
        assert_eq!(
            resolve(&n, Some(&cursor)),
            Some(TreeNode::Folder("News".into()))
        );
        // The same feed in another folder must not steal the cursor.
        n.retain(|node| node != &TreeNode::Folder("News".into()));
        assert_eq!(resolve(&n, Some(&cursor)), None);
    }
    #[test]
    fn filtered_or_removed_nodes_cannot_be_opened_from_stale_cursor() {
        let n = vec![TreeNode::Tag("saved".into())];
        assert_eq!(resolve(&n, Some(&feed(99, "gone"))), None);
        assert_eq!(
            step(&n, Some(&feed(99, "gone")), None, true),
            Some(n[0].clone())
        );
    }
}
