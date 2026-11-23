use crate::session::{FolderTarget, Panel, Session};
use ferroui_base::media::TextWrapping;
use ferroui_view::prelude::*;
use reader_controls::{action, action_row, paragraph};
use reader_state::AppViewModel;
use std::rc::Rc;

pub(super) fn view(
    vm: Rc<AppViewModel>,
    ui: Rc<Session>,
    target: FolderTarget,
    unsubscribe: bool,
) -> Element {
    let panel = if unsubscribe {
        Panel::UnsubscribeFolder(target.clone())
    } else {
        Panel::DeleteFolder(target.clone())
    };
    let count = target.feeds.len();
    let valid = target.valid(&vm);
    let text = if unsubscribe {
        format!(
            "Unsubscribe from {count} {} in {}? They will be removed from every folder. Starred, tagged and noted articles will remain in your library; other articles from these subscriptions will be removed. You can undo this change.",
            if count == 1 { "subscription" } else { "subscriptions" }, target.name
        )
    } else {
        format!(
            "Delete the folder {}? Its {count} {} and all saved articles will stay in your library. Only this folder label will be removed. You can undo this change.",
            target.name, if count == 1 { "subscription" } else { "subscriptions" }
        )
    };
    let confirm = ui.clone();
    let cancel = ui.clone();
    let confirm_target = target.clone();
    let label = if unsubscribe {
        "Unsubscribe from all"
    } else {
        "Delete folder"
    };
    stack_panel()
        .name("FolderConfirmation")
        .spacing(16.0)
        .child(paragraph(text))
        .child(text_block()
            .name("FolderConfirmationError")
            .is_visible(!valid)
            .text("This folder has changed. Close this dialog and review its current subscriptions before trying again.")
            .foreground("#B3261E")
            .text_wrapping(TextWrapping::Wrap))
        .child(action_row([
            button().name("ConfirmFolderAction").class("reader-primary")
                .content(label).automation_properties_name(label)
                .is_enabled(valid && (!unsubscribe || count > 0))
                .min_height(if crate::responsive::touch(&vm) { 44.0 } else { 29.0 })
                .on_click(move |event| {
                    event.set_handled(true);
                    if confirm.panel.get_untracked() != panel { return; }
                    if !confirm_target.valid(&vm) {
                        confirm.form_error.set("This folder has changed. Close this dialog and try again.".into());
                        return;
                    }
                    let changed = if unsubscribe {
                        vm.unsubscribe_many(confirm_target.feeds.iter().map(|(id, _)| *id).collect())
                    } else {
                        vm.delete_folder(confirm_target.name.clone())
                    };
                    if changed {
                        if unsubscribe {
                            confirm.removed_subscriptions(&confirm_target.feeds.iter().map(|(id, _)| *id).collect::<Vec<_>>());
                        } else {
                            confirm.removed_folder(&vm, &confirm_target.name);
                        }
                        confirm.close();
                    }
                }).into(),
            action("Cancel", move || cancel.close()),
        ], 16.0))
        .into()
}
