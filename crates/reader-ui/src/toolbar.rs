mod actions;
use crate::{menus, session::Session};
use actions::{filter_menu, mark_menu, more_items, settings_items, sort_menu};
use ferroui_base::layout::{Orientation, VerticalAlignment as V};
use ferroui_base::media::{FontWeight, TextTrimming};
use ferroui_view::prelude::*;
use reader_controls::{icon_action, toolbar_icon, Icon};
use reader_domain::{Density, View, ViewMode};
use reader_state::AppViewModel;
use std::rc::Rc;

pub fn view(vm: Rc<AppViewModel>, ui: Rc<Session>) -> Element {
    let narrow = crate::responsive::narrow(&vm);
    let touch = crate::responsive::touch(&vm);
    let sidebar = !narrow && ui.desktop_sidebar.get() && !ui.focus_mode.get();
    let available = vm.viewport()
        - if sidebar {
            crate::sidebar::width(vm.state().settings.density)
        } else {
            if touch {
                48.0
            } else {
                40.0
            }
        };
    let compact = available < if touch { 820.0 } else { 730.0 };
    let settings = vm.state().settings.clone();
    let refresh = vm.clone();
    let previous = vm.clone();
    let next = vm.clone();
    let list = vm.clone();
    let expanded = vm.clone();
    let count = vm.count(&vm.view());
    let title = vm.title();
    let title = if count > 0 {
        format!("{title} {}", crate::format::unread_badge(count))
    } else {
        title
    };
    let mut tools = grid()
        .column_definitions("*,Auto")
        .height(if touch {
            56.0
        } else if settings.density == Density::Compact {
            43.0
        } else {
            57.0
        })
        .margin((
            if narrow { 0.0 } else { 3.0 },
            0.0,
            if narrow {
                8.0
            } else {
                crate::article::right_inset(settings.density)
            },
            0.0,
        ));
    let mut actions = stack_panel()
        .orientation(Orientation::Horizontal)
        .spacing(if compact { 4.0 } else { 18.0 })
        .vertical_alignment(V::Center)
        .margin((0.0, if touch { 0.0 } else { 6.0 }, 0.0, 0.0))
        .child(
            toolbar_icon(
                Icon::Refresh,
                if vm.is_refreshing() {
                    "Refreshing feeds"
                } else {
                    "Refresh"
                },
            )
            .width(if compact {
                if touch {
                    44.0
                } else {
                    35.0
                }
            } else {
                72.0
            })
            .on_click(move |_| refresh.refresh()),
        )
        .child(filter_menu(vm.clone()));
    if compact {
        actions = actions.child(menus::attach(
            &vm,
            toolbar_icon(Icon::ChevronDown, "More actions").width(if touch { 44.0 } else { 35.0 }),
            more_items(vm.clone(), ui.clone()),
        ));
    } else {
        actions = actions
            .child(mark_menu(vm.clone(), ui.clone()))
            .child(sort_menu(vm.clone(), ui.clone()));
    }
    tools = tools.child(actions);
    let mut nav = stack_panel()
        .grid_column(1)
        .orientation(Orientation::Horizontal)
        .vertical_alignment(V::Center)
        .margin((0.0, if touch { 0.0 } else { 6.0 }, 0.0, 0.0))
        .spacing(0.0);
    if !compact {
        nav = nav
            .child(
                toolbar_icon(Icon::Expanded, "Expanded view (1)")
                    .background(if settings.view_mode == ViewMode::Expanded {
                        "#D9D9D9"
                    } else {
                        "#F3F3F3"
                    })
                    .on_click(move |_| expanded.set_view_mode(ViewMode::Expanded)),
            )
            .child(
                toolbar_icon(Icon::List, "List view (2)")
                    .background(if settings.view_mode == ViewMode::List {
                        "#D9D9D9"
                    } else {
                        "#F3F3F3"
                    })
                    .on_click(move |_| list.set_view_mode(ViewMode::List)),
            )
            .child(border().width(18.0));
    }
    if !compact {
        nav = nav
            .child(icon_action(
                Icon::ChevronUp,
                "Previous item (k)",
                move || previous.next_article(-1),
            ))
            .child(icon_action(Icon::ChevronDown, "Next item (j)", move || {
                next.next_article(1)
            }))
            .child(border().width(18.0));
    }
    nav = nav.child(menus::attach(
        &vm,
        toolbar_icon(Icon::Settings, "Settings").width(if compact { 35.0 } else { 72.0 }),
        settings_items(vm.clone(), ui),
    ));
    tools = tools.child(nav);
    rsx! { Grid {row_definitions:"Auto,34",background:"White",
        Border {border_brush:"#EBEBEB",border_thickness:(0.0,0.0,0.0,1.0),{tools}}
        TextBlock {Grid.row:1,text:title,font_size:14.0,font_weight:FontWeight::Bold,foreground:"#333333",text_trimming:<dyn TextTrimming>::character_ellipsis(),margin:(6.0,5.0,12.0,0.0),vertical_alignment:V::Center,tool_tip_tip:vm.title()}
    }}
}
/// Overview, Trends and discovery have a section header rather than stream actions.
pub fn page_header(vm: Rc<AppViewModel>, ui: Rc<Session>) -> Element {
    let touch = crate::responsive::touch(&vm);
    let narrow = crate::responsive::narrow(&vm);
    let height = if touch {
        56.0
    } else if vm.state().settings.density == Density::Compact {
        43.0
    } else {
        57.0
    };
    rsx! { Border {height,border_brush:"#EBEBEB",border_thickness:(0.0,0.0,0.0,1.0),
        Grid {column_definitions:"*,Auto",column_spacing:8.0,margin:(if narrow{0.0}else{13.0},0.0,if narrow{8.0}else{crate::article::right_inset(vm.state().settings.density)},0.0),
            TextBlock {text:if vm.view()==View::Home{if narrow{"Home".to_string()}else{"A look at what's new in Reader".to_string()}}else{vm.title()},font_size:13.0,foreground:"#777777",text_trimming:<dyn TextTrimming>::character_ellipsis(),vertical_alignment:V::Center}
            Border {Grid.column:1,vertical_alignment:V::Center,margin:(0.0,if touch{0.0}else{6.0},0.0,0.0),
                {menus::attach(&vm,toolbar_icon(Icon::Settings,"Settings").width(if narrow{44.0}else{72.0}),settings_items(vm.clone(),ui))}
            }
        }
    }}
}
/// Always reachable reading navigation on a phone, without covering article text.
pub fn reading_navigation(vm: Rc<AppViewModel>) -> Element {
    let previous = vm.clone();
    let next = vm.clone();
    let mode = vm.state().settings.view_mode;
    let switch = vm.clone();
    let enabled = vm.visible_len() > 0;
    rsx! {Border{background:"#FAFAFA",border_brush:"#DDDDDD",border_thickness:(0.0,1.0,0.0,0.0),padding:(8.0,2.0),
        Grid{column_definitions:"*,44,*",column_spacing:8.0,
            Button{class:"reader-flat",height:44.0,content:"‹ Previous",horizontal_content_alignment:ferroui_base::layout::HorizontalAlignment::Center,automation_properties_name:"Previous item",is_enabled:enabled,on_click:move |_|previous.next_article(-1)}
            Button{Grid.column:1,class:"reader-flat",height:44.0,automation_properties_name:if mode==ViewMode::List{"Expanded view"}else{"List view"},on_click:move |_|switch.set_view_mode(if mode==ViewMode::List{ViewMode::Expanded}else{ViewMode::List}),
                {reader_controls::icon(if mode==ViewMode::List{Icon::Expanded}else{Icon::List},16.0,"#666666")}
            }
            Button{Grid.column:2,class:"reader-flat",height:44.0,content:"Next ›",horizontal_content_alignment:ferroui_base::layout::HorizontalAlignment::Center,automation_properties_name:"Next item",is_enabled:enabled,on_click:move |_|next.next_article(1)}
        }
    }}
}
