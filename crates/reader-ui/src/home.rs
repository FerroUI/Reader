mod empty;
mod preview;
use crate::session::{Panel, Session};
use ferroui_base::layout::{HorizontalAlignment as H, VerticalAlignment as V};
use ferroui_base::media::{FontWeight, TextWrapping};
use ferroui_controls::primitives::ScrollBarVisibility;
use ferroui_view::prelude::*;
use reader_controls::{action_row, link, paragraph, separator};
use reader_domain::View;
use reader_state::AppViewModel;
use std::rc::Rc;

pub fn view(vm: Rc<AppViewModel>, ui: Rc<Session>) -> Element {
    match vm.view() {
        View::Trends => trends(vm, ui),
        View::Explore => explore(vm, ui),
        _ => welcome(vm, ui),
    }
}
fn welcome(vm: Rc<AppViewModel>, ui: Rc<Session>) -> Element {
    let narrow = crate::responsive::narrow(&vm);
    let all = vm.clone();
    let add = ui.clone();
    let shortcuts = ui.clone();
    let imports = ui.clone();
    let unread = vm.count(&View::All);
    let mut content = stack_panel()
        .spacing(9.0)
        .horizontal_alignment(H::Stretch)
        .child(
            stack_panel()
                .spacing(0.0)
                .child(
                    text_block()
                        .text("Some items of interest")
                        .font_size(13.0)
                        .font_weight(FontWeight::Normal)
                        .foreground("#777777")
                        .height(15.0)
                        .line_height(15.0),
                )
                .child(border().height(1.0).background("#EBEBEB")),
        );
    if vm.feeds().is_empty() {
        content=content.child(text_block().text("Welcome to Google Reader").font_size(22.0).text_wrapping(TextWrapping::Wrap))
            .child(paragraph("Keep up with your favorite websites. Reader brings their latest updates together in one place."))
            .child(reader_controls::primary("Add a subscription",move||add.show(Panel::Subscribe)))
            .child(link("Import your subscriptions",move||imports.show(Panel::ImportExport)));
    } else {
        let state = vm.state();
        let mut groups = 0;
        for feed in &state.feeds {
            let articles: Vec<_> = state
                .articles
                .iter()
                .filter(|article| article.feed_id == feed.id && !article.read)
                .take(2)
                .collect();
            if articles.is_empty() {
                continue;
            }
            let navigate = vm.clone();
            let id = feed.id;
            let mut group = stack_panel()
                .key(format!("overview-feed:{id}"))
                .spacing(8.0)
                .margin((0.0, 0.0, 0.0, 16.0))
                .child(
                    button()
                        .class("reader-link")
                        .font_size(20.0)
                        .font_weight(FontWeight::Bold)
                        .child(
                            text_block()
                                .text(feed.title.clone())
                                .text_wrapping(TextWrapping::Wrap),
                        )
                        .on_click(move |_| navigate.navigate(View::Feed(id))),
                );
            for article in articles {
                group = group.child(preview::view(vm.clone(), article));
            }
            content = content.child(group);
            groups += 1;
            if groups == 4 {
                break;
            }
        }
        if groups == 0 {
            content = content.child(empty::view(vm.clone()));
        }
    }
    content = content.child(link(
        &format!("View all {unread} unread items »"),
        move || all.navigate(View::All),
    ));
    let wide = vm.viewport() > 1050.0;
    let mut overview = grid()
        .column_definitions(if wide { "*,360" } else { "*" })
        .row_definitions(if wide { "Auto,Auto" } else { "Auto,Auto,Auto" })
        .column_spacing(40.0)
        .row_spacing(24.0)
        .child(content);
    let tip_add = ui.clone();
    let tip_import = ui.clone();
    let tips=border().background("#F9F9F9").border_brush("#DDDDDD").border_thickness(1.0).vertical_alignment(V::Top).margin((0.0,13.0,0.0,0.0))
        .grid_column(if wide{1}else{0}).grid_row(if wide{0}else{1}).child(rsx!{Grid{row_definitions:"38,Auto",
            Border{border_brush:"#DDDDDD",border_thickness:(0.0,0.0,0.0,1.0),padding:(16.0,0.0),
                TextBlock{text:"Tips and tricks",font_size:14.0,font_weight:FontWeight::Bold,foreground:"#333333",vertical_alignment:V::Center}
            }
            Border{Grid.row:1,padding:16.0,
                StackPanel{spacing:14.0,
                    {paragraph("Read the web in one place. Subscribe to your favorite sites and get their latest posts in Reader.")}
                    {link("Add a subscription",move||tip_add.show(Panel::Subscribe))}
                    {paragraph("Use j and k to move between items, and s to star something you want to keep.")}
                    {link("Learn more keyboard shortcuts »",move||shortcuts.show(Panel::Shortcuts))}
                    {link("Import your subscriptions »",move||tip_import.show(Panel::ImportExport))}
                }
            }
        }});
    overview = overview.child(tips);
    let about = ui.clone();
    let footer = stack_panel()
        .spacing(9.0)
        .horizontal_alignment(H::Center)
        .child(link("About FerroUI Reader", move || {
            about.show(Panel::About)
        }));
    overview = overview.child(
        border()
            .grid_column_span(if wide { 2 } else { 1 })
            .grid_row(if wide { 1 } else { 2 })
            .margin((0.0, 18.0, 0.0, 0.0))
            .child(footer),
    );
    scroll_inset(
        overview.into(),
        if narrow {
            (16.0, 20.0, 16.0, 24.0)
        } else {
            (13.0, 28.0, 46.0, 32.0)
        },
        &ui,
    )
}
fn trends(vm: Rc<AppViewModel>, ui: Rc<Session>) -> Element {
    let narrow = crate::responsive::narrow(&vm);
    let columns = if narrow { "*,48,48,56" } else { "*,80,80,100" };
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
        .child(rsx! {Grid{column_definitions:columns,
            TextBlock{text:"Subscription",font_weight:FontWeight::Bold,text_wrapping:TextWrapping::Wrap}
            TextBlock{Grid.column:1,text:"Items",font_weight:FontWeight::Bold}
            TextBlock{Grid.column:2,text:"Read",font_weight:FontWeight::Bold}
            TextBlock{Grid.column:3,text:"Read %",font_weight:FontWeight::Bold}
        }});
    for feed in data.feeds {
        let model = vm.clone();
        let id = feed.feed_id;
        content = content.child(rsx! {Grid{column_definitions:columns,column_spacing:if narrow{4.0}else{0.0},min_height:25.0,
            {link(&feed.title,move||model.navigate(View::Feed(id)))}
            TextBlock{Grid.column:1,text:feed.total.to_string(),vertical_alignment:V::Center}
            TextBlock{Grid.column:2,text:feed.read.to_string(),vertical_alignment:V::Center}
            TextBlock{Grid.column:3,text:format!("{:.0}%",feed.percent),vertical_alignment:V::Center}
        }});
    }
    scroll(content.into(), narrow, &ui)
}
fn explore(vm: Rc<AppViewModel>, ui: Rc<Session>) -> Element {
    let narrow = crate::responsive::narrow(&vm);
    let subscribe = ui.clone();
    let imports = ui.clone();
    let mut content = stack_panel()
        .spacing(18.0)
        .max_width(800.0)
        .horizontal_alignment(H::Stretch)
        .child(
            text_block()
                .text("Discover new subscriptions")
                .font_size(22.0)
                .text_wrapping(TextWrapping::Wrap),
        )
        .child(paragraph(
            "Browse a few starting points, or add any RSS or Atom feed you follow.",
        ))
        .child(action_row(
            [
                reader_controls::action("Add a subscription", move || {
                    subscribe.show(Panel::Subscribe)
                }),
                reader_controls::action("Import subscriptions", move || {
                    imports.show(Panel::ImportExport)
                }),
            ],
            10.0,
        ));
    for feed in reader_domain::DEFAULT_FEEDS {
        let (title, description, url) = (feed.title, feed.description, feed.url);
        let model = vm.clone();
        let existing = vm
            .state()
            .feeds
            .iter()
            .find(|saved| saved.url == url)
            .map(|saved| saved.id);
        content=content.child(separator()).child(rsx!{Grid{column_definitions:if narrow{"*"}else{"*,Auto"},row_definitions:if narrow{"Auto,Auto"}else{"Auto"},column_spacing:20.0,row_spacing:8.0,
        StackPanel{spacing:6.0,
            TextBlock{text:title,font_size:16.0,font_weight:FontWeight::Bold,foreground:"#1155CC",text_wrapping:TextWrapping::Wrap}
            {paragraph(description)}
        }
        Border{Grid.column:if narrow{0}else{1},Grid.row:if narrow{1}else{0},horizontal_alignment:if narrow{H::Left}else{H::Stretch},vertical_alignment:V::Center,{reader_controls::action(if existing.is_some(){"Read"}else{"Subscribe"},move||{
            if let Some(id)=existing { model.navigate(View::Feed(id)); }
            else { model.subscribe(url.to_string()); }
        })}}
    }});
    }
    scroll(content.into(), narrow, &ui)
}
fn scroll(content: Element, narrow: bool, ui: &Session) -> Element {
    scroll_inset(
        content,
        if narrow {
            (16.0, 20.0, 16.0, 24.0)
        } else {
            (24.0, 26.0, 36.0, 32.0)
        },
        ui,
    )
}
fn scroll_inset(content: Element, padding: (f64, f64, f64, f64), ui: &Session) -> Element {
    scroll_viewer()
        .name("ReaderPageScroll")
        .node_ref(ui.page_scroll)
        .horizontal_scroll_bar_visibility(ScrollBarVisibility::Disabled)
        .vertical_scroll_bar_visibility(ScrollBarVisibility::Auto)
        .child(border().padding(padding).child(content))
        .into()
}
