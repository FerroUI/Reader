//! The retained list headline uses one native shaped layout for title/snippet.
use crate::rows::Row;
use ferroui_base::layout::{HorizontalAlignment as H, VerticalAlignment as V};
use ferroui_base::media::{Brush, FontWeight, TextTrimming};
use ferroui_controls::documents::Run;
use ferroui_view::prelude::*;
use reader_controls::{icon, Icon};
use reader_state::AppViewModel;
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    rc::Rc,
};

pub(super) fn view(vm: Rc<AppViewModel>, row: &Row) -> El<Border> {
    let id = row.article.id;
    let star = vm.clone();
    let external = vm.clone();
    let open = vm.clone();
    let row_open = vm;
    let height = super::headline_height(row);
    let factor = row.scale.factor();
    let action_width = if row.touch { 44.0 } else { 26.0 * factor };
    let content_height = height - if row.expanded && !row.touch { 1.0 } else { 0.0 };
    let mut line = grid()
        .name("ArticleRow")
        .tag(Some(Rc::new(id) as ferroui_base::BoxedValue))
        .with(move |node| {
            crate::gestures::attach_headline_tap(&node.clone().upcast(), row_open.clone())
        })
        .column_definitions(&format!(
            "{},{},*,{},{}",
            action_width,
            if row.narrow || row.single_source {
                0.0
            } else {
                169.0 * factor
            },
            if row.narrow {
                60.0 * factor
            } else {
                91.0 * factor
            },
            action_width
        ))
        .height(content_height)
        .background(if row.selected && !row.expanded {
            "#FFFFCC"
        } else if row.article.read {
            "#EEEEEE"
        } else {
            "#FFFFFF"
        });
    line = line.child(
        button()
            .class("reader-flat")
            .width(action_width)
            .height(content_height)
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
    if !row.narrow && !row.single_source {
        line = line.child(
            text_block()
                .name("ArticleHeadlineSource")
                .grid_column(1)
                .text(row.source.to_string())
                .font_size(13.0 * factor)
                .foreground("#555555")
                // Original source: left 1.85em, width 11em, right padding 1em.
                .width(143.0 * factor)
                .horizontal_alignment(H::Left)
                .text_trimming(<dyn TextTrimming>::character_ellipsis())
                .margin((-1.95 * factor, 0.0, 13.0 * factor, 0.0))
                .vertical_alignment(V::Center),
        );
    }
    line = line.child(
        button()
            .class("reader-flat")
            .grid_column(2)
            .horizontal_content_alignment(H::Stretch)
            .clip_to_bounds(true)
            .automation_properties_name(row.article.title.clone())
            .on_click(move |event| {
                event.set_handled(true);
                toggle(&open, id);
            })
            .child(title(row)),
    );
    line = line.child(
        text_block()
            .name("ArticleHeadlineDate")
            .grid_column(3)
            .text(super::date(row.article.published))
            .font_size(13.0 * factor)
            .foreground("#555555")
            .horizontal_alignment(H::Right)
            .vertical_alignment(V::Center)
            .margin((5.0, 0.0, 4.0, 0.0)),
    );
    line = line.child(
        button()
            .name("ArticleHeadlineOriginal")
            .class("reader-flat")
            .grid_column(4)
            .height(content_height)
            .width(action_width)
            .automation_properties_name("View original article")
            .tool_tip_tip("View original article (v)")
            .on_click(move |event| {
                event.set_handled(true);
                external.open_article(id);
            })
            .child(icon(Icon::External, 13.0, "#999999")),
    );
    border()
        .border_brush("#EBEBEB")
        .border_thickness((0.0, 0.0, 0.0, if row.expanded { 1.0 } else { 0.0 }))
        .child(
            border()
                .name("ArticleHeadlineEdge")
                .border_brush(if row.selected {
                    "#4D90F0"
                } else if row.article.read {
                    "#EEEEEE"
                } else {
                    "#FFFFFF"
                })
                .border_thickness((1.0, 0.0, 0.0, 0.0))
                .child(line),
        )
}

pub(crate) fn toggle(vm: &AppViewModel, id: reader_domain::Id) {
    if vm.selected() == Some(id) {
        vm.select(None);
    } else {
        vm.select(Some(id));
    }
}

fn title(row: &Row) -> El<TextBlock> {
    let title = row.article.title.clone();
    let summary = if row.narrow || row.article.summary.is_empty() {
        String::new()
    } else {
        format!(" - {}", row.article.summary)
    };
    let read = row.article.read;
    let mut key = DefaultHasher::new();
    (&title, &summary, read).hash(&mut key);
    text_block()
        .key(format!("headline-{}-{}", row.article.id, key.finish()))
        .name("ArticleHeadlineText")
        .font_size(13.0 * row.scale.factor())
        .foreground("#000000")
        .text_trimming(<dyn TextTrimming>::character_ellipsis())
        .vertical_alignment(V::Center)
        .with(move |text| {
            let Some(inlines) = text.inlines() else {
                return;
            };
            let name = Run::with_text(Some(&title));
            name.set_font_weight(if read {
                FontWeight::Normal
            } else {
                FontWeight::Bold
            });
            inlines.add(name);
            if !summary.is_empty() {
                let snippet = Run::with_text(Some(&summary));
                snippet.set_foreground(Some(Brush::parse("#777777").expect("Reader palette")));
                inlines.add(snippet);
            }
        })
}
