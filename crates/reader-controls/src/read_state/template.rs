use ferroui_base::data::TemplateBinding;
use ferroui_base::layout::{HorizontalAlignment as H, Orientation, VerticalAlignment as V};
use ferroui_base::media::{Brush, Geometry};
use ferroui_base::Thickness;
use ferroui_controls::automation::AutomationProperties;
use ferroui_controls::presenters::ContentPresenter;
use ferroui_controls::primitives::TemplatedControl;
use ferroui_controls::templates::{
    FuncControlTemplate, FuncTemplateNameScopeExtensions, IControlTemplate,
};
use ferroui_controls::{Border, ContentControl, Decorator, Grid, PathIcon, StackPanel};
use std::rc::Rc;

pub(super) fn build() -> Rc<dyn IControlTemplate> {
    let check = Geometry::parse("M0,4 L2,2 L4,4 L8,0 L10,2 L4,8 Z").expect("Reader check glyph");
    FuncControlTemplate::new(move |_, scope| {
        let root = Border::new();
        root.set_name(Some("ReaderReadStateRoot".into()));
        root.bind_binding(
            Border::background_property().as_property(),
            &*TemplateBinding::new(TemplatedControl::background_property().as_property()),
        );
        root.bind_binding(
            Decorator::padding_property().as_property(),
            &*TemplateBinding::new(TemplatedControl::padding_property().as_property()),
        );
        let row = StackPanel::new();
        row.set_orientation(Orientation::Horizontal);
        row.set_spacing(5.0);
        row.set_vertical_alignment(V::Center);

        let outline = Border::new();
        outline.set_name(Some("ReaderReadStateOutline".into()));
        outline.set_width(13.0);
        outline.set_height(13.0);
        outline.set_border_thickness(Thickness::uniform(1.0));
        outline.set_vertical_alignment(V::Center);
        outline.set_is_hit_test_visible(false);
        let marks = Grid::new();
        let tick = PathIcon::new();
        tick.set_name(Some("ReaderReadStateCheck".into()));
        tick.set_data(Some(check.clone()));
        tick.set_width(10.0);
        tick.set_height(8.0);
        tick.set_horizontal_alignment(H::Center);
        tick.set_vertical_alignment(V::Center);
        tick.set_foreground(Some(Brush::parse("#555555").expect("Reader glyph color")));
        AutomationProperties::set_is_control_element_override(&tick, Some(false));
        marks.children().add(tick.register_in_name_scope(&**scope));
        let dash = Border::new();
        dash.set_name(Some("ReaderReadStateDash".into()));
        dash.set_width(7.0);
        dash.set_height(2.0);
        dash.set_horizontal_alignment(H::Center);
        dash.set_vertical_alignment(V::Center);
        dash.set_background(Some(Brush::parse("#555555").expect("Reader glyph color")));
        AutomationProperties::set_is_control_element_override(&dash, Some(false));
        marks.children().add(dash.register_in_name_scope(&**scope));
        outline.set_child(marks);
        row.children().add(outline.register_in_name_scope(&**scope));

        let presenter = ContentPresenter::new();
        presenter.set_name(Some("PART_ContentPresenter".into()));
        presenter.set_recognizes_access_key(true);
        for (target, source) in [
            (
                ContentPresenter::content_property().as_property(),
                ContentControl::content_property().as_property(),
            ),
            (
                ContentPresenter::content_template_property().as_property(),
                ContentControl::content_template_property().as_property(),
            ),
            (
                ContentPresenter::foreground_property().as_property(),
                TemplatedControl::foreground_property().as_property(),
            ),
            (
                ContentPresenter::font_family_property().as_property(),
                TemplatedControl::font_family_property().as_property(),
            ),
            (
                ContentPresenter::font_size_property().as_property(),
                TemplatedControl::font_size_property().as_property(),
            ),
            (
                ContentPresenter::font_weight_property().as_property(),
                TemplatedControl::font_weight_property().as_property(),
            ),
            (
                ContentPresenter::horizontal_content_alignment_property().as_property(),
                ContentControl::horizontal_content_alignment_property().as_property(),
            ),
            (
                ContentPresenter::vertical_content_alignment_property().as_property(),
                ContentControl::vertical_content_alignment_property().as_property(),
            ),
        ] {
            presenter.bind_binding(target, &*TemplateBinding::new(source));
        }
        row.children()
            .add(presenter.register_in_name_scope(&**scope));
        root.set_child(row);
        root.register_in_name_scope(&**scope).upcast()
    })
}
