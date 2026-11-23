use ferroui_base::media::{Color, GradientStop, GradientStops, IBrush, LinearGradientBrush};
use ferroui_base::{RelativePoint, RelativeUnit, Visual};
use ferroui_controls::{primitives::TemplatedControl, Application, Border};
use ferroui_view::style;
use std::rc::Rc;

pub(super) fn install(app: &Application) {
    app.styles().add(
        style("Button.reader-button.reader-menu-open")
            .set(TemplatedControl::background_property(), opened_background())
            .set(TemplatedControl::border_brush_property(), "#CCCCCC")
            .set(TemplatedControl::foreground_property(), "#333333")
            .build(),
    );
    // Material defines separate brushes for checked + selected/focused. Cover
    // those combinations so native radio choices retain Reader's neutral ink.
    for state in [
        ":selected",
        ":pointerover",
        ":focus-visible",
        ":open",
        ":checked:selected",
        ":checked:pointerover",
        ":checked:focus-visible",
        ":checked:open",
    ] {
        app.styles().add(
            style(&format!(
                "MenuItem.reader-menu{state} /template/ Border#StateLayer"
            ))
            .set(Border::background_property(), "#EEEEEE")
            .set(Visual::opacity_property(), 1.0)
            .build(),
        );
    }
    app.styles().add(
        style("MenuItem.reader-menu:disabled")
            .set(TemplatedControl::foreground_property(), "#CCCCCC")
            .build(),
    );
    app.styles().add(
        style("MenuItem.reader-menu:disabled /template/ Border#LayoutRoot")
            .set(Visual::opacity_property(), 1.0)
            .build(),
    );
    app.styles().add(
        style("MenuItem.reader-menu:disabled /template/ Border#StateLayer")
            .set(Border::background_property(), "Transparent")
            .build(),
    );
}

fn opened_background() -> Rc<dyn IBrush> {
    let brush = LinearGradientBrush::new();
    brush.set_start_point(RelativePoint::TOP_LEFT);
    brush.set_end_point(RelativePoint::new(0.0, 1.0, RelativeUnit::Relative));
    brush.set_gradient_stops(GradientStops::from_items([
        GradientStop::with_color_and_offset(Color::from_rgb(238, 238, 238), 0.0),
        GradientStop::with_color_and_offset(Color::from_rgb(224, 224, 224), 1.0),
    ]));
    brush.into()
}
