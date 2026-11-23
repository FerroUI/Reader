use ferroui_base::media::{
    immutable::ImmutableSolidColorBrush, Color, FontFamily, FontStyle, FontWeight, IBrush,
    TextWrapping,
};
use ferroui_controls::documents::{Hyperlink, Paragraph, Run, TextElement};
use ferroui_view::prelude::*;
use reader_content::projection::Span;
use reader_state::{AppViewModel, HostRequest};
use std::rc::Rc;

pub(super) fn view(
    vm: Rc<AppViewModel>,
    spans: &[Span],
    size: f64,
    heading: u8,
    pre: bool,
    subdued: bool,
    key: String,
) -> Element {
    let spans = spans.to_vec();
    // RichTextBlock supplies native inline hit testing, hyperlink focus and
    // activation alongside selection, copying, and context-menu behavior.
    rich_text_block()
        .key(key)
        .name("ArticleBodyText")
        .font_size(size)
        .font_weight(if heading > 0 {
            FontWeight::Bold
        } else {
            FontWeight::Normal
        })
        .foreground(if subdued { "#222222" } else { "#000000" })
        .text_wrapping(if pre {
            TextWrapping::NoWrap
        } else {
            TextWrapping::Wrap
        })
        .line_height(size * if pre { 1.4 } else { 1.5 })
        .with(move |text| {
            let paragraph = Paragraph::new();
            let inlines = paragraph.inlines();
            let active: Rc<dyn IBrush> =
                Rc::new(ImmutableSolidColorBrush::new(Color::from_rgb(17, 85, 204)));
            let inactive: Rc<dyn IBrush> =
                Rc::new(ImmutableSolidColorBrush::new(Color::from_rgb(68, 68, 68)));
            let foreground = if subdued { &inactive } else { &active };
            let mut links = Vec::new();
            for segment in &spans {
                let run = Run::with_text(Some(&segment.text));
                if segment.bold {
                    run.set_font_weight(FontWeight::Bold);
                }
                if segment.italic {
                    run.set_font_style(FontStyle::Italic);
                }
                if segment.code || pre {
                    run.set_font_family(FontFamily::new("fonts:Reader#Liberation Mono"));
                }
                if let Some(url) = &segment.href {
                    let link = Hyperlink::new();
                    link.set_navigate_uri(Some(url.clone()));
                    link.set_foreground(Some(foreground.clone()));
                    link.inlines().add(run);
                    links.push(link.downgrade());
                    let url = url.clone();
                    let model = vm.clone();
                    let token = link.navigation_requested(move |event| {
                        event.set_handled(true);
                        model.request(HostRequest::OpenUrl { url: url.clone() });
                    });
                    on_cleanup(move || token.dispose());
                    inlines.add(link);
                } else {
                    inlines.add(run);
                }
            }
            text.blocks().add(paragraph);
            if !links.is_empty() {
                // `.with` mounts once, whereas read/current changes reconcile
                // Foreground in place. Update the existing links, keeping the
                // document, inline focus, and native text selection intact.
                let token = text.property_changed(move |change| {
                    if change.property() != TextElement::foreground_property().as_property() {
                        return;
                    }
                    let subdued = change
                        .new_value()
                        .downcast_ref::<Option<Rc<dyn IBrush>>>()
                        .and_then(Option::as_ref)
                        .and_then(|brush| brush.as_solid_color_brush())
                        .is_some_and(|brush| brush.color() == Color::from_rgb(34, 34, 34));
                    let foreground = if subdued { &inactive } else { &active };
                    for link in links.iter().filter_map(|link| link.upgrade()) {
                        link.set_foreground(Some(foreground.clone()));
                    }
                });
                on_cleanup(move || token.dispose());
            }
        })
        .into()
}
