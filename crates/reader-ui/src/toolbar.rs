use crate::{
    menus,
    session::{Panel, Session},
};
use ferroui_base::layout::{Orientation, VerticalAlignment as V};
use ferroui_base::media::FontWeight;
use ferroui_view::prelude::*;
use reader_controls::{icon_action, toolbar_icon, Icon};
use reader_domain::{Density, SortOrder, View, ViewMode};
use reader_state::AppViewModel;
use std::rc::Rc;

pub fn view(vm: Rc<AppViewModel>, ui: Rc<Session>) -> Element {
    let compact = vm.viewport() < 1000.0;
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
        .height(if settings.density == Density::Compact {
            43.0
        } else {
            57.0
        })
        .margin((3.0, 0.0, 38.0, 0.0));
    let mut actions = stack_panel()
        .orientation(Orientation::Horizontal)
        .spacing(if compact { 4.0 } else { 18.0 })
        .vertical_alignment(V::Center)
        .child(
            toolbar_icon(
                Icon::Refresh,
                if vm.is_refreshing() {
                    "Refreshing feeds"
                } else {
                    "Refresh"
                },
            )
            .width(if compact { 35.0 } else { 72.0 })
            .on_click(move |_| refresh.refresh()),
        )
        .child(filter_menu(vm.clone()));
    if compact {
        actions = actions.child(menus::button("More", more_items(vm.clone(), ui.clone())));
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
        toolbar_icon(Icon::Settings, "Settings").width(if compact { 35.0 } else { 72.0 }),
        settings_items(vm.clone(), ui),
    ));
    tools = tools.child(nav);
    rsx! { Grid {row_definitions:"Auto,30",background:"White",
        Border {border_brush:"#EBEBEB",border_thickness:(0.0,0.0,0.0,1.0),{tools}}
        TextBlock {Grid.row:1,text:title,font_size:14.0,font_weight:FontWeight::Bold,foreground:"#333333",margin:(6.0,5.0,12.0,0.0),vertical_alignment:V::Center}
    }}
}
/// Overview, Trends and discovery have a section header rather than stream actions.
pub fn page_header(vm: Rc<AppViewModel>, ui: Rc<Session>) -> Element {
    let height = if vm.state().settings.density == Density::Compact {
        43.0
    } else {
        57.0
    };
    rsx! { Border {height,border_brush:"#EBEBEB",border_thickness:(0.0,0.0,0.0,1.0),
        Grid {column_definitions:"*,Auto",margin:(13.0,0.0,38.0,0.0),
            TextBlock {text:vm.title(),font_size:13.0,foreground:"#777777",vertical_alignment:V::Center}
            Border {Grid.column:1,vertical_alignment:V::Center,
                {menus::attach(toolbar_icon(Icon::Settings,"Settings").width(72.0),settings_items(vm,ui))}
            }
        }
    }}
}
fn filter_menu(vm: Rc<AppViewModel>) -> Element {
    let unread = vm.state().settings.unread_only;
    let all = vm.clone();
    let fresh = vm.clone();
    menus::attach(
        reader_controls::dropdown(if unread { "New items" } else { "All items" })
            .width(72.0)
            .padding((6.0, 5.0)),
        vec![
            menus::checked("All items", !unread, move || all.set_unread_only(false)),
            menus::checked("New items", unread, move || fresh.set_unread_only(true)),
        ],
    )
}
fn mark_menu(vm: Rc<AppViewModel>, ui: Rc<Session>) -> Element {
    let mark = vm.clone();
    let confirm = ui.clone();
    rsx! {Grid{column_definitions:"99,20",height:29.0,
        Button{class:"reader-button",padding:(6.0,5.0),content:"Mark all as read",automation_properties_name:"Mark all as read",on_click:move |_|{
            if mark.state().settings.confirm_mark_all{confirm.show(Panel::MarkAll);}else{mark.mark_all_read(None);}
        }}
        Border{Grid.column:1,{menus::attach(reader_controls::toolbar_icon(Icon::ChevronDown,"Mark all read options").width(20.0).padding(5.0),mark_items(vm,ui))}}
    }}
}
fn mark_items(vm: Rc<AppViewModel>, ui: Rc<Session>) -> Vec<Element> {
    let all = vm.clone();
    let day = vm.clone();
    let week = vm.clone();
    let fortnight = vm.clone();
    vec![
        menus::item("All items", move || {
            if all.state().settings.confirm_mark_all {
                ui.show(Panel::MarkAll);
            } else {
                all.mark_all_read(None);
            }
        }),
        menus::item("Items older than a day", move || {
            day.mark_all_read(Some(day.now() - 86400))
        }),
        menus::item("Items older than a week", move || {
            week.mark_all_read(Some(week.now() - 7 * 86400))
        }),
        menus::item("Items older than two weeks", move || {
            fortnight.mark_all_read(Some(fortnight.now() - 14 * 86400))
        }),
    ]
}
fn sort_menu(vm: Rc<AppViewModel>, ui: Rc<Session>) -> Element {
    let label = if matches!(vm.view(), View::Feed(_)) {
        "Feed settings"
    } else {
        "View settings"
    };
    menus::attach(
        reader_controls::dropdown(label).width(115.0),
        sort_items(vm, ui),
    )
}
fn sort_items(vm: Rc<AppViewModel>, ui: Rc<Session>) -> Vec<Element> {
    let order = vm.state().settings.sort;
    let newest = vm.clone();
    let oldest = vm.clone();
    let mut items = vec![
        menus::checked("Sort by newest", order == SortOrder::Newest, move || {
            newest.set_sort(SortOrder::Newest)
        }),
        menus::checked("Sort by oldest", order == SortOrder::Oldest, move || {
            oldest.set_sort(SortOrder::Oldest)
        }),
    ];
    if let View::Feed(id) = vm.view() {
        let rename = ui.clone();
        let folders = ui.clone();
        let unsubscribe = ui.clone();
        items.push(menus::separator());
        items.push(menus::item("Rename subscription...", move || {
            rename.show(Panel::Rename(id))
        }));
        items.push(menus::item("Change folders...", move || {
            folders.show(Panel::Move(id))
        }));
        items.push(menus::item("Unsubscribe...", move || {
            unsubscribe.show(Panel::Unsubscribe(id))
        }));
    }
    items
}
fn more_items(vm: Rc<AppViewModel>, ui: Rc<Session>) -> Vec<Element> {
    let list = vm.clone();
    let expanded = vm.clone();
    let mut items = vec![
        menus::item("List view", move || list.set_view_mode(ViewMode::List)),
        menus::item("Expanded view", move || {
            expanded.set_view_mode(ViewMode::Expanded)
        }),
        menus::separator(),
    ];
    items.extend(mark_items(vm.clone(), ui.clone()));
    items.push(menus::separator());
    items.extend(sort_items(vm, ui));
    items
}
fn settings_items(vm: Rc<AppViewModel>, ui: Rc<Session>) -> Vec<Element> {
    let pref = ui.clone();
    let sub = ui.clone();
    let imports = ui.clone();
    let keys = ui.clone();
    let about = ui.clone();
    let comfortable = vm.clone();
    let cozy = vm.clone();
    let compact = vm.clone();
    let density = vm.state().settings.density;
    vec![
        menus::item("Reader settings", move || pref.show(Panel::Preferences)),
        menus::item("Subscriptions", move || sub.show(Panel::Subscriptions)),
        menus::item("Import/Export", move || imports.show(Panel::ImportExport)),
        menus::separator(),
        menus::checked("Comfortable", density == Density::Comfortable, move || {
            comfortable.set_density(Density::Comfortable)
        }),
        menus::checked("Cozy", density == Density::Cozy, move || {
            cozy.set_density(Density::Cozy)
        }),
        menus::checked("Compact", density == Density::Compact, move || {
            compact.set_density(Density::Compact)
        }),
        menus::separator(),
        menus::item("Keyboard shortcuts", move || keys.show(Panel::Shortcuts)),
        menus::item("About Reader", move || about.show(Panel::About)),
    ]
}
