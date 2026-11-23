//! A wrapping settings checkbox without Material's fixed 48px icon column.
use super::presenter::build as presenter;
use ferroui_base::data::TemplateBinding;
use ferroui_base::layout::{HorizontalAlignment as H, VerticalAlignment as V};
use ferroui_base::media::{Brush, Geometry};
use ferroui_base::styling::{ControlTheme, Setter};
use ferroui_base::{Ref, Thickness, Visual};
use ferroui_controls::automation::AutomationProperties;
use ferroui_controls::primitives::TemplatedControl;
use ferroui_controls::templates::{
    FuncControlTemplate, FuncTemplateNameScopeExtensions, IControlTemplate,
};
use ferroui_controls::{Border, CheckBox, ColumnDefinitions, Grid, PathIcon};
use ferroui_view::style;
use std::{cell::OnceCell, rc::Rc};
ferroui_base::ferro_thread_local! {
    static CHECK: OnceCell<Ref<ControlTheme>> = const { OnceCell::new() };
}
pub fn theme() -> Ref<ControlTheme> {
    CHECK.with(|slot| {
        slot.get_or_init(|| {
            let theme = ControlTheme::for_type::<CheckBox>();
            let glyph = Geometry::parse("M0,4 L2,2 L4,4 L8,0 L10,2 L4,8 Z").expect("Reader check");
            let template: Rc<dyn IControlTemplate> = FuncControlTemplate::new(move |_, scope| {
                let root = Grid::new();
                root.set_column_definitions(ColumnDefinitions::parse("14,*").unwrap());
                root.set_column_spacing(8.0);
                root.bind_binding(
                    ferroui_controls::Panel::background_property().as_property(),
                    &*TemplateBinding::new(TemplatedControl::background_property().as_property()),
                );
                let outline = Border::new();
                outline.set_name(Some("SettingsCheckOutline".into()));
                outline.set_width(13.0);
                outline.set_height(13.0);
                outline.set_vertical_alignment(V::Center);
                outline.set_border_thickness(Thickness::uniform(1.0));
                let check = PathIcon::new();
                check.set_name(Some("SettingsCheckMark".into()));
                check.set_data(Some(glyph.clone()));
                check.set_width(10.0);
                check.set_height(8.0);
                check.set_horizontal_alignment(H::Center);
                check.set_vertical_alignment(V::Center);
                check.set_foreground(Some(Brush::parse("#FFFFFF").unwrap()));
                AutomationProperties::set_is_control_element_override(&check, Some(false));
                outline.set_child(check.register_in_name_scope(&**scope));
                root.children()
                    .add(outline.register_in_name_scope(&**scope));
                let presenter = presenter();
                Grid::set_column(&presenter, 1);
                root.children()
                    .add(presenter.register_in_name_scope(&**scope));
                root.upcast()
            });
            theme.add_setter(Setter::new(
                TemplatedControl::template_property(),
                Some(template),
            ));
            for (state, edge, fill) in [
                ("", "#AAAAAA", "#FFFFFF"),
                (":pointerover", "#777777", "#F8F8F8"),
                (":pressed", "#777777", "#E5E5E5"),
                (":checked", "#4D90FE", "#4D90FE"),
                (":checked:pointerover", "#3079ED", "#4D90FE"),
                (":checked:pressed", "#3079ED", "#357AE8"),
                (":focus-visible", "#1155CC", "#FFFFFF"),
                (":checked:focus-visible", "#1155CC", "#4D90FE"),
            ] {
                theme.children().add(
                    style(&format!("^{state} /template/ Border#SettingsCheckOutline"))
                        .set(Border::border_brush_property(), edge)
                        .set(Border::background_property(), fill)
                        .build(),
                );
            }
            theme.children().add(
                style("^ /template/ PathIcon#SettingsCheckMark")
                    .set(Visual::is_visible_property(), false)
                    .build(),
            );
            theme.children().add(
                style("^:checked /template/ PathIcon#SettingsCheckMark")
                    .set(Visual::is_visible_property(), true)
                    .build(),
            );
            theme.children().add(
                style("^")
                    .set(TemplatedControl::background_property(), "Transparent")
                    .build(),
            );
            theme.children().add(
                style("^:disabled")
                    .set(Visual::opacity_property(), 0.45)
                    .build(),
            );
            theme
        })
        .clone()
    })
}
