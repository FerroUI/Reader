//! Empty streams keep their identity: recovering from an unread filter never
//! navigates away from the subscription, folder, tag, or search being viewed.
use ferroui_base::layout::{HorizontalAlignment as H, VerticalAlignment as V};
use ferroui_base::media::{FontWeight, TextAlignment, TextWrapping};
use ferroui_controls::primitives::ScrollBarVisibility;
use ferroui_view::prelude::*;
use reader_domain::View;
use reader_state::AppViewModel;
use std::rc::Rc;

pub fn view(vm: Rc<AppViewModel>) -> Element {
    let view = vm.view();
    let unread = vm.state().settings.unread_only;
    let narrow = crate::responsive::narrow(&vm);
    let title = match &view {
        View::Search(query) => format!("Your search - {query} - did not match any items."),
        View::All if unread => "Your reading list has no unread items.".into(),
        View::All => "Your reading list is empty.".into(),
        View::Starred if unread => "You have no unread starred items.".into(),
        View::Starred => "You have no starred items.".into(),
        View::Folder(name) | View::Tag(name) if unread => {
            format!("You have no unread \"{name}\" items.")
        }
        View::Folder(name) | View::Tag(name) => format!("You have no \"{name}\" items."),
        _ if unread => format!("\"{}\" has no unread items.", vm.title()),
        _ => format!("\"{}\" has no items.", vm.title()),
    };
    let recovery = vm.clone();
    let search = matches!(view, View::Search(_));
    rsx! {
        ScrollViewer {name:"EmptyStream",horizontal_scroll_bar_visibility:ScrollBarVisibility::Disabled,vertical_scroll_bar_visibility:ScrollBarVisibility::Auto,
            Border {name:"EmptyStreamMessage",vertical_alignment:V::Top,margin:13.0,padding:if narrow{(16.0,20.0,16.0,32.0)}else{(26.0,20.0,26.0,39.0)},border_brush:"#CCCCCC",border_thickness:2.0,corner_radius:5.0,background:"White",
                StackPanel {spacing:16.0,horizontal_alignment:H::Stretch,
                    TextBlock {name:"EmptyStreamTitle",text:title,font_size:19.0,font_weight:FontWeight::Bold,foreground:"#000000",text_alignment:TextAlignment::Center,text_wrapping:TextWrapping::Wrap,margin:(0.0,16.0,0.0,0.0)}
                    if unread && !search {
                        Border {horizontal_alignment:H::Center,
                            {reader_controls::link("View all items",move||recovery.set_unread_only(false))}
                        }
                    }
                    if search {
                        TextBlock {text:"Try another search or change the search scope. You can use title:, author:, feed:, tag:, is:starred and is:unread.",font_size:13.0,foreground:"#666666",text_alignment:TextAlignment::Center,text_wrapping:TextWrapping::Wrap,max_width:550.0,horizontal_alignment:H::Center}
                    }
                }
            }
        }
    }
}
