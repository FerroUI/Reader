use ferroui_controls::primitives::ScrollBarVisibility;
use ferroui_view::prelude::*;
use reader_content::projection::{CellPlacement, Span, Table};
use reader_state::AppViewModel;
use std::rc::Rc;

pub(super) fn view(vm: Rc<AppViewModel>, table: &Table, size: f64, key: &str) -> Element {
    let layout = table.layout();
    let mut grid = grid()
        .name("ArticleTable")
        .column_definitions(&vec!["*"; layout.columns].join(","))
        .row_definitions(&vec!["Auto"; layout.rows].join(","))
        .min_width((layout.columns as f64 * 120.0).min(3840.0));
    for CellPlacement {
        row,
        column,
        colspan,
        rowspan,
        cell,
    } in layout.cells
    {
        let text = super::text::view(
            vm.clone(),
            &cell.spans,
            size,
            if cell.heading { 6 } else { 0 },
            false,
            false,
            format!("{key}-{row}-{column}"),
        );
        grid = grid.child(
            border()
                .grid_row(row as i32)
                .grid_column(column as i32)
                .grid_row_span(rowspan as i32)
                .grid_column_span(colspan as i32)
                .background(if cell.heading {
                    "#F1F1F1"
                } else if row % 2 == 0 {
                    "#FAFAFA"
                } else {
                    "#FFFFFF"
                })
                .border_brush("#DDDDDD")
                .border_thickness(0.5)
                .padding(8.0)
                .child(text),
        );
    }
    let scroll = scroll_viewer()
        .name("ArticleTableScroll")
        .horizontal_scroll_bar_visibility(ScrollBarVisibility::Auto)
        .vertical_scroll_bar_visibility(ScrollBarVisibility::Disabled)
        .child(grid);
    if layout.overflow.is_empty() {
        return scroll.into();
    }
    let mut panel = stack_panel()
        .spacing(8.0)
        .child(scroll)
        .child(super::text::view(
            vm.clone(),
            &[Span {
                text: "Additional table columns are shown below.".into(),
                italic: true,
                ..Default::default()
            }],
            size,
            0,
            false,
            false,
            format!("{key}-overflow"),
        ));
    for (index, (row, cell)) in layout.overflow.into_iter().enumerate() {
        let mut spans = vec![Span {
            text: format!("Row {}: ", row + 1),
            bold: true,
            ..Default::default()
        }];
        spans.extend_from_slice(&cell.spans);
        panel = panel.child(super::text::view(
            vm.clone(),
            &spans,
            size,
            0,
            false,
            false,
            format!("{key}-extra-{index}"),
        ));
    }
    panel.into()
}
