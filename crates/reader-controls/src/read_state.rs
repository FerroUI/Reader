//! Reader's compact article checkbox replaces Material's fixed 48px icon column.
//! CheckBox continues to own toggling, access keys, focus and automation.
mod template;

use ferroui_base::layout::{HorizontalAlignment as H, Layoutable, VerticalAlignment as V};
use ferroui_base::media::{FontFamily, FontWeight};
use ferroui_base::styling::{ControlTheme, Setter};
use ferroui_base::{Ref, StyledElement, Visual};
use ferroui_controls::primitives::TemplatedControl;
use ferroui_controls::{Application, Border, CheckBox, ContentControl};
use ferroui_view::style;
use std::cell::OnceCell;

ferroui_base::ferro_thread_local! {
    static THEME: OnceCell<Ref<ControlTheme>> = const { OnceCell::new() };
}

pub(crate) fn install(app: &Application) {
    let theme = THEME.with(|slot| {
        slot.get_or_init(|| {
            let theme = ControlTheme::for_type::<CheckBox>();
            theme.add_setter(Setter::new(
                TemplatedControl::template_property(),
                Some(template::build()),
            ));
            theme
        })
        .clone()
    });
    app.styles().add(
        style("CheckBox.reader-read-state")
            .set(StyledElement::theme_property(), Some(theme))
            .set(Layoutable::min_height_property(), 26.0)
            .set(Layoutable::min_width_property(), 0.0)
            .set(
                TemplatedControl::font_family_property(),
                FontFamily::new("fonts:Reader#Liberation Sans"),
            )
            .set(TemplatedControl::font_size_property(), 13.0)
            .set(TemplatedControl::font_weight_property(), FontWeight::Normal)
            .set(TemplatedControl::padding_property(), (8.0, 0.0))
            .set(TemplatedControl::foreground_property(), "#444444")
            .set(TemplatedControl::background_property(), "Transparent")
            .set(
                ContentControl::horizontal_content_alignment_property(),
                H::Left,
            )
            .set(
                ContentControl::vertical_content_alignment_property(),
                V::Center,
            )
            .build(),
    );
    for (state, outline, background) in [
        ("", "#AAAAAA", "#FFFFFF"),
        (":checked", "#8E8E8E", "#F5F5F5"),
        (":indeterminate", "#8E8E8E", "#F5F5F5"),
        (":pointerover", "#777777", "#FFFFFF"),
        (":pressed", "#777777", "#E5E5E5"),
        (":focus-visible", "#4D90FE", "#FFFFFF"),
    ] {
        app.styles().add(
            style(&format!(
                "CheckBox.reader-read-state{state} /template/ Border#ReaderReadStateOutline"
            ))
            .set(Border::border_brush_property(), outline)
            .set(Border::background_property(), background)
            .build(),
        );
    }
    for part in [
        "PathIcon#ReaderReadStateCheck",
        "Border#ReaderReadStateDash",
    ] {
        app.styles().add(
            style(&format!("CheckBox.reader-read-state /template/ {part}"))
                .set(Visual::is_visible_property(), false)
                .build(),
        );
    }
    for (state, part) in [
        (":checked", "PathIcon#ReaderReadStateCheck"),
        (":indeterminate", "Border#ReaderReadStateDash"),
    ] {
        app.styles().add(
            style(&format!(
                "CheckBox.reader-read-state{state} /template/ {part}"
            ))
            .set(Visual::is_visible_property(), true)
            .build(),
        );
    }
    app.styles().add(
        style("CheckBox.reader-read-state:pointerover")
            .set(TemplatedControl::background_property(), "#F1F1F1")
            .build(),
    );
    app.styles().add(
        style("CheckBox.reader-read-state:disabled")
            .set(Visual::opacity_property(), 0.45)
            .build(),
    );
}
