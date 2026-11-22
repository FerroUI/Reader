//! Native link bounds let browser checks click shaped inline text, including
//! links surrounded by plain text, without estimating font or character widths.
use ferroui_base::Point;
use ferroui_controls::{
    documents::{Hyperlink, InlineCollection, Span},
    RichTextBlock,
};
use serde_json::{json, Value};

pub(super) fn links(view: &RichTextBlock, origin: Point) -> Vec<Value> {
    let mut result = Vec::new();
    let mut offset = 0;
    for paragraph in view.document().paragraphs() {
        collect(view, &paragraph.inlines(), origin, &mut offset, &mut result);
        offset += 1; // UTF-16 paragraph separator in RichTextDocument.
        if result.len() >= 64 {
            break;
        }
    }
    result
}

fn collect(
    view: &RichTextBlock,
    inlines: &InlineCollection,
    origin: Point,
    offset: &mut usize,
    result: &mut Vec<Value>,
) {
    for inline in inlines.snapshot().iter() {
        if result.len() >= 64 {
            return;
        }
        if let Some(link) = inline.cast::<Hyperlink>() {
            let text = link.inlines().text().unwrap_or_default();
            let end = *offset + text.encode_utf16().count();
            let bounds: Vec<_> = view
                .get_rects_for_range(*offset..end)
                .into_iter()
                .map(|rect| {
                    json!({"x":origin.x+rect.x,"y":origin.y+rect.y,
                    "width":rect.width,"height":rect.height})
                })
                .collect();
            result.push(json!({"url":link.navigate_uri(),"text":text,"bounds":bounds}));
            *offset = end;
        } else if let Some(span) = inline.cast::<Span>() {
            collect(view, &span.inlines(), origin, offset, result);
        } else {
            let mut text = String::new();
            inline.append_text(&mut text);
            *offset += text.encode_utf16().count();
        }
    }
}
