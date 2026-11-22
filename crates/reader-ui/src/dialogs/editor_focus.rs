//! A responsive edit remount restores the outgoing editor's selection once.
//!
//! Text and two-way bindings are applied before the mount hook. Seed the native
//! range here, before an intervening reconciliation can read this replacement
//! through NodeRef. Loaded is queued; its only remaining job is to focus it.
use crate::session::{Panel, Session};
use ferroui_base::input::text_input::TextSelection;
use ferroui_base::Ref;
use ferroui_controls::{Control, TextBox};
use ferroui_view::on_cleanup;
use std::{cell::Cell, rc::Rc};

pub(super) fn attach(
    field: &Ref<TextBox>,
    ui: &Rc<Session>,
    panel: Panel,
    fresh: bool,
    selection: Option<(TextSelection, i32)>,
) {
    // The browser adapter publishes selection when focus attaches its native
    // input client. Initialize before focus and before any queued TextChanged.
    if fresh {
        field.select_all();
    } else if let Some((range, caret)) = selection {
        field.set_selection_and_caret(range, caret);
    }
    let session = Rc::downgrade(ui);
    let target = field.downgrade();
    let initialized = Cell::new(false);
    let loaded = field.loaded(move |_, _| {
        if initialized.replace(true) {
            return;
        }
        let (Some(ui), Some(field)) = (session.upgrade(), target.upgrade()) else {
            return;
        };
        if ui.panel.get_untracked() != panel
            || !field.is_attached_to_visual_tree()
            || ui
                .field
                .get_untracked()
                .is_none_or(|current| current != field)
        {
            return;
        }
        field.focus();
    });
    let target = field.downgrade();
    on_cleanup(move || {
        if let Some(field) = target.upgrade() {
            field.remove_handler(Control::loaded_event(), loaded);
        }
    });
}
