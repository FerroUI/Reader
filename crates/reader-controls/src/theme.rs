//! Reader's 2013 design tokens, applied after the Material base control theme.
use crate::compiled_markup::THEME_PARTS_FERROUI_THEMES_MATERIAL_MATERIALTHEME as THEME_PARTS;
use crate::templates::{flat_button_theme, list_item_theme};
use ferroui_base::layout::{HorizontalAlignment as H, Layoutable, VerticalAlignment as V};
use ferroui_base::media::{
    Color, FontFamily, FontWeight, GradientStop, GradientStops, IBrush, LinearGradientBrush,
};
use ferroui_base::styling::ThemeVariant;
use ferroui_base::{RelativePoint, StyledElement, Visual};
use ferroui_controls::primitives::{ScrollBarVisibility, TemplatedControl};
use ferroui_controls::{
    documents::TextElement, Application, ContentControl, ItemsControl, ScrollViewer,
};
use ferroui_view::style;
use std::rc::Rc;

pub fn install_theme(app: &Application) {
    app.set_requested_theme_variant(Some(ThemeVariant::light()));
    app.styles()
        .add(ferroui_themes_material::MaterialTheme::with_parts(None, THEME_PARTS).as_style());
    app.styles().add(
        style("Grid#ReaderApp")
            .set(
                TextElement::font_family_property(),
                FontFamily::new("fonts:Reader#Liberation Sans"),
            )
            .set(TextElement::font_size_property(), 13.0)
            .set(TextElement::foreground_property(), "#333333")
            .build(),
    );
    for selector in [
        "Button.reader-button",
        "Button.reader-primary",
        "Button.reader-subscribe",
        "Button.reader-link",
        "Button.reader-nav",
        "Button.reader-flat",
        "ToggleButton.reader-toggle",
    ] {
        app.styles().add(
            style(selector)
                .set(StyledElement::theme_property(), Some(flat_button_theme()))
                .set(
                    TemplatedControl::font_family_property(),
                    FontFamily::new("fonts:Reader#Liberation Sans"),
                )
                .set(TemplatedControl::font_size_property(), 11.0)
                .set(TemplatedControl::font_weight_property(), FontWeight::Bold)
                .set(TemplatedControl::padding_property(), (11.0, 5.0))
                .set(TemplatedControl::corner_radius_property(), 2.0)
                .set(TemplatedControl::border_thickness_property(), 1.0)
                .set(TemplatedControl::border_brush_property(), "#DCDCDC")
                .set(TemplatedControl::foreground_property(), "#444444")
                .set(
                    TemplatedControl::background_property(),
                    gradient("#F5F5F5", "#F1F1F1"),
                )
                .set(Layoutable::min_height_property(), 29.0)
                .set(
                    ContentControl::horizontal_content_alignment_property(),
                    H::Center,
                )
                .set(
                    ContentControl::vertical_content_alignment_property(),
                    V::Center,
                )
                .build(),
        );
        app.styles().add(
            style(&format!("{selector}:pointerover"))
                .set(TemplatedControl::border_brush_property(), "#C6C6C6")
                .set(TemplatedControl::background_property(), "#F8F8F8")
                .build(),
        );
        app.styles().add(
            style(&format!("{selector}:pressed"))
                .set(TemplatedControl::background_property(), "#E5E5E5")
                .build(),
        );
        app.styles().add(
            style(&format!("{selector}:focus-visible"))
                .set(TemplatedControl::border_brush_property(), "#4D90FE")
                .build(),
        );
        app.styles().add(
            style(&format!("{selector}:disabled"))
                .set(Visual::opacity_property(), 0.45)
                .build(),
        );
    }
    for selector in ["Button.reader-primary", "Button.reader-subscribe"] {
        let red = selector.ends_with("subscribe");
        app.styles().add(
            style(selector)
                .set(TemplatedControl::foreground_property(), "#FFFFFF")
                .set(
                    TemplatedControl::border_brush_property(),
                    if red { "#D14836" } else { "#3079ED" },
                )
                .set(
                    TemplatedControl::background_property(),
                    if red {
                        gradient("#DD4B39", "#D14836")
                    } else {
                        gradient("#4D90FE", "#4787ED")
                    },
                )
                .build(),
        );
        app.styles().add(
            style(&format!("{selector}:pointerover"))
                .set(
                    TemplatedControl::background_property(),
                    if red { "#C53727" } else { "#357AE8" },
                )
                .build(),
        );
    }
    for selector in [
        "Button.reader-link",
        "Button.reader-nav",
        "Button.reader-flat",
    ] {
        app.styles().add(
            style(selector)
                .set(TemplatedControl::background_property(), "Transparent")
                .set(TemplatedControl::border_brush_property(), "Transparent")
                .set(TemplatedControl::font_weight_property(), FontWeight::Normal)
                .set(TemplatedControl::font_size_property(), 13.0)
                .set(TemplatedControl::padding_property(), 0.0)
                .set(TemplatedControl::corner_radius_property(), 0.0)
                .set(Layoutable::min_height_property(), 0.0)
                .set(
                    ContentControl::horizontal_content_alignment_property(),
                    H::Left,
                )
                .build(),
        );
        app.styles().add(
            style(&format!("{selector}:pointerover"))
                .set(TemplatedControl::background_property(), "#F1F1F1")
                .build(),
        );
    }
    app.styles().add(
        style("Button.reader-link")
            .set(TemplatedControl::foreground_property(), "#1155CC")
            .build(),
    );
    app.styles().add(
        style("Button.reader-nav.selected")
            .set(TemplatedControl::foreground_property(), "#DD4B39")
            .set(TemplatedControl::font_weight_property(), FontWeight::Bold)
            .build(),
    );
    app.styles().add(
        style("Button.reader-nav.keyboard-cursor")
            .set(TemplatedControl::background_property(), "#DDDDDD")
            .build(),
    );
    app.styles().add(
        style("Button.reader-button.selected")
            .set(TemplatedControl::background_property(), "#D9D9D9")
            .set(TemplatedControl::border_brush_property(), "#BEBEBE")
            .build(),
    );
    crate::inputs::install(app);
    install_list(app);
    crate::menus::install(app);
    crate::checks::install(app);
    crate::read_state::install(app);
    crate::touch::install(app);
}
fn install_list(app: &Application) {
    app.styles().add(
        style("ListBox.reader-list")
            .set(TemplatedControl::background_property(), "White")
            .set(TemplatedControl::border_thickness_property(), 0.0)
            .set(TemplatedControl::padding_property(), 0.0)
            .set(
                ItemsControl::item_container_theme_property(),
                Some(list_item_theme()),
            )
            .set(
                ScrollViewer::horizontal_scroll_bar_visibility_property(),
                ScrollBarVisibility::Disabled,
            )
            .set(
                ScrollViewer::vertical_scroll_bar_visibility_property(),
                ScrollBarVisibility::Auto,
            )
            .build(),
    );
    app.styles().add(
        style("ListBox.reader-list ListBoxItem")
            .set(StyledElement::theme_property(), Some(list_item_theme()))
            .set(TemplatedControl::background_property(), "Transparent")
            .set(TemplatedControl::border_brush_property(), "Transparent")
            .set(TemplatedControl::border_thickness_property(), 0.0)
            .set(TemplatedControl::padding_property(), 0.0)
            .set(Layoutable::min_height_property(), 0.0)
            .set(
                ContentControl::horizontal_content_alignment_property(),
                H::Stretch,
            )
            .set(
                ContentControl::vertical_content_alignment_property(),
                V::Stretch,
            )
            .build(),
    );
}
fn gradient(top: &str, bottom: &str) -> Rc<dyn IBrush> {
    let brush = LinearGradientBrush::new();
    brush.set_start_point(RelativePoint::TOP_LEFT);
    brush.set_end_point(RelativePoint::new(
        0.0,
        1.0,
        ferroui_base::RelativeUnit::Relative,
    ));
    brush.set_gradient_stops(GradientStops::from_items([
        GradientStop::with_color_and_offset(Color::parse(top).expect("theme color"), 0.0),
        GradientStop::with_color_and_offset(Color::parse(bottom).expect("theme color"), 1.0),
    ]));
    brush.into()
}
