use crate::session::{Panel, Session};
use ferroui_base::input::Key;
use ferroui_base::layout::{HorizontalAlignment as H, Orientation, VerticalAlignment as V};
use ferroui_base::media::{FontFamily, FontWeight};
use ferroui_view::prelude::*;
use reader_controls::{icon, Icon};
use reader_domain::View;
use reader_state::AppViewModel;
use std::rc::Rc;

pub fn view(vm: Rc<AppViewModel>, ui: Rc<Session>) -> Element {
    let narrow = vm.viewport() < 700.0;
    let submit_vm = vm.clone();
    let submit_ui = ui.clone();
    let enter_vm = vm.clone();
    let enter_ui = ui.clone();
    let left = if narrow {
        "132".to_string()
    } else {
        crate::sidebar::width(vm.state().settings.density).to_string()
    };
    let mut header = grid()
        .name("SearchHeader")
        .column_definitions(&format!("{left},*,Auto"))
        .height(60.0)
        .background("#F1F1F1")
        .margin(0.0);
    header = header.child(
        border()
            .margin((if narrow { 16.0 } else { 44.0 }, 0.0, 0.0, 0.0))
            .vertical_alignment(V::Center)
            .child(logo()),
    );
    let search = rsx! {
        Grid {Grid.column:1,column_definitions:"*,72",width:if narrow{vm.viewport()-148.0}else{(vm.viewport()-crate::sidebar::width(vm.state().settings.density)-110.0).clamp(180.0,600.0)},max_width:650.0,horizontal_alignment:H::Left,vertical_alignment:V::Center,margin:(0.0,0.0,16.0,0.0),
            {reader_controls::input("Search Reader").name("SearchReader").node_ref(ui.search_input).text(ui.search.two_way()).placeholder_text("Search Reader").min_width(0.0).on_key_down(move |event|{
                if event.key==Key::Return { event.set_handled(true);enter_vm.navigate(View::Search(enter_ui.search.get_untracked())); }
            })}
            Button {Grid.column:1,class:"reader-primary",name:"SearchButton",height:29.0,margin:(16.0,0.0,0.0,0.0),automation_properties_name:"Search Reader",
                on_click:move |_|submit_vm.navigate(View::Search(submit_ui.search.get_untracked())),
                {icon(Icon::Search,15.0,"White")}
            }
        }
    };
    header = header.child(search);
    if !narrow {
        let ui_about = ui.clone();
        header = header.child(
            border()
                .grid_column(2)
                .margin((12.0, 0.0, 28.0, 0.0))
                .vertical_alignment(V::Center)
                .child(reader_controls::link("Reader", move || {
                    ui_about.show(Panel::About)
                })),
        );
    }
    header.into()
}
fn logo() -> Element {
    let mut word = stack_panel()
        .orientation(Orientation::Horizontal)
        .vertical_alignment(V::Center)
        .automation_properties_name("Google");
    for (letter, color) in [
        ("G", "#3369E8"),
        ("o", "#D50F25"),
        ("o", "#EEB211"),
        ("g", "#3369E8"),
        ("l", "#009939"),
        ("e", "#D50F25"),
    ] {
        word = word.child(
            text_block()
                .text(letter)
                .font_family(FontFamily::new("fonts:Reader#Liberation Serif"))
                .font_size(32.0)
                .font_weight(FontWeight::Normal)
                .foreground(color),
        );
    }
    word.into()
}
