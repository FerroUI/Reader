//! The TreeDataGrid realization engine behind a normal ListBox items panel.
//! The source, selection, templates, native scrolling and automation stay native.
//! Reader supplies a persistent height index instead of a moving row average.
mod anchors;
mod changes;
mod containers;
mod navigation;
mod presenter;

use super::metrics::Metrics;
use crate::session::Session;
use ferroui_base::input::{InputElement, InputElementImpl};
use ferroui_base::interactivity::InteractiveImpl;
use ferroui_base::layout::LayoutableImpl;
use ferroui_base::logical_tree::LogicalTreeAttachmentEventArgs;
use ferroui_base::{
    ferro_class, ferro_impl_classes, instantiate, FerroObjectImpl, Ref, Size, StyledElementImpl,
    StyledElementImplExt, Visual, VisualImpl, VisualImplExt, VisualTreeAttachmentEventArgs,
    WeakRef,
};
use ferroui_controls::generators::ItemContainerGenerator;
use ferroui_controls::templates::FuncTemplate;
use ferroui_controls::{ControlImpl, ListBox, Panel, PanelImpl, VirtualizingPanel};
use ferroui_controls_treedatagrid::primitives::TreeDataGridPresenterBase;
use reader_domain::{Id, View};
use reader_state::AppViewModel;
use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};

use containers::Container;

pub(super) fn install(list: &Ref<ListBox>, vm: &Rc<AppViewModel>, ui: &Rc<Session>) {
    let vm = Rc::downgrade(vm);
    let ui = Rc::downgrade(ui);
    // Install once per retained ListBox, not during every reactive render.
    list.set_items_panel(FuncTemplate::new(move || {
        Some(ArticlePanel::new(vm.clone(), ui.clone()).upcast::<Panel>())
    }));
}

pub(super) fn user_input(list: &Ref<ListBox>) {
    if let Some(panel) = list
        .items_panel_root()
        .and_then(|panel| panel.cast::<ArticlePanel>())
    {
        panel.cancel_restore();
    }
}

struct Anchor {
    id: Id,
    within: f64,
    route: View,
    epoch: u64,
    generation: u64,
    content: Option<(WeakRef<ferroui_controls::Control>, f64)>,
}

#[repr(C)]
pub(super) struct ArticlePanel {
    base: VirtualizingPanel,
    presenter: TreeDataGridPresenterBase,
    metrics: RefCell<Metrics>,
    containers: RefCell<Vec<Container>>,
    generator: RefCell<Option<Rc<ItemContainerGenerator>>>,
    vm: Weak<AppViewModel>,
    ui: Weak<Session>,
    route: RefCell<Option<View>>,
    generation: Cell<u64>,
    anchor: RefCell<Option<Anchor>>,
    epoch: Cell<u64>,
    jumping: Cell<bool>,
    body_anchors: RefCell<Vec<Ref<ferroui_controls::Control>>>,
    anchor_provider: RefCell<Option<WeakRef<Visual>>>,
    anchors_queued: Cell<bool>,
}

ferro_class!(ArticlePanel: VirtualizingPanel);
ferro_impl_classes!(ArticlePanel: FerroObjectImpl, InteractiveImpl, ControlImpl, PanelImpl);

impl ArticlePanel {
    fn new(vm: Weak<AppViewModel>, ui: Weak<Session>) -> Ref<Self> {
        instantiate(Self {
            base: VirtualizingPanel::construct(),
            presenter: TreeDataGridPresenterBase::new(),
            metrics: RefCell::new(Metrics::default()),
            containers: RefCell::new(Vec::new()),
            generator: RefCell::new(None),
            vm,
            ui,
            route: RefCell::new(None),
            generation: Cell::new(0),
            anchor: RefCell::new(None),
            epoch: Cell::new(0),
            jumping: Cell::new(false),
            body_anchors: RefCell::new(Vec::new()),
            anchor_provider: RefCell::new(None),
            anchors_queued: Cell::new(false),
        })
    }

    fn synchronize_metrics(&self, width: f64) {
        let width = if width.is_finite() && width > 0.0 {
            width
        } else {
            self.bounds().width.max(320.0)
        };
        if (width - self.metrics.borrow().width).abs() > 0.5 {
            self.capture_anchor();
        }
        self.metrics.borrow_mut().synchronize(&self.items(), width);
        if let Some(vm) = self.vm.upgrade() {
            *self.route.borrow_mut() = Some(vm.view());
            self.generation.set(vm.library_generation());
        }
    }

    fn cancel_restore(&self) {
        self.epoch.set(self.epoch.get().wrapping_add(1));
        self.anchor.borrow_mut().take();
    }
}

impl InputElementImpl for ArticlePanel {
    fn children_in_tab_focus_order(this: &Self) -> Option<Vec<Ref<InputElement>>> {
        let mut elements: Vec<_> = this
            .containers
            .borrow()
            .iter()
            .filter(|entry| entry.index >= 0)
            .map(|entry| (entry.index, entry.control.clone().upcast()))
            .collect();
        elements.sort_unstable_by_key(|(index, _)| *index);
        Some(elements.into_iter().map(|(_, control)| control).collect())
    }
}

impl LayoutableImpl for ArticlePanel {
    fn measure_override(this: &Self, available: Size) -> Size {
        this.synchronize_metrics(available.width);
        let result = this.presenter.measure_override(this, available);
        this.trim_pool(8);
        // The pinned TreeDataGrid base still estimates its extent from the
        // current realized window. Use measured/estimated per-article offsets.
        Size::new(result.width, this.metrics.borrow().extents.total())
    }

    fn arrange_override(this: &Self, final_size: Size) -> Size {
        let result = this.presenter.arrange_override(this, final_size);
        this.restore_anchor_after_layout();
        this.queue_body_anchors();
        result
    }
}

impl VisualImpl for ArticlePanel {
    fn on_attached_to_visual_tree(this: &Self, event: &VisualTreeAttachmentEventArgs) {
        Self::parent_on_attached_to_visual_tree(this, event);
        this.presenter.on_attached_to_visual_tree(this);
    }

    fn on_detached_from_visual_tree(this: &Self, event: &VisualTreeAttachmentEventArgs) {
        Self::parent_on_detached_from_visual_tree(this, event);
        this.cancel_restore();
        this.clear_body_anchors(None);
        this.presenter.on_detached_from_visual_tree(this);
        this.reset_containers();
    }
}

impl StyledElementImpl for ArticlePanel {
    fn on_detached_from_logical_tree(this: &Self, event: &LogicalTreeAttachmentEventArgs) {
        Self::parent_on_detached_from_logical_tree(this, event);
        this.presenter.on_detached_from_logical_tree(this);
        this.reset_containers();
    }
}
