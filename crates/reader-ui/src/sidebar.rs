mod feed;
mod navigation;
use crate::session::{Panel, Session};
use feed::feed_row;
use ferroui_base::layout::{HorizontalAlignment as H, VerticalAlignment as V};
use ferroui_base::media::{FontWeight, TextTrimming};
use ferroui_controls::primitives::ScrollBarVisibility;
use ferroui_view::prelude::*;
use navigation::{group, nav};
use reader_controls::{icon, Icon};
use reader_domain::{Density, View};
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
    let manage = ui.clone();
    let mut content = stack_panel().spacing(0.0);
    content = content.child(nav(vm.clone(), View::Home, "Home", None, 0, inset));
    content = content.child(group(
        vm.clone(),
        ui.all_open,
        View::All,
        "All items",
        vm.count(&View::All),
        inset,
    ));
    if ui.all_open.get() {
        content = content.child(nav(
            vm.clone(),
            View::Starred,
            "Starred items",
            Some(Icon::Starred),
            vm.count(&View::Starred),
            inset + 20.0,
        ));
        content = content.child(nav(
            vm.clone(),
            View::Trends,
            "Trends",
            None,
            0,
            inset + 20.0,
        ));
    }
    content = content.child(border().key("explore-gap").height(12.0));
    content = content.child(group(
        vm.clone(),
        ui.explore_open,
        View::Explore,
        "Explore",
        0,
        inset,
    ));
    if ui.explore_open.get() {
        content = content.child(nav(
            vm.clone(),
            View::Explore,
            "Browse for stuff",
            None,
            0,
            inset + 20.0,
        ));
    }
    let unfiled_vm = vm.clone();
    content = content.child(
        text_block()
            .key("subscriptions-heading")
            .with(move |heading| {
                crate::drag::attach_target(
                    &heading.clone().upcast(),
                    unfiled_vm.clone(),
                    String::new(),
                )
            })
            .text("Subscriptions")
            .font_size(13.0)
            .font_weight(FontWeight::Bold)
            .height(32.0)
            .padding((inset, 8.0, 0.0, 0.0)),
    );
    let feeds = vm.feeds();
    let counts: HashMap<u64, usize> = feeds
        .iter()
        .map(|feed| (feed.id, vm.count(&View::Feed(feed.id))))
        .collect();
    let show_all = vm.state().settings.show_all_subscriptions;
    for feed in feeds.iter().filter(|f| f.folder_names().is_empty()) {
        let count = counts.get(&feed.id).copied().unwrap_or(0);
        if show_all || count > 0 {
            content = content.child(feed_row(
                vm.clone(),
                ui.clone(),
                feed,
                count,
                inset + 6.0,
                "",
            ));
        }
    }
    let collapsed = ui.collapsed.get();
    for folder in vm.folders() {
        let folder_feeds: Vec<_> = feeds.iter().filter(|f| f.in_folder(&folder)).collect();
        let count: usize = folder_feeds
            .iter()
            .map(|f| counts.get(&f.id).copied().unwrap_or(0))
            .sum();
        if !show_all && count == 0 {
            continue;
        }
        let drop_vm = vm.clone();
        let drop_folder = folder.clone();
        let folder_vm = vm.clone();
        let folder_name = folder.clone();
        let disclosure_ui = ui.clone();
        let disclosure_folder = folder.clone();
        let selected = vm.view() == View::Folder(folder.clone());
        content=content.child(rsx! {
            Grid {key:format!("folder:{folder}"),with:move |target|crate::drag::attach_target(&target.clone().upcast(),drop_vm.clone(),drop_folder.clone()),column_definitions:"20,*",margin:(inset-18.0,0.0,8.0,0.0),height:27.0,
                Button {class:"reader-flat",automation_properties_name:format!("Toggle {folder}"),on_click:move |_|disclosure_ui.toggle_folder(&disclosure_folder),
                    {icon(if collapsed.contains(&folder){Icon::ChevronRight}else{Icon::ChevronDown},8.0,"#777777")}
                }
                Button {Grid.column:1,class:if selected {"reader-nav selected"}else{"reader-nav"},horizontal_content_alignment:H::Stretch,automation_properties_name:folder.clone(),on_click:move |_|folder_vm.navigate(View::Folder(folder_name.clone())),
                    Grid {column_definitions:"20,*,Auto",
                        {icon(Icon::Folder,14.0,"#A1A1A1")}
                        TextBlock {Grid.column:1,text:folder.clone(),text_trimming:<dyn TextTrimming>::character_ellipsis(),vertical_alignment:V::Center,font_weight:if count>0{FontWeight::Bold}else{FontWeight::Normal}}
                        TextBlock {Grid.column:2,text:crate::format::unread_badge(count),font_size:11.0,foreground:"#666666",margin:(4.0,0.0,8.0,0.0),vertical_alignment:V::Center}
                    }
                }
            }
        });
        if !collapsed.contains(&folder) {
            for feed in folder_feeds {
                let n = counts.get(&feed.id).copied().unwrap_or(0);
                if show_all || n > 0 {
                    content = content.child(feed_row(
                        vm.clone(),
                        ui.clone(),
                        feed,
                        n,
                        inset + 20.0,
                        &folder,
                    ));
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
        Grid {key:"navigation-pane",name:"NavigationPane",row_definitions:if density==Density::Compact{"43,*,40"}else{"57,*,40"},background:"White",
            Button {class:"reader-subscribe",name:"SubscribeButton",content:"SUBSCRIBE",height:27.0,min_height:27.0,min_width:90.0,horizontal_alignment:H::Left,vertical_alignment:V::Top,margin:(inset,if density==Density::Compact{8.0}else{17.0},0.0,0.0),automation_properties_name:"Subscribe",on_click:move |_|subscribe.show(Panel::Subscribe)}
            ScrollViewer {Grid.row:1,horizontal_scroll_bar_visibility:ScrollBarVisibility::Disabled,vertical_scroll_bar_visibility:ScrollBarVisibility::Auto,
                {content}
            }
            Border {Grid.row:2,margin:(inset,0.0,0.0,0.0),vertical_alignment:V::Center,
                {reader_controls::link("Manage subscriptions »",move||manage.show(Panel::Subscriptions))}
            }
        }
    }
}
