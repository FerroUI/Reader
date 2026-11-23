use super::{cursor, navigation::row_class, tree::TreeNode};
use crate::{
    menus,
    session::{Panel, Session},
};
use ferroui_base::layout::{HorizontalAlignment as H, VerticalAlignment as V};
use ferroui_base::media::{FontWeight, TextTrimming};
use ferroui_view::prelude::*;
use reader_controls::{icon, Icon};
use reader_domain::{Feed, View};
use reader_state::AppViewModel;
use std::rc::Rc;
pub(super) fn feed_row(
    vm: Rc<AppViewModel>,
    ui: Rc<Session>,
    feed: &Feed,
    count: usize,
    inset: f64,
    group: &str,
    keyboard_cursor: bool,
) -> Element {
    let id = feed.id;
    let drag_vm = vm.clone();
    let target_vm = vm.clone();
    let target_folder = group.to_string();
    let selected = vm.view() == View::Feed(id);
    let go = vm.clone();
    let go_ui = ui.clone();
    let go_node = TreeNode::Feed {
        id,
        folder: group.to_string(),
    };
    let rename = ui.clone();
    let folders = ui.clone();
    let remove = ui.clone();
    let refresh = vm.clone();
    let refresh_url = feed.url.clone();
    let generation = vm.library_generation();
    let menu = menus::context(
        &vm,
        vec![
            menus::item_enabled("Refresh subscription", !vm.is_refreshing(), move || {
                if refresh.library_generation() == generation
                    && refresh.feed(id).is_some_and(|feed| feed.url == refresh_url)
                {
                    refresh.refresh_view(View::Feed(id));
                }
            }),
            menus::separator(),
            menus::item("Rename subscription...", move || {
                rename.show(Panel::Rename(id))
            }),
            menus::item("Change folders...", move || folders.show(Panel::Move(id))),
            menus::item("Unsubscribe...", move || {
                remove.show(Panel::Unsubscribe(id))
            }),
        ],
    );
    border().key(format!("feed:{group}:{id}")).border_thickness((3.0,0.0,0.0,0.0)).border_brush(if selected{"#DD4B39"}else{"Transparent"}).child(
        grid().name("SubscriptionRow").with(move|control|crate::drag::attach_row_target(&control.clone().upcast(),target_vm.clone(),id,target_folder.clone())).child(
        button().name("SubscriptionLink").with(move |button|crate::drag::attach_source(button,drag_vm.clone(),id)).class(row_class(selected,keyboard_cursor)).height(if crate::responsive::touch(&vm){44.0}else{23.0}).padding((inset-3.0,0.0,12.0,0.0)).horizontal_content_alignment(H::Stretch)
        .automation_properties_name(format!("{} ({} unread)",feed.title,crate::format::unread(count))).tool_tip_tip(feed.error.clone().unwrap_or_else(||feed.url.clone())).context_menu(menu).with(menus::attach_context_keys)
        .on_click(move |_|{if crate::drag::consume_click(id){return;}cursor::open(&go,&go_ui,go_node.clone());}).child(rsx!{
            Grid {column_definitions:"20,*,Auto",
                {icon(Icon::Feed,12.0,if feed.error.is_some(){"#DD4B39"}else{"#E89437"})}
                TextBlock {Grid.column:1,text:feed.title.clone(),font_size:13.0,vertical_alignment:V::Center,text_trimming:<dyn TextTrimming>::character_ellipsis(),font_weight:if count>0{FontWeight::Bold}else{FontWeight::Normal}}
                TextBlock {Grid.column:2,text:crate::format::unread_badge(count),font_size:11.0,foreground:"#666666",margin:(6.0,0.0,0.0,0.0),vertical_alignment:V::Center}
            }
        })
        ).child(crate::drag::marker(false)).child(crate::drag::marker(true))
    ).into()
}
