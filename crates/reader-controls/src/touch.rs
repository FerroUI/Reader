//! Larger targets only on the responsive touch surface. Desktop Reader metrics stay intact.
use ferroui_base::layout::Layoutable;
use ferroui_controls::{primitives::TemplatedControl, Application};
use ferroui_view::style;

pub(crate) fn install(app: &Application) {
    for selector in [
        "Grid.reader-touch Button",
        "Grid.reader-touch CheckBox",
        "ContextMenu.reader-touch MenuItem.reader-menu",
    ] {
        app.styles().add(
            style(selector)
                .set(Layoutable::min_height_property(), 44.0)
                .set(Layoutable::min_width_property(), 44.0)
                .build(),
        );
    }
    app.styles().add(
        style("Grid.reader-touch TextBox.reader-input")
            .set(Layoutable::min_height_property(), 44.0)
            .set(TemplatedControl::font_size_property(), 16.0)
            .build(),
    );
}
