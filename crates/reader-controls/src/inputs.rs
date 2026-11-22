//! Reader outlines and editing colors on the retained Material TextBox template.
use ferroui_base::layout::Layoutable;
use ferroui_base::media::FontFamily;
use ferroui_base::Visual;
use ferroui_controls::{primitives::TemplatedControl, Application, Border, TextBox};
use ferroui_view::style;

pub(crate) fn install(app: &Application) {
    app.styles().add(
        style("TextBox.reader-input")
            .set(
                TemplatedControl::font_family_property(),
                FontFamily::new("fonts:Reader#Liberation Sans"),
            )
            .set(TemplatedControl::font_size_property(), 13.0)
            .set(TemplatedControl::foreground_property(), "#333333")
            .set(TemplatedControl::background_property(), "#FFFFFF")
            .set(TemplatedControl::corner_radius_property(), 0.0)
            .set(TemplatedControl::border_thickness_property(), 1.0)
            .set(TemplatedControl::border_brush_property(), "#D9D9D9")
            .set(Layoutable::min_height_property(), 0.0)
            .set(TextBox::caret_brush_property(), "#333333")
            .set(TextBox::selection_brush_property(), "#C6DAFC")
            .set(TextBox::selection_foreground_brush_property(), "#000000")
            .set(TextBox::placeholder_foreground_property(), "#777777")
            .build(),
    );
    // Material's state styles write directly to this part, bypassing the outer
    // TextBox border binding. Override the same part, retaining its editor/layout.
    for (state, outline, background) in [
        ("", "#D9D9D9", "#FFFFFF"),
        (":pointerover", "#B9B9B9", "#FFFFFF"),
        (":focus", "#4D90FE", "#FFFFFF"),
        (":disabled", "#E5E5E5", "#F5F5F5"),
    ] {
        app.styles().add(
            style(&format!(
                "TextBox.reader-input{state} /template/ Border#BorderElement"
            ))
            .set(Border::border_brush_property(), outline)
            .set(Border::border_thickness_property(), 1.0)
            .set(Border::corner_radius_property(), 0.0)
            .set(Border::background_property(), background)
            .build(),
        );
    }
    app.styles().add(
        style("TextBox.reader-input:disabled")
            .set(TemplatedControl::foreground_property(), "#AAAAAA")
            .set(TextBox::placeholder_foreground_property(), "#AAAAAA")
            .build(),
    );
    app.styles().add(
        style("TextBox.reader-input:focus /template/ Button#DeleteButton")
            .set(Visual::is_visible_property(), false)
            .build(),
    );
}
