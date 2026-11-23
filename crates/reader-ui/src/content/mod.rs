mod cache;
mod table;
mod text;
use crate::rows::Row;
use ferroui_base::layout::HorizontalAlignment as H;
use ferroui_base::media::TextWrapping;
use ferroui_controls::primitives::ScrollBarVisibility;
use ferroui_view::prelude::*;
use reader_state::AppViewModel;
use std::rc::Rc;

pub fn view(vm: Rc<AppViewModel>, row: &Row) -> Element {
    let article = &row.article;
    let size = row.scale.text_size(row.font_size, 0);
    let subdued = article.read && !row.selected;
    let (fingerprint, blocks) =
        cache::project(article.reading_content(), article.reading_base_url());
    let mut body = stack_panel()
        .name("ArticleBody")
        .max_width(crate::reading::scale::body_max_width(row.wide_body))
        .horizontal_alignment(H::Left)
        .spacing(14.0 * row.scale.factor());
    if blocks.is_empty() {
        body = body.child(text::view(
            vm.clone(),
            &[reader_content::projection::Span {
                text: article.summary.clone(),
                ..Default::default()
            }],
            size,
            0,
            false,
            subdued,
            format!("{}-{fingerprint}-summary", article.id),
        ));
    }
    for (index, block) in blocks.iter().enumerate() {
        let key = format!("{}-{fingerprint}-{index}", article.id);
        if let Some(image) = &block.image {
            body = body.child(crate::images::with_metadata(
                vm.clone(),
                image.clone(),
                key,
                row.media_width,
            ));
            continue;
        }
        if let Some(table) = &block.table {
            body = body.child(table::view(vm.clone(), table, size, subdued, &key));
            continue;
        }
        if block.rule {
            body = body.child(reader_controls::separator());
            continue;
        }
        let content = text::view(
            vm.clone(),
            &block.spans,
            row.scale.text_size(row.font_size, block.heading),
            block.heading,
            block.pre,
            subdued,
            key,
        );
        let content = if block.pre {
            border()
                .background("#F5F5F5")
                .padding(12.0)
                .child(
                    scroll_viewer()
                        .name("ArticleCodeScroll")
                        .horizontal_scroll_bar_visibility(ScrollBarVisibility::Auto)
                        .vertical_scroll_bar_visibility(ScrollBarVisibility::Disabled)
                        .child(content),
                )
                .into()
        } else {
            content
        };
        let content = if block.quote {
            border()
                .border_brush("#CCCCCC")
                .border_thickness((3.0, 0.0, 0.0, 0.0))
                .padding((14.0 * row.scale.factor(), 0.0))
                .child(content)
                .into()
        } else {
            content
        };
        let content = if block.list_depth > 1 {
            border()
                .margin((
                    (block.list_depth - 1) as f64 * 18.0 * row.scale.factor(),
                    0.0,
                    0.0,
                    0.0,
                ))
                .child(content)
                .into()
        } else {
            content
        };
        body = body.child(if block.notice {
            border()
                .background("#FFF7D1")
                .padding(12.0)
                .child(content)
                .into()
        } else {
            content
        });
    }
    if !article.note.is_empty() {
        body = body.child(
            border().background("#FFF9DC").padding(12.0).child(
                text_block()
                    .text(format!("Your note: {}", article.note))
                    .font_size(size)
                    .text_wrapping(TextWrapping::Wrap),
            ),
        );
    }
    body.into()
}
