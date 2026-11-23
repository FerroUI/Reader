//! Compact, wrapping host notifications outside the reading scroll surface.
use ferroui_base::layout::VerticalAlignment;
use ferroui_base::media::TextWrapping;
use ferroui_view::prelude::*;
use reader_state::AppViewModel;
use std::rc::Rc;

pub fn update(vm: Rc<AppViewModel>) -> Option<Element> {
    let notice = vm.update_notice()?;
    let narrow = crate::responsive::narrow(&vm);
    let apply = vm.clone();
    let dismiss = vm.clone();
    Some(rsx! {
        Border {
            name: "ReaderUpdateNotice",
            background: "#FFF8CE", border_brush: "#F0C36D",
            border_thickness: (0.0, 1.0, 0.0, 0.0), padding: (14.0, 7.0),
            Grid {
                column_definitions: if narrow {"*,Auto"} else {"*,Auto,Auto"},
                row_definitions: if narrow {"Auto,Auto"} else {"Auto"}, column_spacing: 12.0,
                TextBlock {
                    name: "ReaderUpdateMessage",
                    Grid.column_span: if narrow {2} else {1},
                    text: notice.message, font_size: 12.0,
                    text_wrapping: TextWrapping::Wrap,
                    vertical_alignment: VerticalAlignment::Center,
                }
                if !notice.applying {
                    Border {Grid.column: if narrow {0} else {1}, Grid.row: if narrow {1} else {0},
                        {reader_controls::link("Update Reader", move || apply.apply_update())}
                    }
                    Border {Grid.column: if narrow {1} else {2}, Grid.row: if narrow {1} else {0},
                        {reader_controls::link("Later", move || dismiss.dismiss_update())}
                    }
                }
            }
        }
    })
}
