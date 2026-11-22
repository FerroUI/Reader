use ferroui_base::input::ContextRequestedEventArgs;
use ferroui_controls::{Button, Control};
use ferroui_view::prelude::*;
use reader_state::AppViewModel;

pub fn item(label: &str, action: impl Fn() + 'static) -> Element {
    menu_item()
        .class("reader-menu")
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
        .class("reader-menu")
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
pub fn button(vm: &AppViewModel, label: &str, items: Vec<Element>) -> Element {
    attach(vm, reader_controls::dropdown(label), items)
}
pub fn context(vm: &AppViewModel, items: Vec<Element>) -> Element {
    let width = (vm.viewport() - 16.0).max(1.0);
    let mut menu = context_menu()
        .with(crate::transient::attach_menu)
        .class(if crate::responsive::touch(vm) {
            "reader-touch"
        } else {
            "reader-desktop"
        })
        .placement(ferroui_controls::PlacementMode::BottomEdgeAlignedLeft)
        .min_width(200.0_f64.min(width))
        .max_width(width)
        .max_height((vm.viewport_height() - 16.0).max(1.0));
    for item in items {
        menu = menu.child(item);
    }
    menu.into_element()
}
pub fn attach(vm: &AppViewModel, element: El<Button>, items: Vec<Element>) -> Element {
    element
        .context_menu(context(vm, items))
        .on_click(move |e| {
            if let Some(button) = e.source().and_then(|s| s.cast::<Control>()) {
                button.raise_event(&ContextRequestedEventArgs::new());
            }
            e.set_handled(true);
        })
        .into()
}
