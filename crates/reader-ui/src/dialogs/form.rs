use crate::session::{Panel, Session};
use ferroui_base::input::text_input::{
    TextInputContentType, TextInputReturnKeyType, TextSelection,
};
use ferroui_base::input::Key;
use ferroui_base::layout::HorizontalAlignment as H;
use ferroui_base::media::TextWrapping;
use ferroui_view::prelude::*;
use reader_controls::{action, action_row, paragraph, primary};
use reader_state::AppViewModel;
use std::rc::Rc;

/// The same edit session powers a desktop quick-add bubble and a narrow dialog.
/// Responsive presentation changes retain the same draft and validation state.
pub(super) struct Form {
    pub input: El<TextBox>,
    pub save: Rc<dyn Fn()>,
    pub label: &'static str,
}

pub(super) fn prepare(vm: Rc<AppViewModel>, ui: Rc<Session>, panel: Panel) -> Form {
    let fresh = ui.form_for.borrow().as_ref() != Some(&panel);
    let selection = if fresh {
        None
    } else {
        ui.field
            .get_untracked()
            .filter(|field| field.is_attached_to_visual_tree())
            .map(|field| {
                (
                    TextSelection::new(field.selection_start(), field.selection_end()),
                    field.caret_index(),
                )
            })
    };
    if fresh {
        let value = match &panel {
            Panel::Rename(id) => vm.feed(*id).map(|f| f.title).unwrap_or_default(),
            Panel::RenameFolder(name) => name.clone(),
            Panel::Tags(id) => vm
                .article(*id)
                .map(|a| a.tags.join(", "))
                .unwrap_or_default(),
            Panel::Note(id) => vm.article(*id).map(|a| a.note).unwrap_or_default(),
            _ => String::new(),
        };
        *ui.form_for.borrow_mut() = Some(panel.clone());
        ui.entry.set(value);
    }
    let label = match panel {
        Panel::Subscribe => "Enter a feed or website URL:",
        Panel::Move(_) => "Folder name (leave blank for no folder):",
        Panel::Tags(_) => "Separate tags with commas:",
        Panel::Note(_) => "Your note:",
        _ => "Name:",
    };
    let generation = vm.library_generation();
    let route = vm.view();
    let save_ui = ui.clone();
    let save_panel = panel.clone();
    let save: Rc<dyn Fn()> = Rc::new(move || {
        if save_ui.panel.get_untracked() != save_panel {
            return;
        }
        if vm.library_generation() != generation || vm.view() != route {
            save_ui.close();
            return;
        }
        let value = save_ui.entry.get_untracked().trim().to_string();
        if value.is_empty()
            && !matches!(save_panel, Panel::Move(_) | Panel::Tags(_) | Panel::Note(_))
        {
            save_ui.form_error.set("Please enter a value.".into());
            if let Some(field) = save_ui.field.get_untracked() {
                field.focus();
            }
            return;
        }
        save_ui.close();
        match save_panel {
            Panel::Subscribe => vm.subscribe(value),
            Panel::Rename(id) => vm.rename_feed(id, value),
            Panel::RenameFolder(ref name) => vm.rename_folder(name.clone(), value),
            Panel::Move(id) => vm.move_feed(id, value),
            Panel::Tags(id) => vm.set_tags(
                id,
                value
                    .split(',')
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(str::to_string)
                    .collect(),
            ),
            Panel::Note(id) => vm.set_note(id, value),
            Panel::NewFolder => vm.create_folder(value),
            _ => {}
        }
    });
    let enter = save.clone();
    let field = ui.field;
    let validation = ui.form_error;
    let focus_ui = ui.clone();
    let focus_panel = panel.clone();
    let mut input = reader_controls::input(label)
        .name("DialogInput")
        .node_ref(field)
        .text(ui.entry.two_way())
        .on_text_changed(move |_| validation.set_if_changed(String::new()))
        .horizontal_alignment(H::Stretch)
        .with(move |field| {
            super::editor_focus::attach(field, &focus_ui, focus_panel.clone(), fresh, selection);
        });
    if panel == Panel::Subscribe {
        input = input
            .text_input_options_content_type(TextInputContentType::Url)
            .text_input_options_return_key_type(TextInputReturnKeyType::Go)
            .text_input_options_auto_capitalization(false)
            .text_input_options_show_suggestions(Some(false));
    }
    if matches!(panel, Panel::Note(_)) {
        input = input
            .height(150.0)
            .accepts_return(true)
            .text_wrapping(TextWrapping::Wrap);
    } else {
        input = input.on_key_down(move |e| {
            if e.key == Key::Return {
                e.set_handled(true);
                enter();
            }
        });
    }
    Form { input, save, label }
}

pub(super) fn error(ui: &Session) -> Element {
    let message = ui.form_error.get();
    if message.is_empty() {
        return Element::Empty;
    }
    text_block()
        .name("FormError")
        .text(message)
        .foreground("#B3261E")
        .font_size(13.0)
        .text_wrapping(TextWrapping::Wrap)
        .into()
}

pub(super) fn view(vm: Rc<AppViewModel>, ui: Rc<Session>, panel: Panel) -> Element {
    let Form { input, save, label } = prepare(vm, ui.clone(), panel.clone());
    let mut form = stack_panel()
        .spacing(12.0)
        .child(paragraph(label))
        .child(input);
    if panel == Panel::Subscribe {
        form = form.child(paragraph("Follow your favorite websites in one place. Paste an RSS or Atom feed address, or a website that advertises one."));
    }
    let validation = error(&ui);
    let action_text = if panel == Panel::Subscribe {
        "Add"
    } else {
        "Save"
    };
    form.child(validation)
        .child(border().margin((0.0, 4.0, 0.0, 0.0)).child(action_row(
            [
                primary(action_text, move || save()),
                action("Cancel", move || ui.close()),
            ],
            16.0,
        )))
        .into()
}
