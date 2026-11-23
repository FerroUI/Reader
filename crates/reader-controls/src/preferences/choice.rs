//! Native radio semantics with Reader's compact preference button appearance.
use super::presenter::build as presenter;
use ferroui_base::data::TemplateBinding;
use ferroui_base::media::{FontFamily, FontWeight};
use ferroui_base::styling::{ControlTheme, Setter};
use ferroui_base::{Ref, Thickness, Visual};
use ferroui_controls::presenters::ContentPresenter;
use ferroui_controls::primitives::TemplatedControl;
use ferroui_controls::templates::{
    FuncControlTemplate, FuncTemplateNameScopeExtensions, IControlTemplate,
};
use ferroui_controls::RadioButton;
use ferroui_view::style;
use std::{cell::OnceCell, rc::Rc};
ferroui_base::ferro_thread_local! {
    static CHOICE: OnceCell<Ref<ControlTheme>> = const { OnceCell::new() };
}
pub fn theme() -> Ref<ControlTheme> {
    CHOICE.with(|slot| {
        slot.get_or_init(|| {
            let theme = ControlTheme::for_type::<RadioButton>();
            let template: Rc<dyn IControlTemplate> = FuncControlTemplate::new(|_, scope| {
                let presenter = presenter();
                for (target, source) in [
                    (
                        ContentPresenter::background_property().as_property(),
                        TemplatedControl::background_property().as_property(),
                    ),
                    (
                        ContentPresenter::border_brush_property().as_property(),
                        TemplatedControl::border_brush_property().as_property(),
                    ),
                    (
                        ContentPresenter::border_thickness_property().as_property(),
                        TemplatedControl::border_thickness_property().as_property(),
                    ),
                    (
                        ContentPresenter::corner_radius_property().as_property(),
                        TemplatedControl::corner_radius_property().as_property(),
                    ),
                    (
                        ContentPresenter::padding_property().as_property(),
                        TemplatedControl::padding_property().as_property(),
                    ),
                ] {
                    presenter.bind_binding(target, &*TemplateBinding::new(source));
                }
                presenter.register_in_name_scope(&**scope).upcast()
            });
            theme.add_setter(Setter::new(
                TemplatedControl::template_property(),
                Some(template),
            ));
            theme.add_setter(Setter::new(
                TemplatedControl::font_family_property(),
                FontFamily::new("fonts:Reader#Liberation Sans"),
            ));
            theme.add_setter(Setter::new(TemplatedControl::font_size_property(), 11.0));
            theme.add_setter(Setter::new(
                TemplatedControl::font_weight_property(),
                FontWeight::Bold,
            ));
            theme.add_setter(Setter::new(
                TemplatedControl::padding_property(),
                Thickness::new(11.0, 5.0, 11.0, 5.0),
            ));
            theme.add_setter(Setter::new(
                TemplatedControl::border_thickness_property(),
                Thickness::uniform(1.0),
            ));
            theme.add_setter(Setter::new(
                TemplatedControl::corner_radius_property(),
                ferroui_base::CornerRadius::uniform(2.0),
            ));
            for (state, background, edge) in [
                ("", "#F5F5F5", "#DCDCDC"),
                (":pointerover", "#F8F8F8", "#C6C6C6"),
                (":checked", "#D9D9D9", "#BEBEBE"),
                (":pressed", "#E5E5E5", "#BEBEBE"),
                (":focus-visible", "#F5F5F5", "#4D90FE"),
                (":checked:focus-visible", "#D9D9D9", "#4D90FE"),
            ] {
                theme.children().add(
                    style(&format!("^{state}"))
                        .set(TemplatedControl::background_property(), background)
                        .set(TemplatedControl::border_brush_property(), edge)
                        .build(),
                );
            }
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
