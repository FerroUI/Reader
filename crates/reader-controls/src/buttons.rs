use crate::{icon, Icon};
use ferroui_base::layout::{HorizontalAlignment as H, Orientation, VerticalAlignment as V};
use ferroui_base::media::{FontWeight, TextTrimming, TextWrapping};
use ferroui_view::prelude::*;

pub fn text(value: impl Into<String>, size: f64, color: &str) -> Element {
    rsx! { TextBlock { text: value.into(), font_size:size, foreground:color } }
}
pub fn paragraph(value: impl Into<String>) -> Element {
    rsx! { TextBlock { text:value.into(),font_size:13.0,foreground:"#333333",text_wrapping:TextWrapping::Wrap,line_height:20.0 } }
}
pub fn action(label: &str, on_click: impl Fn() + 'static) -> Element {
    button()
        .class("reader-button")
        .automation_properties_name(label)
        .content(label)
        .on_click(move |e| {
            e.set_handled(true);
            on_click();
        })
        .into()
}
pub fn primary(label: &str, on_click: impl Fn() + 'static) -> Element {
    button()
        .class("reader-primary")
        .automation_properties_name(label)
        .content(label)
        .on_click(move |e| {
            e.set_handled(true);
            on_click();
        })
        .into()
}
pub fn link(label: &str, on_click: impl Fn() + 'static) -> Element {
    button()
        .class("reader-link")
        .automation_properties_name(label)
        .child(text_block().text(label).text_wrapping(TextWrapping::Wrap))
        .on_click(move |e| {
            e.set_handled(true);
            on_click();
        })
        .into()
}
pub fn icon_action(kind: Icon, label: &str, on_click: impl Fn() + 'static) -> Element {
    toolbar_icon(kind, label)
        .on_click(move |e| {
            e.set_handled(true);
            on_click();
        })
        .into()
}
pub fn toolbar_icon(kind: Icon, label: &str) -> El<Button> {
    button()
        .class("reader-button")
        .width(35.0)
        .height(29.0)
        .padding(8.0)
        .automation_properties_name(label)
        .tool_tip_tip(label)
        .child(icon(kind, 14.0, "#666666"))
}
pub fn dropdown(label: &str) -> El<Button> {
    dropdown_with_gap(label, 9.0)
}
/// Compact Reader filters keep their historical 72px outer width with readable labels.
pub fn compact_dropdown(label: &str) -> El<Button> {
    dropdown_with_gap(label, 2.0)
}
fn dropdown_with_gap(label: &str, gap: f64) -> El<Button> {
    button().class("reader-button").horizontal_content_alignment(H::Stretch).automation_properties_name(label).child(rsx! {
        Grid { column_definitions:"*,8",column_spacing:gap,
            TextBlock {text:label,font_size:11.0,font_weight:FontWeight::Bold,text_trimming:<dyn TextTrimming>::character_ellipsis(),vertical_alignment:V::Center}
            Border {Grid.column:1,{icon(Icon::ChevronDown,8.0,"#777777")}}
        }
    })
}
/// An action group can wrap as its parent becomes narrower without losing actions.
pub fn action_row(actions: impl IntoIterator<Item = Element>, spacing: f64) -> Element {
    let mut row = wrap_panel().orientation(Orientation::Horizontal);
    for action in actions {
        row = row.child(border().margin((0.0, 0.0, spacing, spacing)).child(action));
    }
    row.into()
}
pub fn input(label: &str) -> El<TextBox> {
    text_box()
        .class("reader-input")
        .height(29.0)
        .min_width(0.0)
        .line_height(18.0)
        .letter_spacing(0.0)
        .padding((7.0, 4.0))
        .automation_properties_name(label)
}
pub fn separator() -> Element {
    rsx! { Border {height:1.0,background:"#EBEBEB",margin:(0.0,9.0)} }
}
pub fn header(label: &str) -> Element {
    rsx! { TextBlock { text:label,font_size:20.0,font_weight:FontWeight::Normal,foreground:"#333333",margin:(0.0,0.0,0.0,18.0),horizontal_alignment:H::Left } }
}
