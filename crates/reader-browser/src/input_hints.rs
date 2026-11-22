//! Safe keyboard hints for Reader's plain TextBoxes on the pinned browser host.
//! The framework's exact-edit textarea keeps its own correction/input protocol.
use ferroui_base::{
    input::{
        text_input::{
            TextInputContentType as Content, TextInputOptions, TextInputReturnKeyType as Return,
        },
        InputElement,
    },
    interactivity::RoutingStrategies,
};
use ferroui_controls::{TextBox, TopLevel};
use std::cell::Cell;

ferroui_base::ferro_thread_local! {
    static INSTALLED: Cell<bool> = const { Cell::new(false) };
}

pub fn install() {
    if INSTALLED.with(|installed| installed.replace(true)) {
        return;
    }
    // GotFocus precedes TextInputMethodManager::set_focused_element, so the
    // legacy DOM input receives its hints before the platform focuses it.
    // This also runs when native A -> B retains that same focused DOM input.
    InputElement::got_focus_event()
        .add_class_handler_with::<TopLevel>(
            |root, _| publish(root),
            RoutingStrategies::BUBBLE,
            true,
        )
        .detach();
}

fn publish(root: &TopLevel) {
    let Some(focused) = root.focus_manager().get_focused_element() else {
        return;
    };
    if focused.cast::<TextBox>().is_none() {
        return;
    }
    let options = TextInputOptions::from_styled_element(&focused);
    let content = TextInputOptions::get_effective_content_type(&focused);
    let mode = match content {
        Content::Digits => "tel",
        Content::Pin => "numeric",
        Content::Number => "decimal",
        Content::Email => "email",
        Content::Url => "url",
        Content::Search => "search",
        _ => "text",
    };
    let enter = match options.return_key_type {
        Return::Return => "enter",
        Return::Done => "done",
        Return::Go => "go",
        Return::Send => "send",
        Return::Search => "search",
        Return::Next => "next",
        Return::Previous => "previous",
        Return::Default if options.multiline => "enter",
        Return::Default if mode == "search" => "search",
        Return::Default => "done",
    };
    let capitalization = if options.is_sensitive
        || options.lowercase
        || matches!(content, Content::Password | Content::Pin)
    {
        "none"
    } else if options.uppercase {
        "characters"
    } else if options.auto_capitalization {
        "sentences"
    } else {
        "none"
    };
    let language = options
        .locale_hints
        .as_deref()
        .and_then(|hints| hints.iter().find(|value| !value.is_empty()))
        .map(String::as_str)
        .unwrap_or("");
    crate::host::input_hints_changed(mode, enter, capitalization, language);
}
