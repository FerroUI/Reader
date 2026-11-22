use ferroui_base::media::FontWeight;
use ferroui_view::prelude::*;
pub fn view() -> Element {
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
                ("g then t", "Go to Trends"),
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
    let mut content = stack_panel().spacing(16.0);
    for (title, shortcuts) in sections {
        let mut section = stack_panel().spacing(5.0).child(
            text_block()
                .text(title)
                .font_weight(FontWeight::Bold)
                .font_size(14.0)
                .foreground("#DD4B39"),
        );
        for (key, description) in shortcuts {
            section = section.child(rsx! {Grid{column_definitions:"180,*",min_height:23.0,
                TextBlock{text:key,font_weight:FontWeight::Bold,font_size:12.0}
                TextBlock{Grid.column:1,text:description,font_size:13.0}
            }});
        }
        content = content.child(section);
    }
    content.into()
}
