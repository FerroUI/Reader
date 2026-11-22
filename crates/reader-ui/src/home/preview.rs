//! Reader's compact overview excerpt keeps its title, date and snippet together.
use ferroui_base::layout::HorizontalAlignment as H;
use ferroui_base::media::{FontWeight, TextWrapping};
use ferroui_view::prelude::*;
use reader_domain::{Article, View};
use reader_state::AppViewModel;
use std::rc::Rc;

pub(super) fn view(vm: Rc<AppViewModel>, article: &Article) -> Element {
    let id = article.id;
    let feed = article.feed_id;
    let touch = crate::responsive::touch(&vm);
    let date = chrono::DateTime::from_timestamp(article.published, 0)
        .map(|date| date.format("%-m/%-d/%Y").to_string())
        .unwrap_or_default();
    // The preserved overview uses a line break after the title and 104% text.
    rsx! {Button{
        key:format!("overview-entry:{id}"),
        name:format!("OverviewArticle-{id}"),
        class:"reader-link",
        automation_properties_name:article.title.clone(),
        horizontal_content_alignment:H::Stretch,
        min_height:if touch{44.0}else{0.0},
        on_click:move |event|{
            event.set_handled(true);
            vm.navigate(View::Feed(feed));
            vm.select(Some(id));
        },
        // Keep the whole preview hit-testable, including blank space beside
        // short dates and summaries, while the Button changes pointer states.
        StackPanel{spacing:0.0,background:"Transparent",
            TextBlock{text:article.title.clone(),font_size:13.5,line_height:16.0,font_weight:FontWeight::Bold,foreground:"#444444",text_wrapping:TextWrapping::Wrap}
            TextBlock{text:date,font_size:13.5,line_height:16.0,foreground:"#777777",tool_tip_tip:crate::article::full_date(article.published),margin:(0.0,0.0,0.0,2.0)}
            TextBlock{text:excerpt(&article.summary),font_size:13.5,foreground:"#444444",text_wrapping:TextWrapping::Wrap,line_height:16.0}
        }
    }}
}

fn excerpt(summary: &str) -> String {
    let text = summary.split_whitespace().collect::<Vec<_>>().join(" ");
    let Some((end, _)) = text.char_indices().nth(160) else {
        return text;
    };
    let prefix = &text[..end];
    let end = prefix.rfind(' ').filter(|&at| at > end / 2).unwrap_or(end);
    format!("{}…", prefix[..end].trim_end())
}
