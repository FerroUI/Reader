use ferroui_controls::{ListBox, TextBox};
use ferroui_view::prelude::*;
use reader_domain::Id;
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
    ImportExport,
    Tags(Id),
    Note(Id),
    Rename(Id),
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
    pub search_input: NodeRef<TextBox>,
    pub article_list: NodeRef<ListBox>,
    pub collapsed: Signal<BTreeSet<String>>,
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
            search_input: NodeRef::new(),
            article_list: NodeRef::new(),
            collapsed: signal(BTreeSet::new()),
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
    }
    pub fn toggle_folder(&self, folder: &str) {
        self.collapsed.update(|folders| {
            if !folders.remove(folder) {
                folders.insert(folder.to_string());
            }
        });
    }
}
