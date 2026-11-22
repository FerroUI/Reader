//! ReactiveUI MVVM application state with incremental DynamicData article caches.
mod article_content;
mod commands;
mod connection;
mod counts;
mod history;
mod navigation;
mod persistence;
mod presentation;
mod query;
mod services;
mod subscriptions;

use ferroui_dynamic_data::SourceCache;
use ferroui_reactive::{ObservableExt, concurrency::ImmediateScheduler};
use ferroui_reactiveui::{IReactiveObjectExt, ReactiveCommand, ReactiveObject, reactive_object};
use reader_domain::{Action, AppState, Article, Id, View};
pub use services::{HostRequest, HostServices, SaveCallback};
use std::{
    cell::{Cell, Ref, RefCell},
    collections::{BTreeSet, VecDeque},
    rc::Rc,
};

pub struct AppViewModel {
    reactive: ReactiveObject,
    revision: RefCell<u64>,
    document: RefCell<AppState>,
    library_generation: Cell<u64>,
    article_loads: article_content::ArticleLoads,
    now: Cell<i64>,
    view: RefCell<View>,
    selected: Cell<Option<Id>>,
    cursor: Cell<Option<Id>>,
    kept_unread: RefCell<BTreeSet<Id>>,
    viewport: Cell<f64>,
    viewport_height: Cell<f64>,
    touch_mode: Cell<bool>,
    sidebar: Cell<bool>,
    status: RefCell<String>,
    refreshing: Cell<bool>,
    dirty: Cell<bool>,
    applying: Cell<bool>,
    pending: RefCell<VecDeque<Action>>,
    undo: RefCell<VecDeque<history::UndoEntry>>,
    visible_ids: RefCell<Vec<Id>>,
    visible_set: RefCell<BTreeSet<Id>>,
    counts: RefCell<counts::CountIndex>,
    services: RefCell<HostServices>,
    has_services: Cell<bool>,
    save: SaveCallback,
    pub articles: SourceCache<Rc<Article>, Id>,
    pub visible: SourceCache<Rc<Article>, Id>,
    pub execute: ReactiveCommand<Option<Action>, ()>,
}

reactive_object!(AppViewModel);

impl AppViewModel {
    pub fn new(mut state: AppState, now: i64, save: SaveCallback) -> Rc<Self> {
        // Constructors and parsers produce valid documents; reject malformed caller state visibly.
        let error = state.normalize().err();
        if error.is_some() {
            state = AppState::new();
        }
        let start_page = match state.settings.start_page.clone() {
            View::Feed(id) if state.feed(id).is_none() => View::Home,
            View::Folder(name) if !state.folders().contains(&name) => View::Home,
            View::Tag(name) if !state.tags().contains(&name) => View::Home,
            view => view,
        };
        state
            .stream_settings
            .get(&start_page.storage_key())
            .copied()
            .unwrap_or_default()
            .apply_to(&mut state.settings);
        let articles = SourceCache::new(|a: &Rc<Article>| a.id);
        articles.edit(|cache| {
            cache.add_or_update_items(&mut state.articles.iter().cloned().map(Rc::new))
        });
        let counts = counts::CountIndex::new(&state);
        let vm = Rc::new_cyclic(|weak: &std::rc::Weak<Self>| {
            let receiver = weak.clone();
            Self {
                reactive: ReactiveObject::new(),
                revision: RefCell::new(0),
                document: RefCell::new(state),
                library_generation: Cell::new(0),
                article_loads: article_content::ArticleLoads::default(),
                now: Cell::new(now),
                view: RefCell::new(start_page),
                selected: Cell::new(None),
                cursor: Cell::new(None),
                kept_unread: RefCell::new(BTreeSet::new()),
                viewport: Cell::new(1440.0),
                viewport_height: Cell::new(900.0),
                touch_mode: Cell::new(false),
                sidebar: Cell::new(false),
                status: RefCell::new(error.unwrap_or_default()),
                refreshing: Cell::new(false),
                dirty: Cell::new(false),
                applying: Cell::new(false),
                pending: RefCell::new(VecDeque::new()),
                undo: RefCell::new(VecDeque::new()),
                visible_ids: RefCell::new(Vec::new()),
                visible_set: RefCell::new(BTreeSet::new()),
                counts: RefCell::new(counts),
                services: RefCell::new(HostServices::default()),
                has_services: Cell::new(false),
                save,
                articles,
                visible: SourceCache::new(|a: &Rc<Article>| a.id),
                execute: ReactiveCommand::create_param_with(
                    move |action: Option<Action>| {
                        if let Some(vm) = receiver.upgrade() {
                            vm.apply(action);
                        }
                    },
                    None,
                    Some(ImmediateScheduler::instance()),
                ),
            }
        });
        vm.rebuild_query();
        vm
    }

    pub fn state(&self) -> Ref<'_, AppState> {
        self.document.borrow()
    }
    /// Hosts reject network completions belonging to a replaced library, even if
    /// an imported document happens to reuse the same subscription IDs and URLs.
    pub fn library_generation(&self) -> u64 {
        self.library_generation.get()
    }
    pub fn revision(&self) -> u64 {
        *self.revision.borrow()
    }
    pub fn now(&self) -> i64 {
        self.now.get()
    }
    pub fn set_now(&self, now: i64) {
        self.now.set(now);
    }
    pub fn view(&self) -> View {
        self.view.borrow().clone()
    }
    pub fn selected(&self) -> Option<Id> {
        self.selected.get()
    }
    pub fn cursor(&self) -> Option<Id> {
        self.cursor.get()
    }
    pub fn viewport(&self) -> f64 {
        self.viewport.get()
    }
    pub fn viewport_height(&self) -> f64 {
        self.viewport_height.get()
    }
    pub fn touch_mode(&self) -> bool {
        self.touch_mode.get()
    }
    pub fn sidebar_open(&self) -> bool {
        self.sidebar.get()
    }
    pub fn is_refreshing(&self) -> bool {
        self.refreshing.get()
    }
    pub fn status(&self) -> String {
        self.status.borrow().clone()
    }
    pub fn set_status(&self, text: String) {
        *self.status.borrow_mut() = text;
        self.notify();
    }
    pub fn report_error(&self, text: String) {
        self.set_status(text);
    }
    pub fn clear_status(&self) {
        self.set_status(String::new());
    }
    pub fn set_services(&self, services: HostServices) {
        *self.services.borrow_mut() = services;
        self.has_services.set(true);
    }
    pub fn request(&self, request: HostRequest) {
        let handler = self.services.borrow().request.clone();
        handler(request);
    }
    pub fn dispatch(&self, action: Action) {
        if self.applying.get() {
            self.pending.borrow_mut().push_back(action);
            return;
        }
        let _ = self.execute.execute(Some(action)).subscribe_next(|_| {});
    }
    pub(crate) fn notify(&self) {
        self.raise_and_set_if_changed(&self.revision, self.revision().wrapping_add(1), "Revision");
    }
}
