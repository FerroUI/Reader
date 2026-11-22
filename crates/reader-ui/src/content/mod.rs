mod html;
mod parser;
use ferroui_base::layout::HorizontalAlignment as H;
use ferroui_base::media::{FontFamily, FontStyle, FontWeight, TextWrapping};
use ferroui_controls::documents::{Hyperlink, InlineCollection, Run};
use ferroui_view::prelude::*;
use reader_domain::Article;
use reader_state::{AppViewModel, HostRequest};
use std::{
    hash::{Hash, Hasher},
    rc::Rc,
};

pub fn view(vm: Rc<AppViewModel>, article: &Article, size: f64) -> Element {
    let mut hash = std::collections::hash_map::DefaultHasher::new();
    article.content.hash(&mut hash);
    article.content_base_url().hash(&mut hash);
    let content_hash = hash.finish();
    let blocks = parser::parse(&article.content, article.content_base_url());
    let mut body = stack_panel()
        .max_width(650.0)
        .horizontal_alignment(H::Left)
        .spacing(14.0);
    if blocks.is_empty() {
        body = body.child(reader_controls::paragraph(article.summary.clone()));
    }
    for (i, block) in blocks.into_iter().enumerate() {
        if let Some((url, alt)) = block.image {
            body = body.child(crate::images::view(vm.clone(), url, alt));
            continue;
        }
        let model = vm.clone();
        let heading = block.heading;
        let quote = block.quote;
        let pre = block.pre;
        let spans = block.spans;
        let text = selectable_text_block()
            .key(format!("{}-{content_hash}-{i}", article.id))
            .font_size(if heading > 0 {
                (24.0 - f64::from(heading) * 2.0).max(size)
            } else {
                size
            })
            .font_weight(if heading > 0 {
                FontWeight::Bold
            } else {
                FontWeight::Normal
            })
            .foreground("#333333")
            .text_wrapping(TextWrapping::Wrap)
            .line_height(size * 1.5)
            .max_width(650.0)
            .with(move |text| {
                let inlines = InlineCollection::new();
                text.set_inlines(Some(inlines.clone()));
                for segment in &spans {
                    let run = Run::with_text(Some(&segment.text));
                    if segment.bold {
                        run.set_font_weight(FontWeight::Bold);
                    }
                    if segment.italic {
                        run.set_font_style(FontStyle::Italic);
                    }
                    if segment.code || pre {
                        run.set_font_family(FontFamily::new("monospace"));
                    }
                    if let Some(url) = &segment.href {
                        let link = Hyperlink::new();
                        link.set_navigate_uri(Some(url.clone()));
                        link.inlines().add(run);
                        let url = url.clone();
                        let model = model.clone();
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
            });
        if quote {
            body = body.child(
                border()
                    .border_brush("#CCCCCC")
                    .border_thickness((3.0, 0.0, 0.0, 0.0))
                    .padding((14.0, 0.0))
                    .child(text),
            );
        } else if pre {
            body = body.child(border().background("#F5F5F5").padding(12.0).child(text));
        } else {
            body = body.child(text);
        }
    }
    if !article.note.is_empty() {
        body = body.child(border().background("#FFF9DC").padding(12.0).child(
            reader_controls::paragraph(format!("Your note: {}", article.note)),
        ));
    }
    body.into()
}
