pub(crate) mod cursor;
mod feed;
pub(crate) mod folder;
mod navigation;
mod ordering;
mod subscriptions;
pub(crate) mod tree;
use crate::session::{Panel, Session};
use feed::feed_row;
use ferroui_base::layout::{HorizontalAlignment as H, VerticalAlignment as V};
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
    content = content.child(subscriptions::header(vm.clone(), ui.clone(), inset));
    let feeds: HashMap<_, _> = vm.feeds().into_iter().map(|feed| (feed.id, feed)).collect();
    let ordering = ordering::Context::capture(&vm);
    let nodes = cursor::with_order(&vm, &ui, &ordering.order);
    let keyboard_cursor = cursor::current(&nodes, &ui);
    let mut tags_started = false;
    for node in nodes {
        let selected_cursor = keyboard_cursor.as_ref() == Some(&node);
        match node {
            TreeNode::Feed { id, folder } => {
                if let Some(feed) = feeds.get(&id) {
                    content = content.child(feed_row(
                        vm.clone(),
                        ui.clone(),
                        feed,
                        vm.count(&View::Feed(id)),
                        inset + if folder.is_empty() { 6.0 } else { 20.0 },
                        &folder,
                        selected_cursor,
                        ordering.clone(),
                    ));
                }
            }
            TreeNode::Folder(name) => {
                content = content.child(folder::row(
                    vm.clone(),
                    ui.clone(),
                    &name,
                    vm.count(&View::Folder(name.clone())),
                    inset,
                    selected_cursor,
                    ordering.clone(),
                ));
            }
            TreeNode::Tag(tag) => {
                if !tags_started {
                    content = content.child(border().height(15.0));
                    tags_started = true;
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
