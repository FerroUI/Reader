mod header;
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
    let row_open = vm.clone();
    let height = if row.touch {
        44.0
    } else {
        match row.density {
            Density::Comfortable => 37.0,
            Density::Cozy => 31.0,
            Density::Compact => 27.0,
        }
    };
    let source_width = if row.narrow { "0" } else { "165" };
    let mut line = grid()
        .name("ArticleRow")
        .tag(Some(Rc::new(id) as ferroui_base::BoxedValue))
        .with(move |node| {
            crate::gestures::attach_article_tap(&node.clone().upcast(), row_open.clone())
        })
        .column_definitions(&format!(
            "{},{source_width},*,70",
            if row.touch { 44 } else { 28 }
        ))
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
            .width(if row.touch { 44.0 } else { 28.0 })
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
            Grid {column_definitions:if row.narrow{"*"}else{"Auto,*"},
                TextBlock {text:row.article.title.clone(),font_size:13.0,font_weight:if row.article.read{FontWeight::Normal}else{FontWeight::Bold},text_trimming:<dyn TextTrimming>::character_ellipsis(),max_width:if row.narrow{f64::INFINITY}else{570.0},vertical_alignment:V::Center}
                if !row.narrow {TextBlock {Grid.column:1,text:format!(" — {}",row.article.summary),foreground:"#777777",font_size:13.0,text_trimming:<dyn TextTrimming>::character_ellipsis(),vertical_alignment:V::Center}}
            }
        }));
    line = line.child(
        text_block()
            .grid_column(3)
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
    let select = vm.clone();
    let spacing = if row.narrow {
        12.0
    } else {
        match row.density {
            Density::Comfortable => 20.0,
            Density::Cozy => 10.0,
            Density::Compact => 5.0,
        }
    };
    let body = content::view(vm.clone(), &row.article, row.font_size, row.selected);
    let mut card = grid()
        .name("ArticleEntry")
        .tag(Some(Rc::new(id) as ferroui_base::BoxedValue))
        .row_definitions("Auto,Auto,Auto")
        .background("White");
    card = card.child(header::view(vm.clone(), &row, spacing));
    card = card.child(
        border()
            .grid_row(1)
            .margin((spacing, 6.0, spacing, 20.0))
            .horizontal_alignment(H::Stretch)
            .with(move |node| {
                crate::gestures::attach_article_tap(&node.clone().upcast(), select.clone())
            })
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
    let height = if row.touch { 44.0 } else { 26.0 };
    let mut actions = wrap_panel()
        .orientation(Orientation::Horizontal)
        .child(button().class("reader-link").height(height).padding((8.0,0.0))
            .automation_properties_name(if row.article.starred{"Unstar"}else{"Add star"})
            .on_click(move |event|{event.set_handled(true);star.toggle_star(id);})
            .child(rsx!{StackPanel{orientation:Orientation::Horizontal,spacing:4.0,vertical_alignment:V::Center,
                {icon(if row.article.starred{Icon::Starred}else{Icon::Star},12.0,if row.article.starred{"#E9BC36"}else{"#777777"})}
                TextBlock{text:if row.article.starred{"Unstar"}else{"Add star"},font_size:if row.touch{13.0}else{11.0},foreground:"#666666",vertical_alignment:V::Center}
            }}))
        .child(footer_link(
            row.touch,
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
        .child(footer_link(row.touch,"Email", move || email.email_article(id)))
        .child(footer_link(row.touch,"Send to", move || share.share_article(id)))
        .child(footer_link(row.touch,"Edit tags", move || tags.show(Panel::Tags(id))))
        .child(footer_link(row.touch,"Add note", move || note.show(Panel::Note(id))))
        .child(footer_link(row.touch,"Print", move || print.print_article(id)));
    if !row.article.tags.is_empty() {
        actions = actions.child(
            text_block()
                .text(format!("Tags: {}", row.article.tags.join(", ")))
                .font_size(11.0)
                .text_wrapping(TextWrapping::Wrap)
                .max_width(if row.narrow {
                    (vm.viewport() - 72.0).max(1.0)
                } else {
                    650.0
                })
                .foreground("#666666")
                .vertical_alignment(V::Center)
                .margin((8.0, 0.0, 0.0, 0.0)),
        );
    }
    border()
        .background("#FAFAFA")
        .border_brush("#EBEBEB")
        .border_thickness((0.0, 1.0, 0.0, 0.0))
        .padding((if row.narrow { 4.0 } else { 12.0 }, 2.0))
        .child(actions)
}
fn footer_link(touch: bool, label: &str, action: impl Fn() + 'static) -> Element {
    button()
        .class("reader-link")
        .font_size(if touch { 13.0 } else { 11.0 })
        .foreground("#666666")
        .padding((8.0, 0.0))
        .height(if touch { 44.0 } else { 26.0 })
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

pub fn full_date(timestamp: i64) -> String {
    chrono::DateTime::from_timestamp(timestamp, 0)
        .map(|date| date.format("%b %-d, %Y %-I:%M %p").to_string())
        .unwrap_or_default()
}
