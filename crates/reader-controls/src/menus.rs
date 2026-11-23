//! Reader geometry on the native Material menu template preserves access keys and popups.
mod palette;
use ferroui_base::layout::Layoutable;
use ferroui_base::media::{BoxShadows, FontFamily, Geometry, Stretch};
use ferroui_base::Visual;
use ferroui_controls::{
    primitives::TemplatedControl,
    shapes::{Path, Shape},
    Application, Border, Control,
};
use ferroui_view::style;

pub fn install(app: &Application) {
    app.styles().add(
        style("Separator.reader-menu-separator")
            .set(TemplatedControl::background_property(), "#EBEBEB")
            .set(Layoutable::margin_property(), (0.0, 6.0))
            .build(),
    );
    app.styles().add(
        style("ContextMenu")
            .set(TemplatedControl::background_property(), "White")
            .set(TemplatedControl::foreground_property(), "#333333")
            .set(TemplatedControl::border_brush_property(), "#CCCCCC")
            .set(TemplatedControl::border_thickness_property(), 1.0)
            .set(TemplatedControl::corner_radius_property(), 0.0)
            .set(TemplatedControl::padding_property(), (0.0, 6.0))
            .set(Layoutable::margin_property(), 0.0)
            .set(Layoutable::min_height_property(), 0.0)
            .build(),
    );
    app.styles().add(
        style("ContextMenu /template/ Border#ThemeShadowWrapper")
            .set(
                Border::box_shadow_property(),
                BoxShadows::parse("0 2 4 #33000000").expect("Reader menu shadow"),
            )
            .set(Border::corner_radius_property(), 0.0)
            .build(),
    );
    app.styles().add(
        style("MenuItem.reader-menu")
            .set(
                TemplatedControl::font_family_property(),
                FontFamily::new("fonts:Reader#Liberation Sans"),
            )
            .set(TemplatedControl::font_size_property(), 13.0)
            .set(TemplatedControl::padding_property(), (16.0, 6.0, 44.0, 6.0))
            .set(TemplatedControl::foreground_property(), "#333333")
            .set(TemplatedControl::background_property(), "White")
            .set(Layoutable::min_height_property(), 27.0)
            .set(Layoutable::min_width_property(), 0.0)
            .set(Layoutable::max_width_property(), f64::INFINITY)
            .set(Control::focus_adorner_property(), None)
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
            .set(Layoutable::width_property(), 14.0)
            .set(Layoutable::height_property(), 14.0)
            .set(Layoutable::margin_property(), (-10.0, 0.0, 10.0, 0.0))
            .set(Shape::stretch_property(), Stretch::Uniform)
            .build(),
    );
    app.styles().add(
        style("MenuItem.reader-menu:radio /template/ Path#CheckGlyph")
            .set(
                Path::data_property(),
                Some(
                    Geometry::parse("M9 16.17 4.83 12 3.41 13.41 9 19 21 7 19.59 5.59Z")
                        .expect("Reader menu check"),
                ),
            )
            .set(Shape::stretch_property(), Stretch::Uniform)
            .build(),
    );
    app.styles().add(
        style("MenuItem.reader-menu /template/ MaterialRipple")
            .set(Visual::is_visible_property(), false)
            .build(),
    );
    palette::install(app);
}
