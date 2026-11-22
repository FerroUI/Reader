use ferroui_base::media::{FontFamily, FontStyle, FontWeight, TextWrapping};
use ferroui_controls::documents::{Hyperlink, Paragraph, Run};
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
    let font_size = if heading > 0 {
        (28.0 - f64::from(heading) * 2.0).max(size)
    } else {
        size
    };
    // RichTextBlock supplies native inline hit testing, hyperlink focus and
    // activation alongside selection, copying, and context-menu behavior.
    rich_text_block()
        .key(key)
        .name("ArticleBodyText")
        .font_size(font_size)
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
        .line_height(font_size * if pre { 1.4 } else { 1.5 })
        .with(move |text| {
            let paragraph = Paragraph::new();
            let inlines = paragraph.inlines();
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
                    link.inlines().add(run);
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
        })
        .into()
}
