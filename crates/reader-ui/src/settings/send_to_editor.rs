use super::controls::{check, heading, help};
use crate::session::{Panel, Session};
use ferroui_base::input::{
    text_input::{TextInputContentType, TextInputReturnKeyType},
    Key,
};
use ferroui_base::media::TextWrapping;
use ferroui_controls::TextBox;
use ferroui_view::prelude::*;
use reader_controls::{action, action_row, primary};
use reader_state::AppViewModel;
use std::rc::Rc;

pub fn view(vm: Rc<AppViewModel>, ui: Rc<Session>, id: Option<u64>) -> Element {
    let panel = Panel::EditSendTo(id);
    let touch = crate::responsive::touch(&vm);
    let fresh = ui.form_for.borrow().as_ref() != Some(&panel);
    if fresh {
        let destination = id.and_then(|id| {
            vm.state()
                .settings
                .send_to
                .iter()
                .find(|item| item.id == id)
                .cloned()
        });
        ui.entry.set(
            destination
                .as_ref()
                .map(|item| item.name.clone())
                .unwrap_or_default(),
        );
        ui.send_to_url.set(
            destination
                .as_ref()
                .map(|item| item.url_template.clone())
                .unwrap_or_default(),
        );
        ui.send_to_enabled
            .set(destination.as_ref().is_none_or(|item| item.enabled));
        *ui.form_for.borrow_mut() = Some(panel.clone());
    }
    let name = ui.entry;
    let template = ui.send_to_url;
    let enabled = ui.send_to_enabled;
    let error = ui.form_error;
    let generation = vm.library_generation();
    let save_ui = ui.clone();
    let save: Rc<dyn Fn()> = Rc::new(move || {
        if save_ui.panel.get_untracked() != panel || vm.library_generation() != generation {
            return;
        }
        match vm.save_send_to(
            id,
            &name.get_untracked(),
            &template.get_untracked(),
            enabled.get_untracked(),
        ) {
            Ok(_) => save_ui.show(Panel::SendToSettings),
            Err(message) => error.set(message),
        }
    });
    let enter_url = save.clone();
    let focus = ui.clone();
    let cancel = ui.clone();
    let message = error.get();
    rsx! {StackPanel {name:"SendToEditor",spacing:14.0,
        StackPanel {spacing:6.0,
        {heading("Name")}
        {reader_controls::input("Send to name").key("send-to-name").name("SendToName").node_ref(ui.field).height(if touch {44.0}else{29.0}).max_length(80).text(name.two_way())
            .on_text_changed(move |_|error.set_if_changed(String::new()))
            .on_loaded(move |event|{if fresh && focus.panel.get_untracked()==Panel::EditSendTo(id){if let Some(field)=event.source().and_then(|source|source.cast::<TextBox>()){field.focus();field.select_all();}}})
            .text_input_options_return_key_type(TextInputReturnKeyType::Next)
            .on_key_down(move |event|{if event.key==Key::Return{
                if let Some(field)=event.source().and_then(|source|source.cast::<TextBox>()) {
                    if field.is_composing_text(){return;}
                    if let Some(editor)=field.find_ancestor_of_type_where::<ferroui_controls::Control>(false,|control|control.has_name("SendToEditor")) {
                        if let Some(url)=editor.get_visual_descendants().filter_map(|visual|visual.cast::<TextBox>()).find(|field|field.has_name("SendToTemplate")){
                            event.set_handled(true);url.focus();url.bring_into_view();
                        }
                    }
                }
            }})}
        }
        StackPanel {spacing:6.0,
        {heading("URL template")}
        {reader_controls::input("Send to URL template").key("send-to-template").name("SendToTemplate").height(if touch {44.0}else{29.0}).max_length(8192).text(template.two_way())
            .placeholder_text("https://example.com/share?url=${url}&title=${title}")
            .text_input_options_content_type(TextInputContentType::Url).text_input_options_return_key_type(TextInputReturnKeyType::Done)
            .text_input_options_auto_capitalization(false).text_input_options_show_suggestions(Some(false))
            .on_text_changed(move |_|error.set_if_changed(String::new()))
            .on_key_down(move |event|{if event.key==Key::Return && !event.source().and_then(|source|source.cast::<TextBox>()).is_some_and(|field|field.is_composing_text()){event.set_handled(true);enter_url();}})}
        {help("Use ${url} for the article address, ${title} for its title, and ${source} for its subscription name. Values are URL-encoded automatically.")}
        }
        {check("Show in Send to menu","SendToShowInMenu",enabled.get(),touch,move|value|enabled.set_if_changed(value))}
        if !message.is_empty() {TextBlock {name:"SendToError",text:message,foreground:"#B3261E",font_size:13.0,text_wrapping:TextWrapping::Wrap}}
        {action_row([primary("Save destination",move||save()),action("Cancel",move||cancel.show(Panel::SendToSettings))],10.0)}
    }}
}
