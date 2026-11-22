use crate::session::{Panel, Session};
use ferroui_base::input::Key;
use ferroui_base::layout::HorizontalAlignment as H;
use ferroui_view::prelude::*;
use reader_domain::View;
use reader_state::AppViewModel;
use std::rc::Rc;

pub fn view(vm: Rc<AppViewModel>, ui: Rc<Session>, tags: bool) -> Element {
    let panel = if tags {
        Panel::GoToTag
    } else {
        Panel::GoToSubscription
    };
    if ui.form_for.borrow().as_ref() != Some(&panel) {
        *ui.form_for.borrow_mut() = Some(panel);
        ui.entry.set(String::new());
    }
    let filter = ui.entry.get().to_lowercase();
    let choices: Vec<(String, View)> = if tags {
        vm.tags()
            .into_iter()
            .map(|name| (name.clone(), View::Tag(name)))
            .chain(
                vm.folders()
                    .into_iter()
                    .map(|name| (format!("{name} (folder)"), View::Folder(name))),
            )
            .collect()
    } else {
        vm.feeds()
            .into_iter()
            .map(|feed| (feed.title, View::Feed(feed.id)))
            .collect()
    };
    let choices: Vec<_> = choices
        .into_iter()
        .filter(|(name, _)| name.to_lowercase().contains(&filter))
        .collect();
    let first = choices.first().map(|(_, view)| view.clone());
    let enter_vm = vm.clone();
    let enter_ui = ui.clone();
    let field = ui.field;
    let mut content = stack_panel().spacing(10.0).child(
        reader_controls::input(if tags {
            "Find a tag or folder"
        } else {
            "Find a subscription"
        })
        .name("NavigationFilter")
        .node_ref(field)
        .text(ui.entry.two_way())
        .placeholder_text("Start typing a name...")
        .on_loaded(move |_| {
            if let Some(field) = field.get_untracked() {
                field.focus();
                field.select_all();
            }
        })
        .on_key_down(move |event| {
            if event.key == Key::Return {
                if let Some(view) = &first {
                    enter_ui.close();
                    enter_vm.navigate(view.clone());
                }
                event.set_handled(true);
            }
        }),
    );
    if choices.is_empty() {
        content=content.child(reader_controls::paragraph(if tags{"No matching tags or folders. You can add tags with the t shortcut while reading an item."}else{"No matching subscriptions."}));
    }
    for (label, target) in choices {
        let model = vm.clone();
        let close = ui.clone();
        content = content.child(
            button()
                .key(format!("go:{target:?}"))
                .class("reader-link")
                .automation_properties_name(label.clone())
                .child(
                    text_block()
                        .text(label)
                        .text_wrapping(ferroui_base::media::TextWrapping::Wrap),
                )
                .horizontal_content_alignment(H::Left)
                .min_height(if crate::responsive::touch(&vm) {
                    44.0
                } else {
                    26.0
                })
                .on_click(move |_| {
                    close.close();
                    model.navigate(target.clone());
                }),
        );
    }
    content.into()
}
