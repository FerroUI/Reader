use ferroui_base::layout::{HorizontalAlignment as H, VerticalAlignment as V};
use ferroui_base::media::FontWeight;
use ferroui_view::prelude::*;
use reader_controls::{icon, Icon};
use reader_domain::View;
use reader_state::AppViewModel;
use std::rc::Rc;
pub(super) fn nav(
    vm: Rc<AppViewModel>,
    view: View,
    label: &str,
    kind: Option<Icon>,
    count: usize,
    inset: f64,
) -> Element {
    let selected = vm.view() == view;
    let model = vm.clone();
    let key = format!("view:{view:?}");
    let target = view;
    let mut line = grid()
        .column_definitions(if kind.is_some() {
            "22,*,Auto"
        } else {
            "0,*,Auto"
        })
        .margin((inset - 3.0, 0.0, 12.0, 0.0));
    if let Some(kind) = kind {
        line = line.child(icon(
            kind,
            14.0,
            if kind == Icon::Starred {
                "#E7BB3D"
            } else {
                "#777777"
            },
        ));
    }
    line = line.child(
        text_block()
            .grid_column(1)
            .text(label)
            .vertical_alignment(V::Center)
            .font_weight(if count > 0 || selected {
                FontWeight::Bold
            } else {
                FontWeight::Normal
            }),
    );
    if count > 0 {
        line = line.child(
            text_block()
                .grid_column(2)
                .text(crate::format::unread_badge(count))
                .font_size(11.0)
                .foreground("#666666")
                .vertical_alignment(V::Center),
        );
    }
    border()
        .key(key)
        .border_thickness((3.0, 0.0, 0.0, 0.0))
        .border_brush(if selected { "#DD4B39" } else { "Transparent" })
        .child(
            button()
                .class(if selected {
                    "reader-nav selected"
                } else {
                    "reader-nav"
                })
                .height(32.0)
                .horizontal_content_alignment(H::Stretch)
                .automation_properties_name(label)
                .on_click(move |_| {
                    model.navigate(target.clone());
                    if model.viewport() < 700.0 && model.sidebar_open() {
                        model.toggle_sidebar();
                    }
                })
                .child(line),
        )
        .into()
}
pub(super) fn group(
    vm: Rc<AppViewModel>,
    open: Signal<bool>,
    target: View,
    title: &str,
    count: usize,
    inset: f64,
) -> Element {
    let key = format!("group:{target:?}");
    let selected = vm.view() == target;
    let toggle = open;
    grid()
        .key(key)
        .child(nav(vm, target, title, None, count, inset))
        .child(
            button()
                .class("reader-flat")
                .width(18.0)
                .height(32.0)
                .horizontal_alignment(H::Left)
                .margin(((inset - 22.0).max(0.0), 0.0, 0.0, 0.0))
                .automation_properties_name(format!(
                    "{} {}",
                    if open.get() { "Collapse" } else { "Expand" },
                    title
                ))
                .on_click(move |_| toggle.update(|value| *value = !*value))
                .child(icon(
                    if open.get() {
                        Icon::ChevronDown
                    } else {
                        Icon::ChevronRight
                    },
                    8.0,
                    if selected { "#DD4B39" } else { "#777777" },
                )),
        )
        .into()
}
