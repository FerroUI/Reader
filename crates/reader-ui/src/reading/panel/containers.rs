use super::ArticlePanel;
use ferroui_base::{BoxedValue, Ref, StyledElement, Visual};
use ferroui_controls::generators::RecycleKey;
use ferroui_controls::Control;
use ferroui_controls_treedatagrid::primitives::PresenterItemsChange;

pub(super) struct Container {
    pub control: Ref<Control>,
    pub index: i32,
    key: Option<RecycleKey>,
    generated: bool,
    prepared: bool,
}

impl ArticlePanel {
    pub(super) fn create_or_recycle(&self, item: &Option<BoxedValue>, index: i32) -> Ref<Control> {
        let generator = self
            .generator
            .borrow()
            .clone()
            .expect("attached article generator");
        let (generated, key) = generator.needs_container(item, index);
        let recycled = self
            .containers
            .borrow()
            .iter()
            .find(|entry| entry.index < 0 && generated && key.is_some() && entry.key == key)
            .map(|entry| entry.control.clone());
        if let Some(control) = recycled {
            return control;
        }
        let control = if generated {
            generator.create_container(item, index, key)
        } else {
            item.as_ref()
                .and_then(Control::from_boxed)
                .expect("a self container")
        };
        self.containers.borrow_mut().push(Container {
            control: control.clone(),
            index: -1,
            key,
            generated,
            prepared: false,
        });
        control
    }

    pub(super) fn prepare(&self, control: &Ref<Control>, item: &Option<BoxedValue>, index: i32) {
        let generator = self
            .generator
            .borrow()
            .clone()
            .expect("attached article generator");
        {
            let mut containers = self.containers.borrow_mut();
            let entry = containers
                .iter_mut()
                .find(|entry| entry.control == *control)
                .expect("article container");
            entry.index = index;
            entry.prepared = true;
        }
        generator.prepare_item_container(control, item, index);
        // ListBox selection and child-index automation require the ItemsControl
        // as logical parent; the TreeDataGrid engine owns visual realization.
        if control.parent().is_none() {
            let owner = self.items_control().expect("article owner");
            owner
                .logical_children()
                .add(control.clone().upcast::<StyledElement>());
        }
        if control.visual_parent().is_none() {
            self.visual_children()
                .add(control.clone().upcast::<Visual>());
        }
        generator.item_container_prepared(control, item, index);
    }

    pub(super) fn recycle(&self, control: &Ref<Control>) {
        self.clear_body_anchors(Some(control));
        let clear = {
            let mut containers = self.containers.borrow_mut();
            let Some(entry) = containers
                .iter_mut()
                .find(|entry| entry.control == *control)
            else {
                return;
            };
            let clear = entry.generated && entry.prepared;
            entry.index = -1;
            entry.prepared = false;
            clear
        };
        if clear {
            if let Some(generator) = self.generator.borrow().clone() {
                generator.clear_item_container(control);
            }
        }
    }

    pub(super) fn update_index(&self, control: &Ref<Control>, old: i32, new: i32) {
        if let Some(entry) = self
            .containers
            .borrow_mut()
            .iter_mut()
            .find(|entry| entry.control == *control)
        {
            entry.index = new;
        }
        if let Some(generator) = self.generator.borrow().clone() {
            generator.item_container_index_changed(control, old, new);
        }
    }

    /// The framework preserves recycled tree membership. Cap Reader's idle pool
    /// independently of feed length and of a previously much larger viewport.
    pub(super) fn trim_pool(&self, keep: usize) {
        let removed = {
            let mut containers = self.containers.borrow_mut();
            let mut remaining = keep;
            let mut removed = Vec::new();
            containers.retain(|entry| {
                if entry.index >= 0 {
                    true
                } else if remaining > 0 && entry.key.is_some() {
                    remaining -= 1;
                    true
                } else {
                    removed.push(entry.control.clone());
                    false
                }
            });
            removed
        };
        for control in removed {
            if control.visual_parent() == Some(self.to_ref().upcast()) {
                self.visual_children()
                    .remove(&control.clone().upcast::<Visual>());
            }
            if let Some(parent) = control.parent() {
                parent
                    .logical_children()
                    .remove(&control.clone().upcast::<StyledElement>());
            }
        }
    }

    pub(super) fn reset_containers(&self) {
        self.presenter
            .on_items_collection_changed(self, &PresenterItemsChange::reset());
        self.trim_pool(0);
    }
}
