use super::{cursor, navigation::row_class, ordering, tree::TreeNode};
use crate::{
    menus,
    session::{FolderTarget, Panel, Session},
};
use ferroui_base::layout::{HorizontalAlignment as H, VerticalAlignment as V};
use ferroui_base::media::{FontWeight, TextTrimming};
use ferroui_view::prelude::*;
use reader_controls::{icon, Icon};
use reader_domain::View;
use reader_state::AppViewModel;
use std::rc::Rc;

pub(super) fn row(
    vm: Rc<AppViewModel>,
    ui: Rc<Session>,
    name: &str,
    count: usize,
    inset: f64,
    keyboard_cursor: bool,
    ordering: Rc<ordering::Context>,
) -> Element {
    let touch = crate::responsive::touch(&vm);
    let collapsed = ui.collapsed.get().contains(name);
    let selected = vm.view() == View::Folder(name.to_owned());
    let drop_vm = vm.clone();
    let drop_folder = name.to_owned();
    let open_vm = vm.clone();
    let open_ui = ui.clone();
    let open_name = name.to_owned();
    let disclosure_ui = ui.clone();
    let disclosure_name = name.to_owned();
    let hold_vm = vm.clone();
    let held_name = name.to_owned();
    let drag_vm = vm.clone();
    let drag_ui = ui.clone();
    let drag_node = TreeNode::Folder(name.to_owned());
    let context = menus::context(&vm, items_with_order(vm.clone(), ui, name, ordering));
    rsx! {
        Grid {key:format!("folder:{name}"),name:"SubscriptionFolderRow",
            with:move |target|crate::drag::attach_target(&target.clone().upcast(),drop_vm.clone(),drop_folder.clone()),
            column_definitions:if touch{"44,*"}else{"20,*"},margin:((inset-18.0).max(0.0),0.0,8.0,0.0),height:if touch{44.0}else{23.0},
            Button {class:"reader-flat",automation_properties_name:format!("Toggle {name}"),on_click:move |_|disclosure_ui.toggle_folder(&disclosure_name),
                {icon(if collapsed{Icon::ChevronRight}else{Icon::ChevronDown},8.0,"#777777")}
            }
            Button {Grid.column:1,name:"SubscriptionFolder",class:row_class(selected,keyboard_cursor),
                horizontal_content_alignment:H::Stretch,automation_properties_name:name.to_owned(),context_menu:context,
                with:move |button|{
                    menus::attach_context_keys(button);
                    crate::drag::attach_source(button,drag_vm.clone(),drag_ui.clone(),drag_node.clone());
                    let vm=hold_vm.clone();
                    let name=held_name.clone();
                    let generation=hold_vm.clone();
                    crate::gestures::attach_held_menu(button,Rc::new(move||vm.folders().contains(&name)),Rc::new(move||generation.library_generation()));
                },
                on_click:move |_|{
                    let node=TreeNode::Folder(open_name.clone());
                    if !crate::drag::consume_click(&node) { cursor::open(&open_vm,&open_ui,node); }
                },
                Grid {column_definitions:"20,*,Auto",
                    {icon(Icon::Folder,14.0,"#A1A1A1")}
                    TextBlock {Grid.column:1,text:name.to_owned(),text_trimming:<dyn TextTrimming>::character_ellipsis(),vertical_alignment:V::Center,font_weight:if count>0{FontWeight::Bold}else{FontWeight::Normal}}
                    TextBlock {Grid.column:2,text:crate::format::unread_badge(count),font_size:11.0,foreground:"#666666",margin:(4.0,0.0,8.0,0.0),vertical_alignment:V::Center}
                }
            }
            Border {Grid.column_span:2,is_hit_test_visible:false,{crate::drag::marker(false)}}
            Border {Grid.column_span:2,is_hit_test_visible:false,{crate::drag::marker(true)}}
        }
    }
}

/// Sidebar and toolbar actions share the same captured target. A popup that
/// outlives a replacement library cannot operate on an unrelated folder.
pub(crate) fn items(vm: Rc<AppViewModel>, ui: Rc<Session>, name: &str) -> Vec<Element> {
    let ordering = ordering::Context::capture(&vm);
    items_with_order(vm, ui, name, ordering)
}

fn items_with_order(
    vm: Rc<AppViewModel>,
    ui: Rc<Session>,
    name: &str,
    ordering: Rc<ordering::Context>,
) -> Vec<Element> {
    let target = FolderTarget::capture(&vm, name);
    let refresh_vm = vm.clone();
    let refresh_target = target.clone();
    let rename_vm = vm.clone();
    let rename_ui = ui.clone();
    let rename_target = target.clone();
    let delete_vm = vm.clone();
    let delete_ui = ui.clone();
    let delete_target = target.clone();
    let unsubscribe_vm = vm.clone();
    let unsubscribe_ui = ui.clone();
    let unsubscribe_target = target.clone();
    let mut items = vec![
        menus::item_enabled(
            "Refresh folder",
            !vm.is_refreshing() && !target.feeds.is_empty(),
            move || {
                if refresh_target.valid(&refresh_vm) {
                    refresh_vm.refresh_view(View::Folder(refresh_target.name.clone()));
                }
            },
        ),
        menus::separator(),
        menus::item("Rename folder...", move || {
            if rename_target.valid(&rename_vm) {
                rename_ui.show(Panel::RenameFolder(rename_target.name.clone()));
            }
        }),
        menus::item("Delete folder...", move || {
            if delete_target.valid(&delete_vm) {
                delete_ui.show(Panel::DeleteFolder(delete_target.clone()));
            }
        }),
        menus::item_enabled(
            "Unsubscribe from all...",
            !target.feeds.is_empty(),
            move || {
                if unsubscribe_target.valid(&unsubscribe_vm) {
                    unsubscribe_ui.show(Panel::UnsubscribeFolder(unsubscribe_target.clone()));
                }
            },
        ),
    ];
    items.push(menus::separator());
    items.extend(ordering::move_items(
        vm.clone(),
        ui,
        TreeNode::Folder(name.to_owned()),
        ordering.clone(),
    ));
    items.push(menus::separator());
    items.push(ordering::sort_item(vm, name, ordering));
    items
}
