mod opening;
use ferroui_base::input::{ContextRequestedEventArgs, Key};
use ferroui_base::Rect;
use ferroui_controls::{Button, Control, MenuItemToggleType};
use ferroui_view::prelude::*;
use reader_state::AppViewModel;

pub fn item(label: &str, action: impl Fn() + 'static) -> Element {
    item_enabled(label, true, action)
}
pub fn item_enabled(label: &str, enabled: bool, action: impl Fn() + 'static) -> Element {
    menu_item()
        .class("reader-menu")
        .header(label)
        .automation_properties_name(label)
        .tool_tip_tip(label)
        .is_enabled(enabled)
        .on_click(move |e| {
            e.set_handled(true);
            action();
        })
        .into()
}
pub fn checked(label: &str, value: bool, action: impl Fn() + 'static) -> Element {
    choice(label, value, None, action)
}
pub fn choice(
    label: &str,
    value: bool,
    group: Option<&str>,
    action: impl Fn() + 'static,
) -> Element {
    menu_item()
        .class("reader-menu")
        .header(label)
        .automation_properties_name(label)
        .tool_tip_tip(label)
        // These are exclusive preferences. Re-selecting the current value must
        // not clear its checkmark when the model correctly publishes no change.
        .toggle_type(MenuItemToggleType::Radio)
        .group_name(group.map(str::to_owned))
        .is_checked(value)
        .on_click(move |e| {
            e.set_handled(true);
            action();
        })
        .into()
}
pub fn separator() -> Element {
    ferroui_view::prelude::separator()
        .class("reader-menu-separator")
        .into_element()
}
pub fn button(vm: &AppViewModel, label: &str, items: Vec<Element>) -> Element {
    attach(vm, reader_controls::dropdown(label), items)
}
pub fn context(vm: &AppViewModel, items: Vec<Element>) -> Element {
    let width = (vm.viewport() - 16.0).max(1.0);
    let mut menu = context_menu()
        .name("ReaderMenu")
        .with(crate::transient::attach_menu)
        .class(if crate::responsive::touch(vm) {
            "reader-touch"
        } else {
            "reader-desktop"
        })
        .placement(ferroui_controls::PlacementMode::BottomEdgeAlignedLeft)
        .min_width(160.0_f64.min(width))
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
        .with(opening::track)
        .on_click(move |e| {
            if let Some(button) = e.source().and_then(|s| s.cast::<Control>()) {
                request(&button);
            }
            e.set_handled(true);
        })
        .on_key_down(move |event| {
            if matches!(event.key, Key::Down | Key::Up) && event.key_modifiers.is_empty() {
                if let Some(button) = event.source().and_then(|source| source.cast::<Control>()) {
                    event.set_handled(true);
                    opening::with_selection(&button, event.key == Key::Down);
                }
            }
        })
        .into()
}

fn request(control: &Control) {
    let event = ContextRequestedEventArgs::new();
    event.set_placement_rect(control, Rect::from_size(control.bounds().size()));
    control.raise_event(&event);
}
