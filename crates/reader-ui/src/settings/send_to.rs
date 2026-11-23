use super::controls::{check, help};
use crate::session::{Panel, Session};
use ferroui_base::media::TextWrapping;
use ferroui_view::prelude::*;
use reader_controls::{action, action_row, paragraph, separator};
use reader_state::AppViewModel;
use std::rc::Rc;

pub fn view(vm: Rc<AppViewModel>, ui: Rc<Session>) -> Element {
    let destinations = vm.state().settings.send_to.clone();
    let add = ui.clone();
    let back = ui.clone();
    let generation = vm.library_generation();
    let control_height = if crate::responsive::touch(&vm) {
        44.0
    } else {
        32.0
    };
    let mut page = stack_panel().spacing(14.0)
        .child(paragraph("Choose where to send articles. Enabled destinations appear in the item's Send to menu and with Shift+T."));
    if destinations.is_empty() {
        page = page.child(paragraph(
            "No destinations are configured. Create a custom link to add your preferred service.",
        ));
    }
    for destination in destinations {
        let toggle = vm.clone();
        let remove = vm.clone();
        let edit = ui.clone();
        let id = destination.id;
        let enabled = destination.enabled;
        let name = destination.name.clone();
        page = page.child(rsx! {StackPanel {key:format!("send-to-setting:{id}"),spacing:6.0,
            {check(&destination.name,&format!("SendToEnabled-{id}"),enabled,crate::responsive::touch(&vm),move |value|{
                if toggle.library_generation()==generation && toggle.state().settings.send_to.iter().any(|item|item.id==id && item.enabled!=value) {
                    if let Err(error)=toggle.set_send_to_enabled(id,value){toggle.report_error(error);}
                }
            }).automation_properties_name(format!("Enable {}",destination.name))}
            TextBlock {text:destination.url_template,font_size:12.0,foreground:"#666666",text_wrapping:TextWrapping::Wrap}
            {action_row([
                button().class("reader-link").content("Edit").min_height(control_height).automation_properties_name(format!("Edit {name}")).on_click(move |event|{event.set_handled(true);edit.show(Panel::EditSendTo(Some(id)));}).into(),
                button().class("reader-link").content("Remove").min_height(control_height).automation_properties_name(format!("Remove {name}")).on_click(move |event|{event.set_handled(true);if remove.library_generation()==generation{remove.remove_send_to(id);}}).into()
            ],16.0)}
            {separator()}
        }});
    }
    page.child(help(
        "Changes to enabled destinations are saved automatically.",
    ))
    .child(action_row(
        [
            action("Create a custom link", move || {
                add.show(Panel::EditSendTo(None))
            }),
            action("Back to Reader settings", move || {
                back.show(Panel::Preferences)
            }),
        ],
        10.0,
    ))
    .into()
}
