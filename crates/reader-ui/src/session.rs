use ferroui_controls::{ListBox, TextBox};
use ferroui_view::prelude::*;
use reader_domain::{Id, View};
use std::{
    cell::{Cell, RefCell},
    collections::BTreeSet,
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
    Tags(Id),
    Note(Id),
    Rename(Id),
    RenameFolder(String),
    Move(Id),
    Unsubscribe(Id),
    NewFolder,
    MarkAll,
    About,
}
/// Transient presentation state is separate from the persisted feed document.
pub struct Session {
    pub panel: Signal<Panel>,
    pub search: Signal<String>,
    pub search_scope: Signal<View>,
    pub last_view: RefCell<Option<View>>,
    pub search_input: NodeRef<TextBox>,
    pub article_list: NodeRef<ListBox>,
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
    pub field: NodeRef<TextBox>,
    pub form_for: RefCell<Option<Panel>>,
    pub subscription_filter: Signal<String>,
}
impl Session {
    pub fn new() -> Self {
        Self {
            panel: signal(Panel::None),
            search: signal(String::new()),
            search_scope: signal(View::All),
            last_view: RefCell::new(None),
            search_input: NodeRef::new(),
            article_list: NodeRef::new(),
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
            field: NodeRef::new(),
            form_for: RefCell::new(None),
            subscription_filter: signal(String::new()),
        }
    }
    pub fn show(&self, panel: Panel) {
        self.draft.borrow_mut().clear();
        *self.form_for.borrow_mut() = None;
        self.panel.set(panel);
    }
    pub fn close(&self) {
        self.panel.set(Panel::None);
        let list = self.article_list;
        ferroui_base::threading::Dispatcher::ui_thread().post_local(
            move || {
                if let Some(list) = list.get_untracked() {
                    list.focus();
                }
            },
            ferroui_base::threading::DispatcherPriority::BACKGROUND,
        );
    }
    pub fn toggle_folder(&self, folder: &str) {
        self.collapsed.update(|folders| {
            if !folders.remove(folder) {
                folders.insert(folder.to_string());
            }
        });
    }
}
