pub(crate) mod cursor;
mod feed;
pub(crate) mod folder;
mod navigation;
pub(crate) mod tree;
use crate::session::{Panel, Session};
use feed::feed_row;
use ferroui_base::layout::{HorizontalAlignment as H, VerticalAlignment as V};
use ferroui_base::media::FontWeight;
use ferroui_controls::primitives::ScrollBarVisibility;
use ferroui_view::prelude::*;
use navigation::{group, nav};
use reader_controls::{icon, Icon};
use reader_domain::{Density, View};
use reader_state::AppViewModel;
use std::{collections::HashMap, rc::Rc};
use tree::TreeNode;

pub fn width(density: Density) -> f64 {
    match density {
        Density::Comfortable => 264.0,
        Density::Cozy => 248.0,
        Density::Compact => 236.0,
    }
}
pub fn view(vm: Rc<AppViewModel>, ui: Rc<Session>) -> Element {
    let density = vm.state().settings.density;
    let narrow = crate::responsive::narrow(&vm);
    let touch = crate::responsive::touch(&vm);
    let inset = if narrow {
        16.0
    } else {
        match density {
            Density::Comfortable => 44.0,
            Density::Cozy => 28.0,
            Density::Compact => 16.0,
        }
    };
    let subscribe = ui.clone();
    let manage = ui.clone();
    let mut content = stack_panel().spacing(0.0);
    content = content.child(nav(
        vm.clone(),
        ui.clone(),
        View::Home,
        "Home",
        None,
        0,
        inset,
    ));
    content = content.child(group(
        vm.clone(),
        ui.clone(),
        ui.all_open,
        View::All,
        "All items",
        vm.count(&View::All),
        inset,
    ));
    if ui.all_open.get() {
        content = content.child(nav(
            vm.clone(),
            ui.clone(),
            View::Starred,
            "Starred items",
            Some(Icon::Starred),
            vm.count(&View::Starred),
            inset + 20.0,
        ));
        content = content.child(nav(
            vm.clone(),
            ui.clone(),
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
        ui.clone(),
        ui.explore_open,
        View::Explore,
        "Explore",
        0,
        inset,
    ));
    if ui.explore_open.get() {
        content = content.child(nav(
            vm.clone(),
            ui.clone(),
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
            .height(if touch { 44.0 } else { 32.0 })
            .padding((inset, 8.0, 0.0, 0.0)),
    );
    let feeds = vm.feeds();
    let counts: HashMap<u64, usize> = feeds
        .iter()
        .map(|feed| (feed.id, vm.count(&View::Feed(feed.id))))
        .collect();
    let show_all = vm.state().settings.show_all_subscriptions;
    let keyboard_cursor = cursor::current(&cursor::visible(&vm, &ui), &ui);
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
                keyboard_cursor.as_ref()
                    == Some(&TreeNode::Feed {
                        id: feed.id,
                        folder: String::new(),
                    }),
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
        let folder_cursor = keyboard_cursor.as_ref() == Some(&TreeNode::Folder(folder.clone()));
        content = content.child(folder::row(
            vm.clone(),
            ui.clone(),
            &folder,
            count,
            inset,
            folder_cursor,
        ));
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
                        keyboard_cursor.as_ref()
                            == Some(&TreeNode::Feed {
                                id: feed.id,
                                folder: folder.clone(),
                            }),
                    ));
                }
            }
        }
    }
    let tags = vm.tags();
    if !tags.is_empty() {
        content = content.child(border().height(15.0));
        for tag in tags {
            if !show_all && vm.count(&View::Tag(tag.clone())) == 0 {
                continue;
            }
            content = content.child(nav(
                vm.clone(),
                ui.clone(),
                View::Tag(tag.clone()),
                &tag,
                Some(Icon::Tag),
                vm.count(&View::Tag(tag.clone())),
                inset,
            ));
        }
    }
    let close = vm.clone();
    let restore = ui.clone();
    rsx! {
        Grid {key:"navigation-pane",name:"NavigationPane",row_definitions:if touch{"56,*,48"}else if density==Density::Compact{"43,*,40"}else{"57,*,40"},background:"White",
            Button {class:"reader-subscribe",name:"SubscribeButton",node_ref:ui.subscribe_button,content:"SUBSCRIBE",height:if touch{44.0}else{29.0},min_height:if touch{44.0}else{29.0},min_width:98.0,horizontal_alignment:H::Left,vertical_alignment:V::Top,margin:(inset,if touch{6.0}else if density==Density::Compact{7.0}else{16.5},0.0,0.0),automation_properties_name:"Subscribe",on_click:move |_|subscribe.show(Panel::Subscribe)}
            if narrow {Button{class:"reader-flat",name:"CloseSubscriptions",node_ref:ui.drawer_close,width:44.0,height:44.0,margin:(0.0,6.0,4.0,0.0),horizontal_alignment:H::Right,vertical_alignment:V::Top,automation_properties_name:"Close subscriptions",on_loaded:move |event|{if let Some(control)=event.source().and_then(|source|source.cast::<ferroui_controls::Control>()){control.focus();}},on_click:move |_|{close.set_sidebar_open(false);restore.restore_article_focus();},{icon(Icon::Close,16.0,"#666666")}}}
            ScrollViewer {name:"SubscriptionScroll",node_ref:ui.navigation_scroll,Grid.row:1,horizontal_scroll_bar_visibility:ScrollBarVisibility::Disabled,vertical_scroll_bar_visibility:ScrollBarVisibility::Auto,
                {content}
            }
            Border {Grid.row:2,margin:(inset,0.0,0.0,0.0),vertical_alignment:V::Center,
                {reader_controls::link("Manage subscriptions »",move||manage.show(Panel::Subscriptions))}
            }
        }
    }
}
