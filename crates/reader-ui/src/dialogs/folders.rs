use crate::session::{Panel, Session};
use ferroui_base::layout::Orientation;
use ferroui_view::prelude::*;
use reader_controls::{action, paragraph, primary, separator};
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
                .key(format!("folder-choice:{folder}"))
                .content(folder)
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
    form.child(separator())
        .child(
            reader_controls::input("New folder name")
                .name("NewFolderName")
                .text(ui.entry.two_way())
                .placeholder_text("New folder name"),
        )
        .child(action("Create and add to folder", move || {
            let name = new_ui.entry.get_untracked().trim().to_owned();
            if name.is_empty() {
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
        }))
        .child(
            stack_panel()
                .orientation(Orientation::Horizontal)
                .spacing(10.0)
                .margin((0.0, 12.0, 0.0, 0.0))
                .child(primary("Done", move || done.close()))
                .child(action("Remove from all folders", move || {
                    unfiled.set_feed_folders(id, Vec::new())
                })),
        )
        .into()
}
