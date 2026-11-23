use ferroui_base::data::TemplateBinding;
use ferroui_base::media::TextWrapping;
use ferroui_base::Ref;
use ferroui_controls::presenters::ContentPresenter;
use ferroui_controls::primitives::TemplatedControl;
use ferroui_controls::ContentControl;
pub(super) fn build() -> Ref<ContentPresenter> {
    let presenter = ContentPresenter::new();
    presenter.set_name(Some("PART_ContentPresenter".into()));
    presenter.set_text_wrapping(TextWrapping::Wrap);
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
    presenter
}
