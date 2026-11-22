use ferroui_base::input::ContextRequestedEventArgs;
use ferroui_controls::{Button, Control};
use ferroui_view::prelude::*;

pub fn item(label: &str, action: impl Fn() + 'static) -> Element {
    menu_item()
        .header(label)
        .automation_properties_name(label)
        .on_click(move |e| {
            e.set_handled(true);
            action();
        })
        .into()
}
pub fn checked(label: &str, value: bool, action: impl Fn() + 'static) -> Element {
    menu_item()
        .header(label)
        .toggle_type(ferroui_controls::MenuItemToggleType::CheckBox)
        .is_checked(value)
        .on_click(move |e| {
            e.set_handled(true);
            action();
        })
        .into()
}
pub fn separator() -> Element {
    ferroui_view::prelude::separator().into_element()
}
pub fn button(label: &str, items: Vec<Element>) -> Element {
    let mut menu = context_menu().min_width(205.0).font_size(13.0);
    for item in items {
        menu = menu.child(item);
    }
    reader_controls::dropdown(label)
        .context_menu(menu.into_element())
        .on_click(move |e| {
            if let Some(button) = e.source().and_then(|s| s.cast::<Button>()) {
                button.raise_event(&ContextRequestedEventArgs::new());
            }
            e.set_handled(true);
        })
        .into()
}
pub fn attach(element: El<Button>, items: Vec<Element>) -> Element {
    let mut menu = context_menu().min_width(200.0);
    for item in items {
        menu = menu.child(item);
    }
    element
        .context_menu(menu.into_element())
        .on_click(move |e| {
            if let Some(button) = e.source().and_then(|s| s.cast::<Control>()) {
                button.raise_event(&ContextRequestedEventArgs::new());
            }
            e.set_handled(true);
        })
        .into()
}
