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
    /// Re-project fixed-width media on same-breakpoint resizes. Widths beyond
    /// the reading cap and collapsed headline rows need no template changes.
    pub media_width: f64,
    pub density: Density,
    pub font_size: f64,
    pub narrow: bool,
    pub touch: bool,
    pub single_source: bool,
    pub article_loading: bool,
    pub article_error: Option<String>,
}
impl PartialEq for Row {
    fn eq(&self, other: &Self) -> bool {
        (Rc::ptr_eq(&self.article, &other.article) || self.article == other.article)
            && self.source == other.source
            && self.selected == other.selected
            && self.expanded == other.expanded
            && (!self.expanded || self.media_width == other.media_width)
            && self.density == other.density
            && self.font_size == other.font_size
            && self.narrow == other.narrow
            && self.touch == other.touch
            && self.single_source == other.single_source
            && self.article_loading == other.article_loading
            && self.article_error == other.article_error
    }
}
impl Row {
    fn boxed(&self) -> Option<BoxedValue> {
        Some(Rc::new(self.clone()))
    }
}
/// Retains the observable collection and recycling template for the full window lifetime.
pub struct Rows {
    data: RefCell<Vec<Row>>,
    items: Rc<FerroList<Option<BoxedValue>>>,
    pub template: Rc<dyn IDataTemplate>,
    applying: Cell<bool>,
    pending: RefCell<Option<Vec<Row>>>,
}
impl Rows {
    pub fn new(vm: Rc<AppViewModel>, session: Rc<Session>) -> Self {
        Self {
            data: RefCell::new(Vec::new()),
            items: Rc::new(FerroList::new()),
            template: ferroui_view::template(move |row: Row| {
                crate::article::view(vm.clone(), session.clone(), row)
            }),
            applying: Cell::new(false),
            pending: RefCell::new(None),
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
        let selected = vm.selected();
        let cursor = vm.cursor();
        let titles: std::collections::HashMap<_, Rc<str>> = vm
            .feeds()
            .into_iter()
            .map(|f| (f.id, Rc::from(f.title)))
            .collect();
        let narrow = vm.viewport() < 700.0;
        let media_width = crate::images::available_width(vm.viewport());
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
                Row {
                    selected: cursor.or(selected) == Some(article.id),
                    article_loading: vm.article_loading(article.id),
                    article_error: vm.article_error(article.id),
                    single_source,
                    article,
                    source,
                    expanded,
                    media_width,
                    density: settings.density,
                    font_size: settings.font_size,
                    narrow,
                    touch: crate::responsive::touch(vm),
                }
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
