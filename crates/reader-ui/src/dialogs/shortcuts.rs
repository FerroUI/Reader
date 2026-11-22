use crate::session::Session;
use ferroui_base::input::{Key, KeyboardNavigationMode};
use ferroui_base::layout::{HorizontalAlignment as H, VerticalAlignment as V};
use ferroui_base::media::{FontWeight, TextAlignment, TextTrimming, TextWrapping};
use ferroui_controls::primitives::ScrollBarVisibility;
use ferroui_view::prelude::*;
use std::rc::Rc;

fn view(wide: bool) -> Element {
    let sections = [
        (
            "Navigation",
            vec![
                ("j / k", "Next / previous item"),
                ("Space", "Next item or page"),
                ("Shift + Space", "Previous item or page"),
                ("n / p", "Item scan down / up (list only)"),
                ("Shift + n / p", "Next / previous subscription"),
                ("Shift + x", "Expand or collapse folder"),
                ("Shift + o", "Open subscription or folder"),
            ],
        ),
        (
            "Application",
            vec![
                ("r", "Refresh"),
                ("f", "Toggle full-screen reading"),
                ("u", "Show / hide subscriptions"),
                ("1 / 2", "Expanded view / list view"),
                ("/", "Focus search"),
                ("a", "Add a subscription"),
                ("= / −", "Increase / decrease text size"),
            ],
        ),
        (
            "Jumping",
            vec![
                ("g then h", "Go home"),
                ("g then a", "Go to all items"),
                ("g then s", "Go to starred items"),
                ("g then u", "Choose a subscription"),
                ("g then t", "Choose a tag or folder"),
                ("g then Shift + t", "Go to Trends"),
                ("g then d / e", "Browse subscriptions"),
            ],
        ),
        (
            "Acting on items",
            vec![
                ("s", "Star item"),
                ("t", "Tag item"),
                ("e", "Email item"),
                ("Shift + s", "Share item"),
                ("v", "View original"),
                ("o / Enter", "Expand / collapse item (list only)"),
                ("m", "Mark item read / unread"),
                ("Shift + a", "Mark all as read"),
                ("Shift + t", "Send item"),
            ],
        ),
    ];
    let mut content = grid()
        .column_definitions(if wide { "*,*" } else { "*" })
        .row_definitions(if wide {
            "Auto,Auto"
        } else {
            "Auto,Auto,Auto,Auto"
        })
        .column_spacing(30.0)
        .row_spacing(16.0);
    for (index, (title, shortcuts)) in sections.into_iter().enumerate() {
        let mut section = stack_panel().spacing(4.0).child(
            text_block()
                .text(title)
                .font_weight(FontWeight::Bold)
                .font_size(14.0)
                .foreground("#DDDD00")
                .text_alignment(TextAlignment::Center)
                .margin((0.0, 0.0, 0.0, 4.0)),
        );
        for (key, description) in shortcuts {
            section = section.child(rsx! {Grid{column_definitions:if wide{"135,*"}else{"98,*"},column_spacing:8.0,min_height:20.0,
                TextBlock{text:format!("{key}:"),font_weight:FontWeight::Bold,font_size:12.0,foreground:"White",text_alignment:TextAlignment::Right,text_wrapping:TextWrapping::Wrap}
                TextBlock{Grid.column:1,text:description,font_size:13.0,foreground:"White",text_wrapping:TextWrapping::Wrap}
            }});
        }
        content = content.child(
            section
                .grid_column(if wide { (index % 2) as i32 } else { 0 })
                .grid_row(if wide {
                    (index / 2) as i32
                } else {
                    index as i32
                }),
        );
    }
    content.into()
}

pub fn overlay(ui: Rc<Session>, vm: &reader_state::AppViewModel) -> Element {
    let close = ui.clone();
    let footer_close = ui.clone();
    let escape = ui.clone();
    let narrow = crate::responsive::narrow(vm);
    let touch = crate::responsive::touch(vm);
    let metrics = crate::responsive::DialogMetrics::new(
        vm,
        if narrow {
            vm.viewport() - 16.0
        } else {
            vm.viewport() * 0.9
        },
        700.0,
    );
    let padding = if narrow { 16.0 } else { 18.0 };
    let body_height =
        (metrics.height - 2.0 * padding - if touch { 44.0 } else { 29.0 } - 16.0 - 2.0).max(1.0);
    rsx! {Grid{name:"KeyboardHelp",keyboard_navigation_tab_navigation:KeyboardNavigationMode::Cycle,on_key_down:move |event|{if event.key==Key::Escape{event.set_handled(true);escape.close();}},
        Border{focusable:true,is_tab_stop:false,on_loaded:move |event|{if let Some(control)=event.source().and_then(|source|source.cast::<ferroui_controls::Control>()){control.focus();}},width:metrics.width,max_height:metrics.height,background:"#CC000000",corner_radius:10.0,padding:padding,horizontal_alignment:H::Center,vertical_alignment:if narrow||vm.viewport_height()<760.0{V::Center}else{V::Top},margin:if narrow||vm.viewport_height()<760.0{(0.0,0.0,0.0,0.0)}else{(0.0,vm.viewport_height()*0.05,0.0,20.0)},
            Grid{row_definitions:"Auto,*",row_spacing:16.0,
                Grid{column_definitions:"*,Auto",column_spacing:8.0,
                    TextBlock{text:"Keyboard shortcuts",font_size:if narrow{18.0}else{20.0},foreground:"White",vertical_alignment:V::Center,text_trimming:<dyn TextTrimming>::character_ellipsis()}
                    Button{Grid.column:1,class:"reader-flat",width:if touch{44.0}else{29.0},height:if touch{44.0}else{29.0},automation_properties_name:"Close keyboard shortcuts",on_click:move |_|close.close(),{reader_controls::icon(reader_controls::Icon::Close,18.0,"White")}}
                }
                ScrollViewer{Grid.row:1,max_height:body_height,horizontal_scroll_bar_visibility:ScrollBarVisibility::Disabled,vertical_scroll_bar_visibility:ScrollBarVisibility::Auto,
                    StackPanel{spacing:16.0,
                        {view(vm.viewport()>=900.0)}
                        TextBlock{text:"Esc closes this help. Ctrl+Z / Cmd+Z undoes the last library change.",font_size:12.0,foreground:"#DDDDDD",text_alignment:TextAlignment::Center,text_wrapping:TextWrapping::Wrap}
                        Button{class:"reader-link",horizontal_alignment:H::Center,automation_properties_name:"Close shortcuts help",on_click:move |_|footer_close.close(),TextBlock{text:"Close",font_size:13.0,foreground:"#DDDD00"}}
                    }
                }
            }
        }
    }}
}
