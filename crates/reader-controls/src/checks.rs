//! Reader selection colors on the retained native Material checkbox template.
use ferroui_base::Visual;
use ferroui_controls::{primitives::TemplatedControl, Application, Border};
use ferroui_view::style;

pub fn install(app: &Application) {
    app.styles().add(
        style("CheckBox.reader-check")
            .set(TemplatedControl::foreground_property(), "#333333")
            .build(),
    );
    app.styles().add(
        style("CheckBox.reader-check /template/ Border#SelectionOutline")
            .set(Border::border_brush_property(), "#AAAAAA")
            .set(Border::corner_radius_property(), 1.0)
            .build(),
    );
    for selector in [
        "CheckBox.reader-check:checked /template/ Border#SelectionOutline",
        "CheckBox.reader-check:indeterminate /template/ Border#SelectionOutline",
        "CheckBox.reader-check:checked /template/ Border#MaterialStateLayer",
    ] {
        app.styles().add(
            style(selector)
                .set(Border::background_property(), "#4D90FE")
                .build(),
        );
    }
    app.styles().add(
        style("CheckBox.reader-check /template/ MaterialRipple")
            .set(Visual::is_visible_property(), false)
            .build(),
    );
}
