//! Reader geometry on the native Material menu template preserves access keys and popups.
use ferroui_base::layout::Layoutable;
use ferroui_base::media::{FontFamily, Stretch};
use ferroui_base::Visual;
use ferroui_controls::{primitives::TemplatedControl, shapes::Shape, Application, Border};
use ferroui_view::style;

pub fn install(app: &Application) {
    app.styles().add(
        style("ContextMenu")
            .set(TemplatedControl::background_property(), "White")
            .set(TemplatedControl::foreground_property(), "#333333")
            .set(TemplatedControl::border_brush_property(), "#CCCCCC")
            .set(TemplatedControl::border_thickness_property(), 1.0)
            .set(TemplatedControl::corner_radius_property(), 0.0)
            .set(TemplatedControl::padding_property(), (0.0, 4.0))
            .set(Layoutable::margin_property(), 0.0)
            .set(Layoutable::min_height_property(), 0.0)
            .build(),
    );
    app.styles().add(
        style("MenuItem.reader-menu")
            .set(
                TemplatedControl::font_family_property(),
                FontFamily::new("fonts:Reader#Liberation Sans"),
            )
            .set(TemplatedControl::font_size_property(), 13.0)
            .set(TemplatedControl::padding_property(), (9.0, 3.0))
            .set(TemplatedControl::foreground_property(), "#333333")
            .set(TemplatedControl::background_property(), "White")
            .set(Layoutable::min_height_property(), 27.0)
            .build(),
    );
    app.styles().add(
        style("MenuItem.reader-menu:checked")
            .set(TemplatedControl::background_property(), "White")
            .set(TemplatedControl::foreground_property(), "#333333")
            .build(),
    );
    app.styles().add(
        style("MenuItem.reader-menu /template/ Path#CheckGlyph")
            .set(Layoutable::width_property(), 16.0)
            .set(Layoutable::height_property(), 16.0)
            .set(Layoutable::margin_property(), (0.0, 0.0, 8.0, 0.0))
            .set(Shape::stretch_property(), Stretch::Uniform)
            .build(),
    );
    app.styles().add(
        style("MenuItem.reader-menu /template/ MaterialRipple")
            .set(Visual::is_visible_property(), false)
            .build(),
    );
    for selector in [
        "MenuItem.reader-menu:selected /template/ Border#StateLayer",
        "MenuItem.reader-menu:pointerover /template/ Border#StateLayer",
    ] {
        app.styles().add(
            style(selector)
                .set(Border::background_property(), "#EEEEEE")
                .build(),
        );
    }
}
