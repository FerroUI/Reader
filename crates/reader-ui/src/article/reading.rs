use crate::rows::Row;
use ferroui_base::{
    layout::{HorizontalAlignment as H, Orientation, VerticalAlignment as V},
    media::TextWrapping,
};
use ferroui_view::prelude::*;
use reader_state::AppViewModel;
use std::rc::Rc;

/// Publisher copies are an explicit extension of the original inline reader.
/// Feed text remains available even while a download fails or is cancelled.
pub(super) fn view(vm: Rc<AppViewModel>, row: &Row) -> Element {
    let id = row.article.id;
    let height = if row.touch {
        crate::responsive::TOUCH_TARGET
    } else {
        24.0
    };
    let full = row.article.show_full_content && row.article.has_full_content();
    let cached = row.article.has_full_content();
    let load = vm.clone();
    let cancel = vm.clone();
    let feed = vm.clone();
    let reload = vm;
    rsx! {StackPanel{name:"ArticleReadingControls",max_width:650.0,horizontal_alignment:H::Left,spacing:4.0,
        WrapPanel{orientation:Orientation::Horizontal,
            if row.article_loading {
                TextBlock{text:"Loading full article…",font_size:12.0,foreground:"#666666",vertical_alignment:V::Center,margin:(0.0,0.0,12.0,0.0)}
                Button{class:"reader-link",content:"Cancel",height:height,padding:(0.0,0.0,12.0,0.0),automation_properties_name:"Cancel article download",on_click:move |_|cancel.cancel_full_article(id)}
            } else {
                if full {
                    TextBlock{text:"Full article · saved",font_size:12.0,foreground:"#666666",vertical_alignment:V::Center,margin:(0.0,0.0,14.0,0.0),tool_tip_tip:"This article is saved for offline reading. Images require a connection unless already cached."}
                    Button{class:"reader-link",content:"Feed content",height:height,padding:(0.0,0.0,14.0,0.0),automation_properties_name:"Show feed content",on_click:move |_|feed.show_feed_content(id)}
                    Button{class:"reader-link",content:"Reload",height:height,automation_properties_name:"Reload full article",on_click:move |_|{reload.load_full_article(id,true);}}
                } else {
                    Button{class:"reader-link",content:if row.article_error.is_some(){"Try again"}else if cached{"Full article"}else{"Read full article"},height:height,automation_properties_name:if row.article_error.is_some(){"Retry full article"}else{"Read full article"},on_click:move |_|{load.load_full_article(id,false);}}
                }
            }
        }
        if let Some(message) = &row.article_error {
            TextBlock{text:message.clone(),font_size:12.0,foreground:"#A33A2B",text_wrapping:TextWrapping::Wrap}
        }
    }}
}
