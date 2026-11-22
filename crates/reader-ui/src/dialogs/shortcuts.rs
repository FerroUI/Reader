use crate::session::Session;
use ferroui_base::input::KeyboardNavigationMode;
use ferroui_base::layout::{HorizontalAlignment as H, VerticalAlignment as V};
use ferroui_base::media::{FontWeight, TextWrapping};
use ferroui_controls::primitives::ScrollBarVisibility;
use ferroui_view::prelude::*;
use std::rc::Rc;
fn view(wide: bool) -> Element {
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
            section = section.child(rsx! {Grid{column_definitions:if wide{"115,*"}else{"140,*"},min_height:23.0,
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

pub fn overlay(ui: Rc<Session>, viewport: f64) -> Element {
    let close = ui.clone();
    rsx! {Grid{name:"KeyboardHelp",keyboard_navigation_tab_navigation:KeyboardNavigationMode::Cycle,
        Border{focusable:true,is_tab_stop:false,on_loaded:move |event|{if let Some(control)=event.source().and_then(|source|source.cast::<ferroui_controls::Control>()){control.focus();}},width:(viewport*0.9).max(280.0),max_height:700.0,background:"#DD000000",corner_radius:10.0,padding:25.0,horizontal_alignment:H::Center,vertical_alignment:V::Top,margin:(0.0,40.0,0.0,20.0),
            Grid{row_definitions:"Auto,*",row_spacing:20.0,
                Grid{column_definitions:"*,Auto",
                    TextBlock{text:"Keyboard shortcuts",font_size:20.0,foreground:"White"}
                    Button{Grid.column:1,class:"reader-flat",automation_properties_name:"Close keyboard shortcuts",on_click:move |_|close.close(),{reader_controls::icon(reader_controls::Icon::Close,18.0,"White")}}
                }
                ScrollViewer{Grid.row:1,max_height:580.0,horizontal_scroll_bar_visibility:ScrollBarVisibility::Disabled,vertical_scroll_bar_visibility:ScrollBarVisibility::Auto,{view(viewport>1100.0)}}
            }
        }
    }}
}
