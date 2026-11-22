//! The original desktop tag editor sits beside the article action that opened it.
use super::{form, placement};
use crate::session::{Panel, Session};
use ferroui_base::input::{Key, KeyboardNavigationMode};
use ferroui_base::layout::{HorizontalAlignment as H, VerticalAlignment as V};
use ferroui_base::media::TextWrapping;
use ferroui_base::{Point, Rect};
use ferroui_controls::{primitives::ScrollBarVisibility, Border, Button, ScrollViewer};
use ferroui_view::prelude::*;
use reader_domain::Id;
use reader_state::AppViewModel;
use std::rc::Rc;

pub(super) fn anchor(ui: &Session, vm: &AppViewModel, id: Id) -> Option<Rect> {
    let list = ui.article_list.get_untracked()?;
    let button = list.find_descendant_of_type_where::<Button>(false, |button| {
        button.is_attached_to_visual_tree()
            && button.is_effectively_visible()
            && button.bounds().width > 0.0
            && button.bounds().height > 0.0
            && button.name().as_deref() == Some("ArticleTags")
            && button
                .tag()
                .as_ref()
                .and_then(|tag| tag.downcast_ref::<Id>())
                .copied()
                == Some(id)
    })?;
    let local = button.translate_point(Point::new(0.0, 0.0), &list)?;
    let bounds = button.bounds();
    if local.x + bounds.width <= 0.0
        || local.x >= list.bounds().width
        || local.y + bounds.height <= 0.0
        || local.y >= list.bounds().height
    {
        return None;
    }
    let root = button.visual_root()?;
    let point = button.translate_point(Point::new(0.0, 0.0), &root)?;
    (point.x >= 0.0 && point.x < vm.viewport() && point.y >= 0.0 && point.y <= vm.viewport_height())
        .then_some(Rect::new(point.x, point.y, bounds.width, bounds.height))
}

/// A resize may temporarily clip the source row before native layout completes.
/// Keep the existing edit presentation instead of oscillating through a modal.
pub(super) fn existing_anchor(ui: &Session, id: Id) -> Option<Rect> {
    if ui.form_for.borrow().as_ref() != Some(&Panel::Tags(id)) {
        return None;
    }
    let field = ui.field.get_untracked()?;
    let body = field.find_ancestor_of_type_where::<ScrollViewer>(false, |control| {
        control.name().as_deref() == Some("ArticleTagsPopup")
    })?;
    let popup = body.find_ancestor_of_type_where::<Border>(false, |control| {
        control.name().as_deref() == Some("ReaderDialog")
    })?;
    let root = popup.visual_root()?;
    let origin = popup.translate_point(Point::new(0.0, 0.0), &root)?;
    Some(Rect::new(origin.x, origin.y, 0.0, 0.0))
}

pub(super) fn popup(vm: Rc<AppViewModel>, ui: Rc<Session>, id: Id, position: Rect) -> Element {
    let form::Form { input, save, .. } = form::prepare(vm.clone(), ui.clone(), Panel::Tags(id));
    let width = 278.0_f64.min(vm.viewport() - 16.0);
    let x = position
        .x
        .clamp(8.0, (vm.viewport() - width - 8.0).max(8.0));
    let y = if vm.viewport_height() - position.bottom() >= 130.0 {
        position.bottom()
    } else {
        (position.y - 100.0).max(8.0)
    };
    let height = (vm.viewport_height() - y - 8.0).max(1.0);
    let escape = ui.clone();
    let outside = ui.clone();
    let cancel = ui.clone();
    let validation = form::error(&ui);
    let anchor_vm = vm.clone();
    let anchor_ui = Rc::downgrade(&ui);
    rsx! {Grid {key:format!("tags-popup:{id}"),name:"DialogOverlay",keyboard_navigation_tab_navigation:KeyboardNavigationMode::Cycle,
        on_key_down:move |event|{if event.key==Key::Escape{event.set_handled(true);escape.close();}},
        Button {class:"reader-flat",background:"Transparent",horizontal_alignment:H::Stretch,vertical_alignment:V::Stretch,is_tab_stop:false,automation_properties_name:"Dismiss tag editor",on_click:move |_|outside.close()}
        Border {name:"ReaderDialog",with:move |popup|{let weak=anchor_ui.clone();placement::follow(popup,&anchor_vm,move |vm|weak.upgrade().and_then(|ui|anchor(&ui,vm,id)));},width:width,max_height:height,background:"#F7F7F7",border_brush:"#D0D0D0",border_thickness:1.0,horizontal_alignment:H::Left,vertical_alignment:V::Top,margin:(x,y,0.0,0.0),padding:8.0,
            ScrollViewer {name:"ArticleTagsPopup",max_height:(height-18.0).max(1.0),horizontal_scroll_bar_visibility:ScrollBarVisibility::Disabled,vertical_scroll_bar_visibility:ScrollBarVisibility::Auto,
                StackPanel {spacing:4.0,
                    {input}
                    TextBlock {text:"Separate tags by commas",font_size:12.0,foreground:"#666666",text_wrapping:TextWrapping::Wrap}
                    {validation}
                    WrapPanel {
                        Border {margin:(0.0,0.0,8.0,0.0),{reader_controls::action("Save",move||save())}}
                        {reader_controls::action("Cancel",move||cancel.close())}
                    }
                }
            }
        }
    }}
}
