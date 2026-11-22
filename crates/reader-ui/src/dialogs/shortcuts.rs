use crate::session::Session;
use ferroui_base::input::{Key, KeyboardNavigationMode};
use ferroui_base::layout::{HorizontalAlignment as H, VerticalAlignment as V};
use ferroui_base::media::{FontWeight, TextTrimming, TextWrapping};
use ferroui_controls::primitives::ScrollBarVisibility;
use ferroui_view::prelude::*;
use std::rc::Rc;
fn view(wide: bool, narrow: bool) -> Element {
    let sections = [
        (
            "Navigation",
            vec![
                ("j / k", "Next / previous item"),
                ("n / p", "Select next / previous item without opening"),
                ("o / Enter", "Expand or collapse selected item"),
                ("Space / Shift+Space", "Page down / page up"),
                ("Shift+n / Shift+p", "Next / previous subscription"),
                ("g then h", "Go to Home"),
                ("g then a", "Go to All items"),
                ("g then s", "Go to Starred items"),
                ("g then t", "Choose a tag or folder"),
                ("g then u", "Choose a subscription"),
                ("g then Shift+t", "Go to Trends"),
                ("g then d / e", "Discover subscriptions"),
            ],
        ),
        (
            "Reading",
            vec![
                ("s", "Add or remove a star"),
                ("Shift+s / Shift+t", "Share / send selected item"),
                ("m", "Mark selected item read / unread"),
                ("v", "View original item"),
                ("e", "Email selected item"),
                ("t", "Edit tags"),
                ("Shift+a", "Mark all items as read"),
                ("r", "Refresh subscriptions"),
                ("1 / 2", "Expanded view / list view"),
                ("+ / −", "Increase / decrease article text size"),
            ],
        ),
        (
            "Reader",
            vec![
                ("/", "Search Reader"),
                ("a", "Add a subscription"),
                ("u", "Show or hide subscriptions"),
                ("f", "Reading focus mode"),
                ("?", "Show keyboard shortcuts"),
                ("Escape", "Close dialog / selected item"),
                ("Ctrl+z / Cmd+z", "Undo the last library change"),
            ],
        ),
    ];
    let mut content = grid()
        .column_definitions(if wide { "*,*,*" } else { "*" })
        .row_definitions(if wide { "*" } else { "Auto,Auto,Auto" })
        .column_spacing(25.0)
        .row_spacing(20.0);
    for (index, (title, shortcuts)) in sections.into_iter().enumerate() {
        let mut section = stack_panel().spacing(5.0).child(
            text_block()
                .text(title)
                .font_weight(FontWeight::Bold)
                .font_size(14.0)
                .foreground("#DDDD00"),
        );
        for (key, description) in shortcuts {
            section = section.child(rsx! {Grid{column_definitions:if narrow{"98,*"}else if wide{"115,*"}else{"140,*"},column_spacing:8.0,min_height:23.0,
                TextBlock{text:key,font_weight:FontWeight::Bold,font_size:12.0,foreground:"White",text_wrapping:TextWrapping::Wrap}
                TextBlock{Grid.column:1,text:description,font_size:13.0,foreground:"White",text_wrapping:TextWrapping::Wrap}
            }});
        }
        content = content.child(
            section
                .grid_column(if wide { index as i32 } else { 0 })
                .grid_row(if wide { 0 } else { index as i32 }),
        );
    }
    content.into()
}

pub fn overlay(ui: Rc<Session>, vm: &reader_state::AppViewModel) -> Element {
    let close = ui.clone();
    let escape = ui.clone();
    let narrow = crate::responsive::narrow(vm);
    let metrics = crate::responsive::DialogMetrics::new(
        vm,
        if narrow {
            vm.viewport() - 16.0
        } else {
            vm.viewport() * 0.9
        },
        700.0,
    );
    rsx! {Grid{name:"KeyboardHelp",keyboard_navigation_tab_navigation:KeyboardNavigationMode::Cycle,on_key_down:move |event|{if event.key==Key::Escape{event.set_handled(true);escape.close();}},
        Border{focusable:true,is_tab_stop:false,on_loaded:move |event|{if let Some(control)=event.source().and_then(|source|source.cast::<ferroui_controls::Control>()){control.focus();}},width:metrics.width,max_height:metrics.height,background:"#DD000000",corner_radius:10.0,padding:metrics.padding,horizontal_alignment:H::Center,vertical_alignment:if narrow||vm.viewport_height()<760.0{V::Center}else{V::Top},margin:if narrow||vm.viewport_height()<760.0{(0.0,0.0,0.0,0.0)}else{(0.0,40.0,0.0,20.0)},
            Grid{row_definitions:"Auto,*",row_spacing:metrics.gap,
                Grid{column_definitions:"*,Auto",column_spacing:8.0,
                    TextBlock{text:"Keyboard shortcuts",font_size:if narrow{18.0}else{20.0},foreground:"White",vertical_alignment:V::Center,text_trimming:<dyn TextTrimming>::character_ellipsis()}
                    Button{Grid.column:1,class:"reader-flat",width:if crate::responsive::touch(vm){44.0}else{29.0},height:if crate::responsive::touch(vm){44.0}else{29.0},automation_properties_name:"Close keyboard shortcuts",on_click:move |_|close.close(),{reader_controls::icon(reader_controls::Icon::Close,18.0,"White")}}
                }
                ScrollViewer{Grid.row:1,max_height:metrics.body_height.min(580.0),horizontal_scroll_bar_visibility:ScrollBarVisibility::Disabled,vertical_scroll_bar_visibility:ScrollBarVisibility::Auto,{view(vm.viewport()>1100.0,narrow)}}
            }
        }
    }}
}
