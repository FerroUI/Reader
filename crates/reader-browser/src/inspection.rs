//! Automation snapshots reflect live native controls, including hit testing.
mod rich_text;
mod fonts;

use ferroui_base::{input::InputElement, Point, Visual};
use ferroui_controls::{
    automation::AutomationProperties, primitives::ToggleButton, Application, Border, Button,
    Control, ListBox, MenuItem, RichTextBlock, ScrollViewer, TextBlock, TextBox,
};
use serde_json::json;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name = inspectView)]
pub fn inspect_view() -> String {
    let Some(root) = Application::current()
        .and_then(|a| a.application_lifetime())
        .and_then(|l| {
            l.as_single_top_level_application_lifetime()
                .and_then(|l| l.top_level())
        })
    else {
        return "null".into();
    };
    let viewport = root.bounds();
    let visual_count = root.get_visual_descendants().count();
    let article_scroll = root
        .get_visual_descendants()
        .filter_map(|visual| visual.cast::<ListBox>())
        .find(|list| list.has_name("ArticleList"))
        .and_then(|list| list.scroll())
        .and_then(|scroll| scroll.cast::<ScrollViewer>())
        .map(|scroll| {
            json!({"offsetY":scroll.offset().y,"viewportHeight":scroll.viewport().height,
                "extentHeight":scroll.extent().height})
        });
    let article_containers = root
        .get_visual_descendants()
        .filter(|v| {
            v.name().is_some_and(|name| {
                name == "ArticleEntry" || name == "ArticleRow" || name.starts_with("Article-")
            })
        })
        .count();
    let elements = root
        .get_visual_descendants()
        .filter_map(|visual| {
            let name = visual.name();
            let article_id = name
                .as_deref()
                .filter(|name| matches!(*name, "ArticleEntry" | "ArticleRow"))
                .and_then(|_| visual.cast::<Control>())
                .and_then(|control| control.tag())
                .and_then(|tag| tag.downcast_ref::<u64>().copied());
            let text = if let Some(t) = visual.cast::<TextBlock>() {
                t.text().or_else(|| t.inlines().and_then(|inlines| inlines.text()))
            } else if let Some(t) = visual.cast::<RichTextBlock>() {
                Some(t.document().text())
            } else if let Some(t) = visual.cast::<TextBox>() {
                t.text()
            } else if let Some(item) = visual.cast::<MenuItem>() {
                item.header()
                    .and_then(|header| header.downcast_ref::<String>().cloned())
            } else if visual.cast::<Button>().is_some() || name.is_some() {
                None
            } else {
                return None;
            };
            let b = visual.bounds();
            if !visual.is_effectively_visible() || b.width <= 0.0 || b.height <= 0.0 {
                return None;
            }
            let p = visual.translate_point(Point::new(0.0, 0.0), &root)?;
            if p.x >= viewport.width
                || p.y >= viewport.height
                || p.x + b.width <= 0.0
                || p.y + b.height <= 0.0
            {
                return None;
            }
            let hit = root
                .input_hit_test_with(Point::new(p.x + b.width / 2.0, p.y + b.height / 2.0), false)
                .is_some_and(|h| {
                    let h: &Visual = &h;
                    std::ptr::eq(h, &*visual)
                        || visual.is_visual_ancestor_of(h)
                        || h.is_visual_ancestor_of(&visual)
                });
            let selection = visual.cast::<TextBox>().map(|field| json!({
                "start":field.selection_start(),"end":field.selection_end(),"caret":field.caret_index()
            })).or_else(|| visual.cast::<RichTextBlock>().map(|text| json!({
                "start":text.selection_start(),"end":text.selection_end(),"caret":text.caret_index(),
                "text":text.selected_text()
            })));
            Some(
                json!({"type":visual.get_type().name(),"name":name,"text":text,
            "foreground":visual.cast::<TextBlock>().and_then(|text|rich_text::foreground(text.foreground()))
                .or_else(||visual.cast::<RichTextBlock>().and_then(|text|rich_text::foreground(text.foreground()))),
            "fontSize":visual.cast::<TextBlock>().map(|text|text.font_size())
                .or_else(||visual.cast::<RichTextBlock>().map(|text|text.font_size())),
            "maxWidth":visual.cast::<Control>().map(|control|control.max_width())
                .filter(|width|width.is_finite()),
            "headlineRuns":visual.cast::<TextBlock>().filter(|text|text.has_name("ArticleHeadlineText"))
                .and_then(|text|text.inlines()).map(|inlines|inlines.snapshot().iter().map(|inline|{
                    let mut text = String::new();
                    inline.append_text(&mut text);
                    json!({"text":text,"weight":inline.font_weight().0,
                        "foreground":rich_text::foreground(inline.foreground())})
                }).collect::<Vec<_>>()),
            "border":visual.cast::<Border>().map(|border|{
                let thickness=border.border_thickness();
                json!({"brush":rich_text::foreground(border.border_brush()),
                    "left":thickness.left,"top":thickness.top,"right":thickness.right,"bottom":thickness.bottom})
            }),
            "background":visual.cast::<Border>().and_then(|border|rich_text::foreground(border.background())),
            "opacity":visual.opacity(),
            "automationName":AutomationProperties::get_name(&visual),"articleId":article_id,
            "treeCursor":visual.classes().contains("keyboard-cursor"),
            "checked":visual.cast::<ToggleButton>().and_then(|toggle|toggle.is_checked())
                .or_else(||visual.cast::<MenuItem>().map(|item|item.is_checked())),
            "selected":visual.cast::<MenuItem>().map(|item|item.is_selected()),
            "enabled":visual.cast::<InputElement>().is_some_and(|input|input.is_effectively_enabled()),
            "focused":visual.cast::<InputElement>().is_some_and(|i|i.is_focused()),
            "scroll":visual.cast::<ScrollViewer>().map(|scroll|json!({
                "offsetX":scroll.offset().x,"offsetY":scroll.offset().y,
                "viewportWidth":scroll.viewport().width,"viewportHeight":scroll.viewport().height,
                "extentWidth":scroll.extent().width,"extentHeight":scroll.extent().height})),
            "textSelection":selection,
            "richText":visual.cast::<RichTextBlock>().map(|text|rich_text::identity(&text)),
            "links":visual.cast::<RichTextBlock>().map(|text|rich_text::links(&text,p)),
            "x":p.x,"y":p.y,"width":b.width,"height":b.height,"hit":hit}),
            )
        })
        .collect::<Vec<_>>();
    json!({"width":viewport.width,"height":viewport.height,"visualCount":visual_count,
        "articleContainers":article_containers,"articleScroll":article_scroll,"elements":elements})
    .to_string()
}
