use crate::sidebar::tree::TreeNode;
use ferroui_controls::{Button, ListBox, ScrollViewer, TextBox};
use ferroui_view::prelude::*;
use reader_domain::{Id, View};
use std::{
    cell::{Cell, RefCell},
    collections::BTreeSet,
    rc::Rc,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Panel {
    None,
    Subscribe,
    Shortcuts,
    Preferences,
    Subscriptions,
    GoToSubscription,
    GoToTag,
    ImportExport,
    SendToSettings,
    EditSendTo(Option<u64>),
    SendToMenu(Id),
    Tags(Id),
    Note(Id),
    Rename(Id),
    RenameFolder(String),
    Move(Id),
    Unsubscribe(Id),
    NewFolder,
    MarkAll(Option<i64>),
    About,
}
/// Transient presentation state is separate from the persisted feed document.
pub struct Session {
    pub panel: Signal<Panel>,
    pub search: Signal<String>,
    pub search_scope: Signal<View>,
    pub last_view: RefCell<Option<View>>,
    pub last_library_generation: Cell<u64>,
    pub search_input: NodeRef<TextBox>,
    pub article_list: NodeRef<ListBox>,
    pub page_scroll: NodeRef<ScrollViewer>,
    pub drawer_close: NodeRef<Button>,
    pub subscribe_button: NodeRef<Button>,
    pub navigation_scroll: NodeRef<ScrollViewer>,
    pub tree_cursor: Signal<Option<TreeNode>>,
    pub tree_opened: RefCell<Option<TreeNode>>,
    pub skip_mark_all_confirmation: Signal<bool>,
    pub collapsed: Signal<BTreeSet<String>>,
    pub all_open: Signal<bool>,
    pub explore_open: Signal<bool>,
    pub focus_mode: Signal<bool>,
    pub desktop_sidebar: Signal<bool>,
    pub prefix: Cell<bool>,
    pub prefix_at: Cell<Option<std::time::Instant>>,
    pub draft: RefCell<String>,
    pub programmatic_scroll: Cell<Option<std::time::Instant>>,
    pub entry: Signal<String>,
    pub feed_connection: Signal<reader_domain::FeedConnection>,
    pub send_to_url: Signal<String>,
    pub send_to_enabled: Signal<bool>,
    pub field: NodeRef<TextBox>,
    pub form_for: RefCell<Option<Panel>>,
    pub subscription_filter: Signal<String>,
    pub form_error: Signal<String>,
    focus_epoch: Rc<Cell<u64>>,
}
impl Session {
    pub fn new() -> Self {
        Self {
            panel: signal(Panel::None),
            search: signal(String::new()),
            search_scope: signal(View::All),
            last_view: RefCell::new(None),
            last_library_generation: Cell::new(0),
            search_input: NodeRef::new(),
            article_list: NodeRef::new(),
            page_scroll: NodeRef::new(),
            drawer_close: NodeRef::new(),
            subscribe_button: NodeRef::new(),
            navigation_scroll: NodeRef::new(),
            tree_cursor: signal(None),
            tree_opened: RefCell::new(None),
            skip_mark_all_confirmation: signal(false),
            collapsed: signal(BTreeSet::new()),
            all_open: signal(false),
            explore_open: signal(false),
            focus_mode: signal(false),
            desktop_sidebar: signal(true),
            prefix: Cell::new(false),
            prefix_at: Cell::new(None),
            draft: RefCell::new(String::new()),
            programmatic_scroll: Cell::new(None),
            entry: signal(String::new()),
            feed_connection: signal(reader_domain::FeedConnection::Automatic),
            send_to_url: signal(String::new()),
            send_to_enabled: signal(true),
            field: NodeRef::new(),
            form_for: RefCell::new(None),
            subscription_filter: signal(String::new()),
            form_error: signal(String::new()),
            focus_epoch: Rc::new(Cell::new(0)),
        }
    }
    pub fn show(&self, panel: Panel) {
        self.cancel_prefix();
        self.focus_epoch.set(self.focus_epoch.get().wrapping_add(1));
        crate::gestures::cancel_active();
        self.draft.borrow_mut().clear();
        self.skip_mark_all_confirmation.set(false);
        self.form_error.set(String::new());
        *self.form_for.borrow_mut() = None;
        self.panel.set(panel);
        crate::transient::close_menus();
        crate::notify_navigation();
    }
    pub fn close(&self) {
        crate::gestures::cancel_active();
        self.cancel_prefix();
        self.form_error.set(String::new());
        self.panel.set(Panel::None);
        self.restore_article_focus();
        crate::notify_navigation();
    }
    pub fn cancel_prefix(&self) {
        self.prefix.set(false);
        self.prefix_at.set(None);
    }
    /// A deferred close cannot steal focus from a dialog opened in the same frame.
    pub fn restore_article_focus(&self) {
        let epoch = self.focus_epoch.get().wrapping_add(1);
        self.focus_epoch.set(epoch);
        let current = self.focus_epoch.clone();
        let panel = self.panel;
        let list = self.article_list;
        let drawer = self.drawer_close;
        ferroui_base::threading::Dispatcher::ui_thread().post_local(
            move || {
                if current.get() != epoch || panel.get_untracked() != Panel::None {
                    return;
                }
                if let Some(close) = drawer
                    .get_untracked()
                    .filter(|button| button.is_attached_to_visual_tree())
                {
                    close.focus();
                    return;
                }
                if let Some(list) = list
                    .get_untracked()
                    .filter(|list| list.is_attached_to_visual_tree())
                {
                    list.focus();
                }
            },
            ferroui_base::threading::DispatcherPriority::BACKGROUND,
        );
    }
    pub fn toggle_folder(&self, folder: &str) {
        if !self.collapsed.get_untracked().contains(folder) {
            if matches!(self.tree_cursor.get_untracked(), Some(TreeNode::Feed { folder: ref parent, .. }) if parent == folder)
            {
                self.tree_cursor
                    .set(Some(TreeNode::Folder(folder.to_string())));
            }
        }
        self.collapsed.update(|folders| {
            if !folders.remove(folder) {
                folders.insert(folder.to_string());
            }
        });
    }
}
