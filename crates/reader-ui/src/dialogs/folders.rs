use crate::session::{Panel, Session};
use ferroui_base::input::Key;
use ferroui_base::layout::HorizontalAlignment as H;
use ferroui_view::prelude::*;
use reader_controls::{action, action_row, paragraph, primary, separator};
use reader_domain::Id;
use reader_state::AppViewModel;
use std::rc::Rc;

/// Folder labels are independent memberships, matching Reader's folder checklist.
pub fn view(vm: Rc<AppViewModel>, ui: Rc<Session>, id: Id) -> Element {
    if ui.form_for.borrow().as_ref() != Some(&Panel::Move(id)) {
        *ui.form_for.borrow_mut() = Some(Panel::Move(id));
        ui.entry.set(String::new());
    }
    let Some(feed) = vm.feed(id) else {
        return paragraph("This subscription has been removed.");
    };
    let mut form = stack_panel()
        .spacing(10.0)
        .child(paragraph(format!("Show {} in these folders:", feed.title)));
    for folder in vm.folders() {
        let current = feed.in_folder(&folder);
        let model = vm.clone();
        let name = folder.clone();
        form = form.child(
            check_box()
                .class("reader-check")
                .key(format!("folder-choice:{folder}"))
                .content(folder)
                .horizontal_alignment(H::Stretch)
                .is_checked(Some(current))
                .font_size(13.0)
                .min_height(26.0)
                .padding(0.0)
                .on_click(move |_| {
                    let Some(feed) = model.feed(id) else {
                        return;
                    };
                    let mut folders = feed.folder_names();
                    if current {
                        folders.retain(|folder| folder != &name)
                    } else {
                        folders.push(name.clone());
                    }
                    model.set_feed_folders(id, folders);
                }),
        );
    }
    let new_vm = vm.clone();
    let new_ui = ui.clone();
    let done = ui.clone();
    let unfiled = vm.clone();
    let create: Rc<dyn Fn()> = Rc::new(move || {
        let name = new_ui.entry.get_untracked().trim().to_owned();
        if name.is_empty() {
            new_ui.form_error.set("Enter a folder name.".into());
            return;
        }
        let Some(feed) = new_vm.feed(id) else {
            return;
        };
        let mut folders = feed.folder_names();
        folders.push(name.clone());
        new_vm.create_folder(name);
        new_vm.set_feed_folders(id, folders);
        new_ui.entry.set(String::new());
        new_ui.form_error.set(String::new());
    });
    let enter = create.clone();
    let error = ui.form_error;
    form.child(separator())
        .child(
            reader_controls::input("New folder name")
                .name("NewFolderName")
                .node_ref(ui.field)
                .text(ui.entry.two_way())
                .placeholder_text("New folder name")
                .on_text_changed(move |_| error.set(String::new()))
                .on_key_down(move |event| {
                    if event.key == Key::Return {
                        event.set_handled(true);
                        enter();
                    }
                }),
        )
        .child(
            text_block()
                .is_visible(!ui.form_error.get().is_empty())
                .text(ui.form_error.get())
                .foreground("#B3261E")
                .text_wrapping(ferroui_base::media::TextWrapping::Wrap),
        )
        .child(action("Create and add to folder", move || create()))
        .child(border().margin((0.0, 12.0, 0.0, 0.0)).child(action_row(
            [
                primary("Done", move || done.close()),
                action("Remove from all folders", move || {
                    unfiled.set_feed_folders(id, Vec::new())
                }),
            ],
            10.0,
        )))
        .into()
}
