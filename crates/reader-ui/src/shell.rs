use crate::{
    dialogs, header, home, keyboard,
    rows::Rows,
    session::{Panel, Session},
    sidebar, toolbar,
};
use ferroui_base::layout::{HorizontalAlignment as H, VerticalAlignment as V};
use ferroui_base::media::TextWrapping;
use ferroui_view::prelude::*;
use reader_controls::{icon_action, Icon};
use reader_domain::View;
use reader_state::AppViewModel;
use std::rc::Rc;

pub fn view(vm: Rc<AppViewModel>, ui: Rc<Session>, rows: Rc<Rows>) -> Element {
    let narrow = vm.viewport() < 700.0;
    let focus = ui.focus_mode.get();
    let show_sidebar = !narrow && ui.desktop_sidebar.get() && !focus;
    let width = if show_sidebar {
        sidebar::width(vm.state().settings.density)
    } else {
        0.0
    };
    let keys_vm = vm.clone();
    let keys_ui = ui.clone();
    let close = vm.clone();
    let undo = vm.clone();
    let menu = vm.clone();
    let mut root = grid()
        .name("ReaderApp")
        .background("White")
        .row_definitions(if focus { "0,*" } else { "60,*" })
        .focusable(true)
        .is_tab_stop(false)
        .with(move |root| keyboard::attach(root, keys_vm.clone(), keys_ui.clone()));
    if !focus {
        root = root.child(header::view(vm.clone(), ui.clone()));
    }
    let mut layout = grid().grid_row(1).column_definitions(&format!("{width},*"));
    if show_sidebar {
        layout = layout.child(sidebar::view(vm.clone(), ui.clone()));
    }
    let mut main = grid().grid_column(1).row_definitions("Auto,*,Auto");
    let mut top = grid().column_definitions(if narrow || !show_sidebar {
        "40,*"
    } else {
        "0,*"
    });
    if narrow || !show_sidebar {
        let side = ui.clone();
        top = top.child(
            border()
                .vertical_alignment(V::Top)
                .margin((4.0, 14.0, 0.0, 0.0))
                .child(icon_action(Icon::Menu, "Show subscriptions", move || {
                    if menu.viewport() < 700.0 {
                        menu.toggle_sidebar();
                    } else {
                        side.desktop_sidebar.update(|v| *v = !*v);
                        side.focus_mode.set(false);
                    }
                })),
        );
    }
    top = top.child(border().grid_column(1).child(
        if matches!(vm.view(), View::Home | View::Trends | View::Explore) {
            toolbar::page_header(vm.clone(), ui.clone())
        } else {
            toolbar::view(vm.clone(), ui.clone())
        },
    ));
    main = main.child(top);
    if matches!(vm.view(), View::Home | View::Trends | View::Explore) {
        main = main.child(grid().grid_row(1).child(home::view(vm.clone(), ui.clone())));
    } else if vm.view() == View::Settings {
        main = main.child(
            grid()
                .grid_row(1)
                .child(crate::settings::preferences(vm.clone(), ui.clone())),
        );
    } else {
        let reading_vm = vm.clone();
        let reading_ui = ui.clone();
        let mut stream = grid().grid_row(1).clip_to_bounds(true).child(
            list_box()
                .name("ArticleList")
                .with(move |list| {
                    crate::reading::attach(list, reading_vm.clone(), reading_ui.clone())
                })
                .node_ref(ui.article_list)
                .class("reader-list")
                .automation_properties_name("Articles")
                .items_source(rows.source())
                .item_template(rows.template.clone())
                .border_thickness(0.0)
                .padding(0.0),
        );
        if rows.len() == 0 {
            let all = vm.clone();
            let add = ui.clone();
            stream=stream.child(rsx!{StackPanel{margin:(30.0,40.0),spacing:16.0,horizontal_alignment:H::Left,
                TextBlock{text:if matches!(vm.view(),View::Search(_)){"No matching items"}else if vm.state().settings.unread_only{"No unread items"}else{"No items to display"},font_size:22.0}
                TextBlock{text:if matches!(vm.view(),View::Search(_)){"Try another search. You can use title:, author:, feed:, tag:, is:starred and is:unread."}else{"You're all caught up. Refresh your subscriptions or add something new to read."},font_size:13.0,foreground:"#666666",text_wrapping:TextWrapping::Wrap,max_width:550.0}
                {reader_controls::link("Show all items",move||{all.set_unread_only(false);all.navigate(View::All);})}
                {reader_controls::link("Add a subscription",move||add.show(Panel::Subscribe))}
            }});
        }
        main = main.child(stream);
    }
    if !vm.status().is_empty() {
        main=main.child(rsx!{
        Border{Grid.row:2,background:"#FFF8CE",border_brush:"#F0C36D",border_thickness:(0.0,1.0,0.0,0.0),padding:(14.0,7.0),
            Grid{column_definitions:"*,Auto,Auto",column_spacing:12.0,
                TextBlock{text:vm.status(),font_size:12.0,text_wrapping:TextWrapping::Wrap,vertical_alignment:V::Center}
                if vm.can_undo(){Border{Grid.column:1,{reader_controls::link("Undo",move||undo.undo())}}}
                Border{Grid.column:2,{reader_controls::link("Dismiss",move||close.clear_status())}}
            }
        }
    });
    }
    layout = layout.child(main);
    root = root.child(layout);
    if narrow && vm.sidebar_open() {
        let dismiss = vm.clone();
        root=root.child(rsx!{Grid{Grid.row:1,
        Button{class:"reader-flat",background:"#55000000",horizontal_alignment:H::Stretch,vertical_alignment:V::Stretch,automation_properties_name:"Close subscriptions",on_click:move |_|dismiss.set_sidebar_open(false)}
        Border{width:264.0,horizontal_alignment:H::Left,background:"White",{sidebar::view(vm.clone(),ui.clone())}}
    }});
    }
    if ui.panel.get() != Panel::None {
        root = root.child(grid().grid_row_span(2).child(dialogs::view(vm, ui)));
    }
    root.into()
}
