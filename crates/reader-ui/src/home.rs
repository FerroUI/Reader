use crate::session::{Panel, Session};
use ferroui_base::layout::{HorizontalAlignment as H, Orientation, VerticalAlignment as V};
use ferroui_base::media::{FontWeight, TextWrapping};
use ferroui_controls::primitives::ScrollBarVisibility;
use ferroui_view::prelude::*;
use reader_controls::{link, paragraph, separator};
use reader_domain::View;
use reader_state::AppViewModel;
use std::rc::Rc;

pub fn view(vm: Rc<AppViewModel>, ui: Rc<Session>) -> Element {
    match vm.view() {
        View::Trends => trends(vm),
        View::Explore => explore(vm, ui),
        _ => welcome(vm, ui),
    }
}
fn welcome(vm: Rc<AppViewModel>, ui: Rc<Session>) -> Element {
    let all = vm.clone();
    let add = ui.clone();
    let shortcuts = ui.clone();
    let imports = ui.clone();
    let unread = vm.count(&View::All);
    let mut content=stack_panel().spacing(18.0).max_width(900.0).horizontal_alignment(H::Left)
        .child(text_block().text("A look at what's new").font_size(22.0).foreground("#333333"))
        .child(rsx! {StackPanel{orientation:Orientation::Horizontal,spacing:5.0,
            TextBlock{text:format!("You have {unread} unread items."),font_size:13.0,vertical_alignment:V::Center}
            {link("View all items »",move||all.navigate(View::All))}
        }});
    if vm.feeds().is_empty() {
        content=content.child(paragraph("Keep up with your favorite websites. Reader brings their latest updates together in one place."))
            .child(reader_controls::primary("Add a subscription",move||add.show(Panel::Subscribe)))
            .child(link("Import your subscriptions",move||imports.show(Panel::ImportExport)));
    } else {
        for feed in vm.feeds().into_iter().take(4) {
            let navigate = vm.clone();
            let id = feed.id;
            let articles: Vec<_> = vm
                .state()
                .articles
                .iter()
                .filter(|a| a.feed_id == id && !a.read)
                .take(2)
                .cloned()
                .collect();
            if articles.is_empty() {
                continue;
            }
            let mut group = stack_panel()
                .spacing(9.0)
                .margin((0.0, 5.0, 0.0, 20.0))
                .child(
                    button()
                        .class("reader-link")
                        .font_size(20.0)
                        .font_weight(FontWeight::Bold)
                        .content(feed.title.clone())
                        .on_click(move |_| navigate.navigate(View::Feed(id))),
                );
            for article in articles {
                let model = vm.clone();
                let id = article.id;
                group=group.child(rsx! {StackPanel{spacing:4.0,
                    Button{class:"reader-link",horizontal_content_alignment:H::Stretch,on_click:move |_|{model.navigate(View::All);model.select(Some(id));},
                        TextBlock{text:article.title,font_size:13.5,font_weight:FontWeight::Bold,foreground:"#444444",text_wrapping:TextWrapping::Wrap}
                    }
                    TextBlock{text:article.summary,font_size:13.0,foreground:"#555555",text_wrapping:TextWrapping::Wrap,line_height:19.0}
                }});
            }
            content = content.child(group);
        }
    }
    content = content
        .child(separator())
        .child(link("Read faster with keyboard shortcuts", move || {
            shortcuts.show(Panel::Shortcuts)
        }));
    if vm.feeds().iter().any(|feed| feed.sample) {
        content=content.child(text_block().text("Your starter library contains example articles. Add your own subscriptions to start reading live feeds.").font_size(11.0).foreground("#777777").text_wrapping(TextWrapping::Wrap));
    }
    scroll(content)
}
fn trends(vm: Rc<AppViewModel>) -> Element {
    let data = vm.trends();
    let mut content=stack_panel().spacing(20.0).max_width(900.0).horizontal_alignment(H::Stretch)
        .child(paragraph(format!("From your {} subscriptions, you have read {} items and starred {} items. In the last 30 days, you read {} items.",data.subscriptions,data.read,data.starred,data.read_last_30_days)))
        .child(text_block().text("Reading trends").font_size(17.0).font_weight(FontWeight::Bold));
    let max = data.days.iter().map(|(_, n)| *n).max().unwrap_or(1).max(1) as f64;
    let mut bars = grid()
        .column_definitions(&vec!["*"; 30].join(","))
        .height(160.0)
        .vertical_alignment(V::Bottom);
    for (i, (_, n)) in data.days.iter().enumerate() {
        bars = bars.child(
            border()
                .grid_column(i as i32)
                .background("#6699CC")
                .margin((2.0, 0.0))
                .height((140.0 * (*n as f64) / max).max(1.0))
                .vertical_alignment(V::Bottom)
                .tool_tip_tip(format!("{n} items read")),
        );
    }
    content = content
        .child(
            border()
                .border_brush("#CCCCCC")
                .border_thickness((0.0, 0.0, 0.0, 1.0))
                .child(bars),
        )
        .child(
            text_block()
                .text("Last 30 days")
                .font_size(11.0)
                .foreground("#777777")
                .horizontal_alignment(H::Right),
        )
        .child(separator())
        .child(
            text_block()
                .text("Subscription trends")
                .font_size(17.0)
                .font_weight(FontWeight::Bold),
        )
        .child(rsx! {Grid{column_definitions:"*,80,80,100",
            TextBlock{text:"Subscription",font_weight:FontWeight::Bold}
            TextBlock{Grid.column:1,text:"Items",font_weight:FontWeight::Bold}
            TextBlock{Grid.column:2,text:"Read",font_weight:FontWeight::Bold}
            TextBlock{Grid.column:3,text:"Read %",font_weight:FontWeight::Bold}
        }});
    for feed in data.feeds {
        let model = vm.clone();
        let id = feed.feed_id;
        content = content.child(rsx! {Grid{column_definitions:"*,80,80,100",min_height:25.0,
            {link(&feed.title,move||model.navigate(View::Feed(id)))}
            TextBlock{Grid.column:1,text:feed.total.to_string()}
            TextBlock{Grid.column:2,text:feed.read.to_string()}
            TextBlock{Grid.column:3,text:format!("{:.0}%",feed.percent)}
        }});
    }
    scroll(content)
}
fn explore(vm: Rc<AppViewModel>, ui: Rc<Session>) -> Element {
    let subscribe = ui.clone();
    let imports = ui.clone();
    let mut content=stack_panel().spacing(18.0).max_width(800.0).horizontal_alignment(H::Left)
        .child(text_block().text("Discover new subscriptions").font_size(22.0))
        .child(paragraph("Browse a few starting points, or add any RSS or Atom feed you follow."))
        .child(rsx!{StackPanel{orientation:Orientation::Horizontal,spacing:10.0,
            {reader_controls::action("Add a subscription",move||subscribe.show(Panel::Subscribe))}
            {reader_controls::action("Import subscriptions",move||imports.show(Panel::ImportExport))}
        }});
    for (title, description, url) in [
        (
            "Rust Blog",
            "Updates from the Rust programming language team.",
            "https://blog.rust-lang.org/feed.xml",
        ),
        (
            "NASA",
            "Discoveries, missions and stories from space.",
            "https://www.nasa.gov/feed/",
        ),
        (
            "BBC World News",
            "The latest world news.",
            "https://feeds.bbci.co.uk/news/world/rss.xml",
        ),
        (
            "GitHub Blog",
            "Product news and stories for developers.",
            "https://github.blog/feed/",
        ),
    ] {
        let model = vm.clone();
        content=content.child(separator()).child(rsx!{Grid{column_definitions:"*,Auto",column_spacing:20.0,
        StackPanel{spacing:6.0,
            TextBlock{text:title,font_size:16.0,font_weight:FontWeight::Bold,foreground:"#1155CC"}
            {paragraph(description)}
        }
        Border{Grid.column:1,vertical_alignment:V::Center,{reader_controls::action("Subscribe",move||model.subscribe(url.to_string()))}}
    }});
    }
    scroll(content)
}
fn scroll(content: El<StackPanel>) -> Element {
    scroll_viewer()
        .horizontal_scroll_bar_visibility(ScrollBarVisibility::Disabled)
        .vertical_scroll_bar_visibility(ScrollBarVisibility::Auto)
        .child(border().padding((24.0, 26.0, 36.0, 32.0)).child(content))
        .into()
}
