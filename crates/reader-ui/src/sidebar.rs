use crate::{
    menus,
    session::{Panel, Session},
};
use ferroui_base::layout::{HorizontalAlignment as H, VerticalAlignment as V};
use ferroui_base::media::{FontWeight, TextTrimming};
use ferroui_controls::primitives::ScrollBarVisibility;
use ferroui_view::prelude::*;
use reader_controls::{icon, Icon};
use reader_domain::{Density, Feed, View};
use reader_state::AppViewModel;
use std::{collections::HashMap, rc::Rc};

pub fn width(density: Density) -> f64 {
    match density {
        Density::Comfortable => 264.0,
        Density::Cozy => 248.0,
        Density::Compact => 236.0,
    }
}
pub fn view(vm: Rc<AppViewModel>, ui: Rc<Session>) -> Element {
    let density = vm.state().settings.density;
    let inset = match density {
        Density::Comfortable => 44.0,
        Density::Cozy => 28.0,
        Density::Compact => 16.0,
    };
    let subscribe = ui.clone();
    let home = vm.clone();
    let manage = ui.clone();
    let mut content = stack_panel().spacing(0.0);
    content = content.child(nav(vm.clone(), View::Home, "Home", None, 0, inset));
    content = content.child(nav(
        vm.clone(),
        View::All,
        "All items",
        Some(Icon::Feed),
        vm.count(&View::All),
        inset,
    ));
    content = content.child(nav(
        vm.clone(),
        View::Starred,
        "Starred items",
        Some(Icon::Starred),
        vm.count(&View::Starred),
        inset,
    ));
    content = content.child(nav(
        vm.clone(),
        View::Trends,
        "Trends",
        None,
        0,
        inset + 22.0,
    ));
    content = content.child(nav(
        vm.clone(),
        View::Explore,
        "Browse for stuff",
        None,
        0,
        inset + 22.0,
    ));
    content = content.child(border().height(18.0));
    content = content.child(
        text_block()
            .text("Subscriptions")
            .font_size(13.0)
            .font_weight(FontWeight::Bold)
            .margin((inset, 6.0, 0.0, 9.0)),
    );
    let mut counts: HashMap<u64, usize> = HashMap::new();
    {
        let state = vm.state();
        for article in &state.articles {
            if !article.read {
                *counts.entry(article.feed_id).or_default() += 1;
            }
        }
    }
    let feeds = vm.feeds();
    let show_all = vm.state().settings.show_all_subscriptions;
    for feed in feeds.iter().filter(|f| f.folder.is_empty()) {
        let count = counts.get(&feed.id).copied().unwrap_or(0);
        if show_all || count > 0 {
            content = content.child(feed_row(vm.clone(), ui.clone(), feed, count, inset + 6.0));
        }
    }
    let collapsed = ui.collapsed.get();
    for folder in vm.folders() {
        let folder_feeds: Vec<_> = feeds.iter().filter(|f| f.folder == folder).collect();
        let count: usize = folder_feeds
            .iter()
            .map(|f| counts.get(&f.id).copied().unwrap_or(0))
            .sum();
        if !show_all && count == 0 {
            continue;
        }
        let folder_vm = vm.clone();
        let folder_name = folder.clone();
        let disclosure_ui = ui.clone();
        let disclosure_folder = folder.clone();
        let selected = vm.view() == View::Folder(folder.clone());
        content=content.child(rsx! {
            Grid {column_definitions:"20,*",margin:(inset-18.0,0.0,8.0,0.0),height:27.0,
                Button {class:"reader-flat",automation_properties_name:format!("Toggle {folder}"),on_click:move |_|disclosure_ui.toggle_folder(&disclosure_folder),
                    {icon(if collapsed.contains(&folder){Icon::ChevronRight}else{Icon::ChevronDown},8.0,"#777777")}
                }
                Button {Grid.column:1,class:if selected {"reader-nav selected"}else{"reader-nav"},horizontal_content_alignment:H::Stretch,automation_properties_name:folder.clone(),on_click:move |_|folder_vm.navigate(View::Folder(folder_name.clone())),
                    Grid {column_definitions:"20,*,Auto",
                        {icon(Icon::Folder,14.0,"#A1A1A1")}
                        TextBlock {Grid.column:1,text:folder.clone(),text_trimming:<dyn TextTrimming>::character_ellipsis(),vertical_alignment:V::Center,font_weight:if count>0{FontWeight::Bold}else{FontWeight::Normal}}
                        TextBlock {Grid.column:2,text:if count>0{format!("({count})")}else{String::new()},font_size:11.0,foreground:"#666666",margin:(4.0,0.0,8.0,0.0),vertical_alignment:V::Center}
                    }
                }
            }
        });
        if !collapsed.contains(&folder) {
            for feed in folder_feeds {
                let n = counts.get(&feed.id).copied().unwrap_or(0);
                if show_all || n > 0 {
                    content =
                        content.child(feed_row(vm.clone(), ui.clone(), feed, n, inset + 20.0));
                }
            }
        }
    }
    let tags = vm.tags();
    if !tags.is_empty() {
        content = content.child(border().height(15.0));
        for tag in tags {
            content = content.child(nav(
                vm.clone(),
                View::Tag(tag.clone()),
                &tag,
                Some(Icon::Tag),
                vm.count(&View::Tag(tag.clone())),
                inset,
            ));
        }
    }
    rsx! {
        Grid {name:"NavigationPane",row_definitions:if density==Density::Compact{"43,43,*,40"}else{"57,57,*,40"},background:"White",
            Button {class:"reader-link",content:"Reader",font_size:20.0,foreground:"#DD4B39",margin:(inset,0.0,0.0,0.0),vertical_alignment:V::Center,on_click:move |_|home.navigate(View::Home)}
            Button {Grid.row:1,class:"reader-subscribe",name:"SubscribeButton",content:"SUBSCRIBE",height:27.0,min_height:27.0,min_width:90.0,horizontal_alignment:H::Left,vertical_alignment:V::Top,margin:(inset,if density==Density::Compact{8.0}else{22.0},0.0,0.0),automation_properties_name:"Subscribe",on_click:move |_|subscribe.show(Panel::Subscribe)}
            ScrollViewer {Grid.row:2,horizontal_scroll_bar_visibility:ScrollBarVisibility::Disabled,vertical_scroll_bar_visibility:ScrollBarVisibility::Auto,
                {content}
            }
            Border {Grid.row:3,margin:(inset,0.0,0.0,0.0),vertical_alignment:V::Center,
                {reader_controls::link("Manage subscriptions »",move||manage.show(Panel::Subscriptions))}
            }
        }
    }
}
fn nav(
    vm: Rc<AppViewModel>,
    view: View,
    label: &str,
    kind: Option<Icon>,
    count: usize,
    inset: f64,
) -> Element {
    let selected = vm.view() == view;
    let model = vm.clone();
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
                .text(format!("({count})"))
                .font_size(11.0)
                .foreground("#666666")
                .vertical_alignment(V::Center),
        );
    }
    border()
        .border_thickness((3.0, 0.0, 0.0, 0.0))
        .border_brush(if selected { "#DD4B39" } else { "Transparent" })
        .child(
            button()
                .class(if selected {
                    "reader-nav selected"
                } else {
                    "reader-nav"
                })
                .height(28.0)
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
fn feed_row(
    vm: Rc<AppViewModel>,
    ui: Rc<Session>,
    feed: &Feed,
    count: usize,
    inset: f64,
) -> Element {
    let id = feed.id;
    let selected = vm.view() == View::Feed(id);
    let go = vm.clone();
    let rename = ui.clone();
    let folders = ui.clone();
    let remove = ui.clone();
    let menu = rsx! {ContextMenu {
        {menus::item("Rename subscription...",move||rename.show(Panel::Rename(id)))}
        {menus::item("Change folders...",move||folders.show(Panel::Move(id)))}
        {menus::item("Unsubscribe...",move||remove.show(Panel::Unsubscribe(id)))}
    }};
    border().border_thickness((3.0,0.0,0.0,0.0)).border_brush(if selected{"#DD4B39"}else{"Transparent"}).child(
        button().class(if selected{"reader-nav selected"}else{"reader-nav"}).height(27.0).padding((inset-3.0,0.0,12.0,0.0)).horizontal_content_alignment(H::Stretch)
        .automation_properties_name(format!("{} ({count} unread)",feed.title)).tool_tip_tip(feed.error.clone().unwrap_or_else(||feed.url.clone())).context_menu(menu)
        .on_click(move |_|{go.navigate(View::Feed(id));if go.viewport()<700.0&&go.sidebar_open(){go.toggle_sidebar();}}).child(rsx!{
            Grid {column_definitions:"20,*,Auto",
                {icon(Icon::Feed,12.0,if feed.error.is_some(){"#DD4B39"}else{"#E89437"})}
                TextBlock {Grid.column:1,text:feed.title.clone(),font_size:13.0,vertical_alignment:V::Center,text_trimming:<dyn TextTrimming>::character_ellipsis(),font_weight:if count>0{FontWeight::Bold}else{FontWeight::Normal}}
                TextBlock {Grid.column:2,text:if count>0{format!("({count})")}else{String::new()},font_size:11.0,foreground:"#666666",margin:(6.0,0.0,0.0,0.0),vertical_alignment:V::Center}
            }
        })
    ).into()
}
