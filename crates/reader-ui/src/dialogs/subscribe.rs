//! Reader's desktop quick-add bubble, anchored to the actual native button.
use super::{form, placement};
use crate::session::{Panel, Session};
use ferroui_base::input::{Key, KeyboardNavigationMode};
use ferroui_base::layout::{HorizontalAlignment as H, VerticalAlignment as V};
use ferroui_base::media::{FontStyle, TextWrapping};
use ferroui_base::{Point, Rect};
use ferroui_controls::primitives::ScrollBarVisibility;
use ferroui_view::prelude::*;
use reader_controls::{icon, Icon};
use reader_state::AppViewModel;
use std::rc::Rc;

pub(super) fn popup(vm: Rc<AppViewModel>, ui: Rc<Session>) -> Element {
    let form::Form { input, save, .. } = form::prepare(vm.clone(), ui.clone(), Panel::Subscribe);
    let touch = crate::responsive::touch(&vm);
    let width = 302.0_f64.min(vm.viewport() - 16.0);
    let origin = anchor(&ui)
        .map(|rect| rect.bottom_left())
        .unwrap_or_else(|| Point::new(8.0, if ui.focus_mode.get() { 44.0 } else { 104.0 }));
    let x = origin.x.clamp(8.0, (vm.viewport() - width - 8.0).max(8.0));
    let y = origin.y.clamp(8.0, (vm.viewport_height() - 120.0).max(8.0));
    let height = (vm.viewport_height() - y - 8.0).max(1.0);
    let escape = ui.clone();
    let outside = ui.clone();
    let close = ui.clone();
    let validation = form::error(&ui);
    let anchor_vm = vm.clone();
    let anchor_ui = Rc::downgrade(&ui);
    rsx! {Grid {key:"subscribe-popup",name:"DialogOverlay",keyboard_navigation_tab_navigation:KeyboardNavigationMode::Cycle,
        on_key_down:move |event|{if event.key==Key::Escape{event.set_handled(true);escape.close();}},
        Button {class:"reader-flat",background:"Transparent",horizontal_alignment:H::Stretch,vertical_alignment:V::Stretch,is_tab_stop:false,automation_properties_name:"Dismiss subscription popup",on_click:move |_|outside.close()}
        Border {name:"ReaderDialog",with:move |popup|{let weak=anchor_ui.clone();placement::follow(popup,&anchor_vm,move |_|weak.upgrade().and_then(|ui|anchor(&ui)));},width:width,max_height:height,background:"#F7F7F7",border_brush:"#D0D0D0",border_thickness:1.0,horizontal_alignment:H::Left,vertical_alignment:V::Top,margin:(x,y,0.0,0.0),
            Grid {name:"SubscribePopup",
                ScrollViewer {max_height:(height-2.0).max(1.0),horizontal_scroll_bar_visibility:ScrollBarVisibility::Disabled,vertical_scroll_bar_visibility:ScrollBarVisibility::Auto,
                    StackPanel {margin:8.0,spacing:0.0,
                        TextBlock {text:"Enter a feed or website URL.",font_size:11.0,line_height:13.0,min_height:if touch{36.0}else{13.0},foreground:"#666666",margin:(0.0,0.0,if touch{36.0}else{16.0},0.0),text_wrapping:TextWrapping::Wrap}
                        Grid {column_definitions:"*,Auto",column_spacing:8.0,
                            {input.height(if touch{44.0}else{22.0}).padding((4.0,1.0)).vertical_alignment(V::Center)}
                            Button {Grid.column:1,class:"reader-button",content:"Add",min_width:if touch{52.0}else{44.0},width:if touch{52.0}else{44.0},padding:(4.0,0.0),automation_properties_name:"Add",on_click:move |event|{event.set_handled(true);save();}}
                        }
                        TextBlock {text:"e.g., googleblog.blogspot.com",font_size:11.0,line_height:13.0,font_style:FontStyle::Italic,foreground:"#666666",text_wrapping:TextWrapping::Wrap}
                        {validation}
                    }
                }
                Button {class:"reader-flat",width:if touch{44.0}else{22.0},height:if touch{44.0}else{22.0},horizontal_alignment:H::Right,vertical_alignment:V::Top,automation_properties_name:"Close dialog",on_click:move |_|close.close(),{icon(Icon::Close,if touch{11.0}else{9.0},"#777777")}}
            }
        }
    }}
}

fn anchor(ui: &Session) -> Option<Rect> {
    let button = ui.subscribe_button.get_untracked().filter(|button| {
        button.is_attached_to_visual_tree()
            && button.is_effectively_visible()
            && button.bounds().width > 0.0
            && button.bounds().height > 0.0
    })?;
    let root = button.visual_root()?;
    let origin = button.translate_point(Point::new(0.0, 0.0), &root)?;
    let bounds = button.bounds();
    Some(Rect::new(origin.x, origin.y, bounds.width, bounds.height))
}
