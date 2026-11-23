use ferroui_base::input::{Key, KeyModifiers};
use ferroui_base::layout::{HorizontalAlignment as H, VerticalAlignment as V};
use ferroui_base::media::{FontWeight, TextWrapping};
use ferroui_controls::{primitives::ToggleButton, Control, RadioButton};
use ferroui_view::prelude::*;
use reader_controls::action_row;

pub(super) fn heading(label: &str) -> Element {
    text_block()
        .text(label)
        .font_size(13.0)
        .font_weight(FontWeight::Bold)
        .text_wrapping(TextWrapping::Wrap)
        .into()
}

pub(super) fn help(value: impl Into<String>) -> Element {
    text_block()
        .text(value.into())
        .font_size(12.0)
        .line_height(18.0)
        .foreground("#666666")
        .text_wrapping(TextWrapping::Wrap)
        .into()
}

pub(super) fn check(
    label: &str,
    name: &str,
    selected: bool,
    touch: bool,
    change: impl Fn(bool) + 'static,
) -> El<CheckBox> {
    check_box()
        .key(name.to_owned())
        .name(name.to_owned())
        .theme(Some(reader_controls::preference_check_theme()))
        .content(label)
        .automation_properties_name(label)
        .font_size(13.0)
        .foreground("#333333")
        .min_width(0.0)
        .min_height(if touch { 44.0 } else { 28.0 })
        .padding(0.0)
        .horizontal_alignment(H::Stretch)
        .horizontal_content_alignment(H::Stretch)
        .vertical_content_alignment(V::Center)
        .is_checked(Some(selected))
        .on_changed(ToggleButton::is_checked_property(), move |value| {
            change(value.unwrap_or(false))
        })
}

pub(super) fn choices(group: &str, items: Vec<Element>) -> Element {
    border()
        .name(format!("SettingsChoices-{group}"))
        .child(action_row(items, 8.0))
        .into()
}

pub(super) fn choice(
    group: &str,
    label: &str,
    selected: bool,
    touch: bool,
    activate: impl Fn() + 'static,
) -> Element {
    let name = format!("SettingsChoices-{group}");
    let loaded_group = name.clone();
    radio_button()
        .key(format!("{group}:{label}"))
        .theme(Some(reader_controls::preference_choice_theme()))
        .content(label)
        .automation_properties_name(label)
        .is_checked(Some(selected))
        .is_tab_stop(selected)
        .min_width(0.0)
        .min_height(if touch { 44.0 } else { 29.0 })
        .foreground("#444444")
        .horizontal_content_alignment(H::Center)
        .vertical_content_alignment(V::Center)
        // A full-page settings route and its modal can coexist. Each retained
        // group owns its native exclusivity rather than sharing a global name.
        .on_loaded(move |event| {
            let Some(option) = event
                .source()
                .and_then(|source| source.cast::<RadioButton>())
            else {
                return;
            };
            if let Some(group) = option.find_ancestor_of_type_where::<Control>(false, |control| {
                control.has_name(&loaded_group)
            }) {
                option.set_group_name(Some(format!("reader-settings:{:p}", &*group)));
            }
        })
        .on_changed(ToggleButton::is_checked_property(), move |value| {
            if *value == Some(true) {
                activate();
            }
        })
        .on_key_down(move |event| {
            if !event.key_modifiers.is_empty() {
                return;
            }
            let step = match event.key {
                Key::Left | Key::Up => -1,
                Key::Right | Key::Down => 1,
                _ => return,
            };
            let Some(current) = event
                .source()
                .and_then(|source| source.cast::<RadioButton>())
            else {
                return;
            };
            let Some(group) = current
                .find_ancestor_of_type_where::<Control>(false, |control| control.has_name(&name))
            else {
                return;
            };
            let options = group
                .get_visual_descendants()
                .filter_map(|visual| visual.cast::<RadioButton>())
                .collect::<Vec<_>>();
            let Some(index) = options.iter().position(|option| *option == current) else {
                return;
            };
            let next =
                &options[(index as isize + step).rem_euclid(options.len() as isize) as usize];
            event.set_handled(true);
            next.focus_with(
                ferroui_base::input::NavigationMethod::Directional,
                KeyModifiers::NONE,
            );
            next.set_current_value(ToggleButton::is_checked_property(), Some(true));
        })
        .into()
}
