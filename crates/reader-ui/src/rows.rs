mod diff;
use crate::session::Session;
use ferroui_base::{collections::FerroList, BoxedValue};
use ferroui_controls::{templates::IDataTemplate, ItemsSource};
use reader_domain::{Article, Density, Id, ViewMode};
use reader_state::AppViewModel;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

#[derive(Clone)]
pub struct Row {
    pub article: Rc<Article>,
    pub source: Rc<str>,
    pub selected: bool,
    pub expanded: bool,
    pub list_mode: bool,
    /// Re-project fixed-width media on same-breakpoint resizes. Widths beyond
    /// the reading cap and collapsed headline rows need no template changes.
    pub media_width: f64,
    pub density: Density,
    pub scale: crate::reading::scale::Scale,
    pub wide_body: bool,
    pub font_size: f64,
    pub narrow: bool,
    pub touch: bool,
    pub single_source: bool,
    pub article_loading: bool,
    pub article_error: Option<String>,
    pub send_to_revision: u64,
    pub auto_read: bool,
    pub kept_unread: bool,
}
impl PartialEq for Row {
    fn eq(&self, other: &Self) -> bool {
        (Rc::ptr_eq(&self.article, &other.article) || self.article == other.article)
            && self.source == other.source
            && self.selected == other.selected
            && self.expanded == other.expanded
            && self.list_mode == other.list_mode
            && (!self.expanded || self.media_width == other.media_width)
            && self.density == other.density
            && self.scale == other.scale
            && (!self.expanded || self.wide_body == other.wide_body)
            && self.font_size == other.font_size
            && self.narrow == other.narrow
            && self.touch == other.touch
            && self.single_source == other.single_source
            && self.article_loading == other.article_loading
            && self.article_error == other.article_error
            && (!self.expanded || self.send_to_revision == other.send_to_revision)
            && (!self.expanded
                || (self.auto_read == other.auto_read && self.kept_unread == other.kept_unread))
    }
}
impl Row {
    fn boxed(&self) -> Option<BoxedValue> {
        Some(Rc::new(self.clone()))
    }
}
/// Retains the observable collection and recycling template for the full window lifetime.
pub struct Rows {
    session: Rc<Session>,
    data: RefCell<Vec<Row>>,
    items: Rc<FerroList<Option<BoxedValue>>>,
    pub template: Rc<dyn IDataTemplate>,
    applying: Cell<bool>,
    pending: RefCell<Option<Vec<Row>>>,
    send_to: RefCell<Vec<reader_domain::SendToDestination>>,
    send_to_generation: Cell<u64>,
    send_to_revision: Cell<u64>,
}
impl Rows {
    pub fn new(vm: Rc<AppViewModel>, session: Rc<Session>) -> Self {
        Self {
            session: session.clone(),
            data: RefCell::new(Vec::new()),
            items: Rc::new(FerroList::new()),
            template: ferroui_view::template(move |row: Row| {
                crate::article::view(vm.clone(), session.clone(), row)
            }),
            applying: Cell::new(false),
            pending: RefCell::new(None),
            send_to: RefCell::new(Vec::new()),
            send_to_generation: Cell::new(0),
            send_to_revision: Cell::new(0),
        }
    }
    pub fn source(&self) -> ItemsSource {
        self.items.clone().into()
    }
    pub fn len(&self) -> usize {
        self.data.borrow().len()
    }
    pub fn index(&self, id: Id) -> Option<i32> {
        self.data
            .borrow()
            .iter()
            .position(|r| r.article.id == id)
            .map(|i| i as i32)
    }
    pub fn synchronize(&self, vm: &AppViewModel) {
        let settings = vm.state().settings.clone();
        if *self.send_to.borrow() != settings.send_to
            || self.send_to_generation.get() != vm.library_generation()
        {
            *self.send_to.borrow_mut() = settings.send_to.clone();
            self.send_to_generation.set(vm.library_generation());
            self.send_to_revision
                .set(self.send_to_revision.get().wrapping_add(1));
        }
        let send_to_revision = self.send_to_revision.get();
        let selected = vm.selected();
        let cursor = vm.cursor();
        let titles: std::collections::HashMap<_, Rc<str>> = vm
            .state()
            .feeds
            .iter()
            .map(|f| (f.id, Rc::from(f.title.as_str())))
            .collect();
        let narrow = vm.viewport() < 700.0;
        let sidebar_hidden = narrow
            || !self.session.desktop_sidebar.get_untracked()
            || self.session.focus_mode.get_untracked();
        let sidebar_width = if sidebar_hidden {
            0.0
        } else {
            crate::sidebar::width(settings.density)
        };
        let scale = self.session.article_scale.get_untracked();
        let single_source = matches!(vm.view(), reader_domain::View::Feed(_));
        let next = vm
            .visible_shared()
            .into_iter()
            .map(|article| {
                let source = titles
                    .get(&article.feed_id)
                    .cloned()
                    .unwrap_or_else(|| Rc::from(""));
                let expanded =
                    settings.view_mode == ViewMode::Expanded || selected == Some(article.id);
                let mut row = Row {
                    selected: cursor.or(selected) == Some(article.id),
                    scale: scale.for_current(cursor.or(selected) == Some(article.id)),
                    wide_body: sidebar_hidden,
                    article_loading: vm.article_loading(article.id),
                    article_error: vm.article_error(article.id),
                    send_to_revision,
                    auto_read: settings.mark_read_on_scroll,
                    kept_unread: vm.is_kept_unread(article.id),
                    single_source,
                    article,
                    source,
                    expanded,
                    list_mode: settings.view_mode == ViewMode::List,
                    media_width: 0.0,
                    density: settings.density,
                    font_size: settings.font_size,
                    narrow,
                    touch: crate::responsive::touch(vm),
                };
                row.media_width = crate::article::media_width(&row, vm.viewport(), sidebar_width);
                row
            })
            .collect();
        *self.pending.borrow_mut() = Some(next);
        if self.applying.replace(true) {
            return;
        }
        loop {
            let next = self.pending.borrow_mut().take();
            let Some(next) = next else { break };
            let previous = self.data.replace(next.clone());
            diff::reconcile(&self.items, previous, &next);
        }
        self.applying.set(false);
    }
}
