use crate::{
    content,
    rows::Row,
    session::{Panel, Session},
};
use ferroui_base::layout::{HorizontalAlignment as H, Orientation, VerticalAlignment as V};
use ferroui_base::media::{FontWeight, TextTrimming, TextWrapping};
use ferroui_view::prelude::*;
use reader_controls::{icon, Icon};
use reader_domain::Density;
use reader_state::AppViewModel;
use std::rc::Rc;

pub fn view(vm: Rc<AppViewModel>, ui: Rc<Session>, row: Row) -> Element {
    if row.expanded {
        expanded(vm, ui, row)
    } else {
        collapsed(vm, row)
    }
}
fn collapsed(vm: Rc<AppViewModel>, row: Row) -> Element {
    let id = row.article.id;
    let selected = row.selected;
    let star = vm.clone();
    let open = vm.clone();
    let source_open = vm.clone();
    let date_open = vm.clone();
    let height = match row.density {
        Density::Comfortable => 37.0,
        Density::Cozy => 31.0,
        Density::Compact => 27.0,
    };
    let source_width = if row.narrow { "0" } else { "165" };
    let mut line = grid()
        .name("ArticleRow")
        .tag(Some(Rc::new(id) as ferroui_base::BoxedValue))
        .column_definitions(&format!("28,{source_width},*,70"))
        .height(height)
        .background(if selected {
            "#FFFFCC"
        } else if row.article.read {
            "#EEEEEE"
        } else {
            "#FFFFFF"
        });
    line = line.child(
        button()
            .class("reader-flat")
            .width(28.0)
            .height(height)
            .automation_properties_name(if row.article.starred {
                "Remove star"
            } else {
                "Add star"
            })
            .on_click(move |event| {
                event.set_handled(true);
                star.toggle_star(id);
            })
            .child(icon(
                if row.article.starred {
                    Icon::Starred
                } else {
                    Icon::Star
                },
                14.0,
                if row.article.starred {
                    "#E6BA35"
                } else {
                    "#B8B8B8"
                },
            )),
    );
    if !row.narrow {
        line = line.child(
            text_block()
                .grid_column(1)
                .on_pointer_pressed(move |_| source_open.select(Some(id)))
                .text(row.source.to_string())
                .font_size(11.0)
                .foreground("#666666")
                .text_trimming(<dyn TextTrimming>::character_ellipsis())
                .margin((0.0, 0.0, 10.0, 0.0))
                .vertical_alignment(V::Center),
        );
    }
    line=line.child(button().class("reader-flat").grid_column(2).horizontal_content_alignment(H::Stretch).automation_properties_name(row.article.title.clone())
        .on_click(move |_|open.select(Some(id))).child(rsx!{
            Grid {column_definitions:"Auto,*",
                TextBlock {text:row.article.title.clone(),font_size:13.0,font_weight:if row.article.read{FontWeight::Normal}else{FontWeight::Bold},text_trimming:<dyn TextTrimming>::character_ellipsis(),max_width:if row.narrow{(vm.viewport()-126.0).max(110.0)}else{570.0},vertical_alignment:V::Center}
                if !row.narrow {TextBlock {Grid.column:1,text:format!(" — {}",row.article.summary),foreground:"#777777",font_size:13.0,text_trimming:<dyn TextTrimming>::character_ellipsis(),vertical_alignment:V::Center}}
            }
        }));
    line = line.child(
        text_block()
            .grid_column(3)
            .on_pointer_pressed(move |_| date_open.select(Some(id)))
            .text(date(row.article.published))
            .font_size(11.0)
            .foreground("#666666")
            .horizontal_alignment(H::Right)
            .margin((5.0, 0.0, 10.0, 0.0))
            .vertical_alignment(V::Center),
    );
    border()
        .border_brush(if selected { "#4D90F0" } else { "#DDDDDD" })
        .border_thickness((if selected { 1.0 } else { 0.0 }, 0.0, 0.0, 1.0))
        .child(line)
        .into()
}
fn expanded(vm: Rc<AppViewModel>, ui: Rc<Session>, row: Row) -> Element {
    let id = row.article.id;
    let open = vm.clone();
    let star = vm.clone();
    let select = vm.clone();
    let source = vm.clone();
    let feed_id = row.article.feed_id;
    let spacing = match row.density {
        Density::Comfortable => 20.0,
        Density::Cozy => 10.0,
        Density::Compact => 5.0,
    };
    let title = row.article.title.clone();
    let body = content::view(vm.clone(), &row.article, row.font_size);
    let mut card = grid()
        .name("ArticleEntry")
        .tag(Some(Rc::new(id) as ferroui_base::BoxedValue))
        .row_definitions("Auto,Auto,Auto")
        .background("White");
    let mut header = grid()
        .column_definitions("28,*,Auto")
        .margin((spacing, spacing, spacing, 8.0));
    header = header.child(
        button()
            .class("reader-flat")
            .vertical_alignment(V::Top)
            .width(24.0)
            .height(25.0)
            .automation_properties_name(if row.article.starred {
                "Remove star"
            } else {
                "Add star"
            })
            .on_click(move |e| {
                e.set_handled(true);
                star.toggle_star(id);
            })
            .child(icon(
                if row.article.starred {
                    Icon::Starred
                } else {
                    Icon::Star
                },
                16.0,
                if row.article.starred {
                    "#E9BC36"
                } else {
                    "#B5B5B5"
                },
            )),
    );
    header=header.child(rsx! {
        StackPanel {Grid.column:1,spacing:5.0,
            Button {class:"reader-link",automation_properties_name:format!("Open {}",title),horizontal_content_alignment:H::Stretch,on_click:move |_|open.open_article(id),
                TextBlock {text:title,font_size:18.0,font_weight:FontWeight::Bold,foreground:if row.article.read{"#444444"}else{"#1155CC"},text_wrapping:TextWrapping::Wrap}
            }
            StackPanel {orientation:Orientation::Horizontal,spacing:4.0,
                {reader_controls::link(&row.source,move||source.navigate(reader_domain::View::Feed(feed_id)))}
                TextBlock {text:if row.article.author.is_empty(){String::new()}else{format!("by {}",row.article.author)},foreground:"#666666",font_size:12.0,vertical_alignment:V::Center}
            }
        }
    });
    if !row.narrow {
        header = header.child(
            text_block()
                .grid_column(2)
                .text(date(row.article.published))
                .font_size(11.0)
                .foreground("#666666")
                .margin((12.0, 2.0, 0.0, 0.0)),
        );
    }
    card = card.child(header);
    card = card.child(
        border()
            .grid_row(1)
            .margin((spacing + 28.0, 6.0, spacing, 20.0))
            .horizontal_alignment(H::Stretch)
            .on_pointer_pressed(move |_| select.select(Some(id)))
            .child(body),
    );
    card = card.child(footer(vm, ui, &row).grid_row(2));
    border()
        .border_brush(if row.selected { "#4D90F0" } else { "#DDDDDD" })
        .border_thickness(1.0)
        .margin((
            5.0,
            spacing,
            if row.narrow { 10.0 } else { spacing + 13.0 },
            spacing,
        ))
        .child(card)
        .into()
}
fn footer(vm: Rc<AppViewModel>, ui: Rc<Session>, row: &Row) -> El<Border> {
    let id = row.article.id;
    let star = vm.clone();
    let read = vm.clone();
    let was_read = row.article.read;
    let share = vm.clone();
    let email = vm.clone();
    let tags = ui.clone();
    let note = ui.clone();
    let print = vm.clone();
    let mut actions = wrap_panel()
        .orientation(Orientation::Horizontal)
        .item_height(26.0)
        .child(footer_link(
            if row.article.starred {
                "★ Unstar"
            } else {
                "☆ Add star"
            },
            move || star.toggle_star(id),
        ))
        .child(footer_link(
            if row.article.read {
                "Keep unread"
            } else {
                "Mark as read"
            },
            move || {
                if was_read {
                    read.keep_unread(id);
                } else {
                    read.set_read(id, true);
                }
            },
        ))
        .child(footer_link("Email", move || email.email_article(id)))
        .child(footer_link("Send to", move || share.share_article(id)))
        .child(footer_link("Edit tags", move || tags.show(Panel::Tags(id))))
        .child(footer_link("Add note", move || note.show(Panel::Note(id))))
        .child(footer_link("Print", move || print.print_article(id)));
    if !row.article.tags.is_empty() {
        actions = actions.child(
            text_block()
                .text(format!("Tags: {}", row.article.tags.join(", ")))
                .font_size(11.0)
                .foreground("#666666")
                .vertical_alignment(V::Center)
                .margin((8.0, 0.0, 0.0, 0.0)),
        );
    }
    border()
        .background("#F3F3F3")
        .border_brush("#EBEBEB")
        .border_thickness((0.0, 1.0, 0.0, 0.0))
        .padding((12.0, 2.0))
        .child(actions)
}
fn footer_link(label: &str, action: impl Fn() + 'static) -> Element {
    button()
        .class("reader-link")
        .font_size(11.0)
        .foreground("#666666")
        .padding((8.0, 0.0))
        .height(26.0)
        .content(label)
        .automation_properties_name(label)
        .on_click(move |e| {
            e.set_handled(true);
            action();
        })
        .into()
}
pub fn date(timestamp: i64) -> String {
    chrono::DateTime::from_timestamp(timestamp, 0)
        .map(|d| d.format("%b %-d").to_string())
        .unwrap_or_default()
}
