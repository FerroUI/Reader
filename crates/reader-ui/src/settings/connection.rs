//! The connection draft is independent of saved settings until validation succeeds.
use crate::session::{Panel, Session};
use ferroui_base::media::{FontWeight, TextWrapping};
use ferroui_view::prelude::*;
use reader_controls::{action, action_row, paragraph};
use reader_domain::FeedConnection;
use reader_state::AppViewModel;
use std::rc::Rc;

// Keep the provider name explicit and shared by every disclosure in this form.
const PUBLIC_RELAY_NAME: &str = "rss2json";

pub(super) fn view(vm: Rc<AppViewModel>, ui: Rc<Session>) -> Element {
    if ui.form_for.borrow().as_ref() != Some(&Panel::Preferences) {
        let settings = vm.state().settings.clone();
        *ui.form_for.borrow_mut() = Some(Panel::Preferences);
        ui.entry.set(settings.feed_proxy);
        ui.feed_connection.set(settings.feed_connection);
    }
    let mode = ui.feed_connection;
    let selected = mode.get();
    let mut choices = Vec::new();
    for (label, connection) in [
        ("Automatic", FeedConnection::Automatic),
        ("Direct only", FeedConnection::Direct),
        ("Custom proxy", FeedConnection::Custom),
    ] {
        let error = ui.form_error;
        choices.push(super::choice(label, selected == connection, move || {
            mode.set_if_changed(connection);
            error.set_if_changed(String::new());
        }));
    }
    let description = match selected {
        FeedConnection::Automatic => format!(
            "Automatic tries publishers directly, then your custom proxy or {PUBLIC_RELAY_NAME}. {PUBLIC_RELAY_NAME} receives public RSS/Atom feed URLs and may cache or limit items. Use Direct only or a trusted proxy for private feeds."
        ),
        FeedConnection::Direct => "Connect only to the publisher. The publisher must allow browser access. Your saved custom proxy is kept but will not be used.".into(),
        FeedConnection::Custom => "Try the publisher first, then your trusted proxy. The proxy receives the feed URL. Its address must contain {url} for the encoded feed address.".into(),
    };
    let input = ui.entry;
    let error = ui.form_error;
    let save = vm.clone();
    let message = error.get();
    rsx! { StackPanel {name:"FeedConnectionSettings",spacing:12.0,
        TextBlock {text:"Feed connection",font_weight:FontWeight::Bold}
        {action_row(choices, 8.0)}
        {paragraph(&description)}
        {reader_controls::input("Feed proxy URL").name("FeedProxy").text(input.two_way())
            .on_text_changed(move |_|error.set_if_changed(String::new()))
            .placeholder_text("https://your-service.example/feed?url={url}")}
        TextBlock {text:format!("Leave the custom proxy empty to use {PUBLIC_RELAY_NAME} in Automatic mode."),font_size:12.0,foreground:"#666666",text_wrapping:TextWrapping::Wrap}
        if !message.is_empty() {
            TextBlock {name:"FeedConnectionError",text:message,foreground:"#B3261E",font_size:13.0,text_wrapping:TextWrapping::Wrap}
        }
        {action("Save feed connection",move || {
            if let Err(message) = save.set_feed_connection(mode.get_untracked(), &input.get_untracked()) {
                error.set(message);
            }
        })}
    }}
}
