mod footer;
mod header;
mod headline;
mod layout;
mod reading;
use crate::{content, rows::Row, session::Session};
use ferroui_base::layout::HorizontalAlignment as H;
use ferroui_base::media::BoxShadows;
use ferroui_view::prelude::*;
pub(crate) use layout::{headline_height, right_inset};
use reader_state::AppViewModel;
use std::rc::Rc;

pub fn view(vm: Rc<AppViewModel>, ui: Rc<Session>, row: Row) -> Element {
    if row.list_mode {
        list_entry(vm, ui, row)
    } else {
        card(vm, ui, row)
    }
}

/// A list expansion appends its body below the same compact headline. Keeping
/// the headline at a stable key also retains its native focus and hit targets.
fn list_entry(vm: Rc<AppViewModel>, ui: Rc<Session>, row: Row) -> Element {
    let mut entry = grid()
        .name(if row.expanded {
            "ArticleEntry"
        } else {
            "ArticleListItem"
        })
        .tag(Some(Rc::new(row.article.id) as ferroui_base::BoxedValue))
        .row_definitions("Auto,Auto")
        .background("White")
        .child(headline::view(vm.clone(), &row).key("headline"));
    if row.expanded {
        entry = entry.child(
            border()
                .key("expanded-body")
                .grid_row(1)
                .name("ArticleListBody")
                .border_brush(if row.selected { "#4D90F0" } else { "#FFFFFF" })
                .border_thickness((1.0, 0.0, 0.0, 0.0))
                .child(details(vm, ui, &row, 13.0)),
        );
    }
    border()
        .border_brush("#CCCCCC")
        .border_thickness((0.0, 0.0, 0.0, 1.0))
        .child(entry)
        .into()
}

fn card(vm: Rc<AppViewModel>, ui: Rc<Session>, row: Row) -> Element {
    let spacing = layout::card_padding(&row);
    let mut frame = grid()
        .name("ArticleCard")
        .margin((
            5.0,
            spacing,
            if row.narrow {
                10.0
            } else {
                right_inset(row.density)
            },
            spacing,
        ))
        .child(
            border()
                .name("ArticleCardFrame")
                .border_brush("#DDDDDD")
                .border_thickness(1.0)
                .box_shadow(BoxShadows::parse("0 0 4 #E3E5EB").expect("Reader card shadow"))
                .child(details(vm, ui, &row, spacing)),
        );
    if row.selected {
        frame = frame.child(
            border()
                .name("ArticleSelectionEdge")
                .border_brush("#4D90F0")
                .border_thickness((1.0, 0.0, 0.0, 0.0))
                .is_hit_test_visible(false),
        );
    }
    frame.into()
}

fn details(vm: Rc<AppViewModel>, ui: Rc<Session>, row: &Row, padding: f64) -> Element {
    let select = vm.clone();
    let body = content::view(vm.clone(), &row.article, row.font_size, row.selected);
    grid()
        .name(if row.list_mode {
            "ArticleExpandedContent"
        } else {
            "ArticleEntry"
        })
        .tag(Some(Rc::new(row.article.id) as ferroui_base::BoxedValue))
        .row_definitions("Auto,Auto,Auto")
        .background("White")
        .child(header::view(vm.clone(), row, padding))
        .child(
            border()
                .grid_row(1)
                .margin((
                    padding,
                    6.5,
                    padding,
                    if row.list_mode { 6.5 } else { 20.0 },
                ))
                .horizontal_alignment(H::Stretch)
                .with(move |node| {
                    crate::gestures::attach_article_tap(&node.clone().upcast(), select.clone())
                })
                .child(
                    stack_panel()
                        .spacing(12.0)
                        .child(reading::view(vm.clone(), row))
                        .child(body),
                ),
        )
        .child(footer::view(vm, ui, row).grid_row(2))
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
