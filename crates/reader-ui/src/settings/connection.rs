//! The connection draft is independent of saved settings until validation succeeds.
use super::controls::{choice, choices, heading, help};
use crate::session::{Panel, Session};
use ferroui_base::input::{
    text_input::{TextInputContentType, TextInputReturnKeyType},
    Key,
};
use ferroui_base::media::TextWrapping;
use ferroui_controls::TextBox;
use ferroui_view::prelude::*;
use reader_domain::{FeedConnection, View};
use reader_state::AppViewModel;
use std::rc::Rc;

const PUBLIC_RELAY_NAME: &str = "rss2json";

pub(super) fn view(vm: Rc<AppViewModel>, ui: Rc<Session>) -> Element {
    let settings = vm.state().settings.clone();
    if ui.form_for.borrow().as_ref() != Some(&Panel::Preferences) {
        *ui.form_for.borrow_mut() = Some(Panel::Preferences);
        ui.entry.set(settings.feed_proxy.clone());
        ui.feed_connection.set(settings.feed_connection);
    }
    let touch = crate::responsive::touch(&vm);
    let mode = ui.feed_connection;
    let selected = mode.get();
    let input = ui.entry;
    let error = ui.form_error;
    let dirty = selected != settings.feed_connection || input.get().trim() != settings.feed_proxy;
    let options = [
        ("Automatic", FeedConnection::Automatic),
        ("Direct only", FeedConnection::Direct),
        ("Custom proxy", FeedConnection::Custom),
    ]
    .into_iter()
    .map(|(label, connection)| {
        choice(
            "connection",
            label,
            selected == connection,
            touch,
            move || {
                mode.set_if_changed(connection);
                error.set_if_changed(String::new());
            },
        )
    })
    .collect();
    let description=match selected {
        FeedConnection::Automatic=>format!("Automatic tries publishers directly, then your custom proxy or {PUBLIC_RELAY_NAME}. {PUBLIC_RELAY_NAME} receives public RSS/Atom feed URLs and may cache or limit items. When you choose Read full article, Jina Reader receives the public article URL if direct access fails. Use Direct only or your trusted proxy for private addresses."),
        FeedConnection::Direct=>"Connect only to the publisher. The publisher must allow browser access. Your saved custom proxy is kept but will not be used.".into(),
        FeedConnection::Custom=>"Try the publisher first, then your trusted proxy. The proxy receives the requested feed or article URL and must return its original XML or HTML.".into(),
    };
    let generation = vm.library_generation();
    let save_ui = ui.clone();
    let save: Rc<dyn Fn()> = Rc::new(move || {
        let panel = save_ui.panel.get_untracked();
        let preferences_visible =
            panel == Panel::Preferences || (panel == Panel::None && vm.view() == View::Settings);
        if !preferences_visible
            || save_ui.form_for.borrow().as_ref() != Some(&Panel::Preferences)
            || vm.library_generation() != generation
        {
            return;
        }
        match vm.set_feed_connection(mode.get_untracked(), &input.get_untracked()) {
            Ok(()) => error.set_if_changed(String::new()),
            Err(message) => {
                error.set(message);
                if let Some(field) = save_ui
                    .field
                    .get_untracked()
                    .filter(|field| field.is_attached_to_visual_tree())
                {
                    field.focus();
                }
            }
        }
    });
    let enter = save.clone();
    let message = error.get();
    rsx! {StackPanel {name:"FeedConnectionSettings",spacing:10.0,
        {heading("Feed connection")}
        {choices("connection",options)}
        {help(description)}
        if selected!=FeedConnection::Direct {StackPanel {spacing:6.0,
            {heading(if selected==FeedConnection::Custom{"Custom proxy URL"}else{"Custom proxy URL (optional)"})}
            {reader_controls::input("Feed proxy URL").name("FeedProxy").key("feed-proxy").node_ref(ui.field).height(if touch{44.0}else{29.0})
                .text(input.two_way()).placeholder_text("https://example.com/feed?url={url}")
                .text_input_options_content_type(TextInputContentType::Url).text_input_options_return_key_type(TextInputReturnKeyType::Done)
                .text_input_options_auto_capitalization(false).text_input_options_show_suggestions(Some(false))
                .on_text_changed(move|_|error.set_if_changed(String::new()))
                .on_key_down(move|event|{if event.key==Key::Return && !event.source().and_then(|source|source.cast::<TextBox>()).is_some_and(|field|field.is_composing_text()){event.set_handled(true);enter();}})}
            {help(if selected==FeedConnection::Custom{"Include {url} where the encoded feed or article address belongs.".into()}else{format!("Include {{url}} for the encoded address, or leave empty to use {PUBLIC_RELAY_NAME}.")})}
        }}
        if !message.is_empty(){TextBlock {name:"FeedConnectionError",text:message,foreground:"#B3261E",font_size:13.0,text_wrapping:TextWrapping::Wrap}}
        TextBlock {name:"FeedConnectionDraftStatus",text:if dirty{"Unsaved connection changes"}else{"Connection settings are saved."},font_size:12.0,foreground:"#666666",text_wrapping:TextWrapping::Wrap}
        Button {class:"reader-button",content:"Save feed connection",automation_properties_name:"Save feed connection",is_enabled:dirty,on_click:move |_|save()}
    }}
}
