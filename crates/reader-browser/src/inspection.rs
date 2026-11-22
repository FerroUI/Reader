//! Automation snapshots reflect live native controls, including hit testing.
use ferroui_base::{input::InputElement, Point, Visual};
use ferroui_controls::{
    automation::AutomationProperties, Application, Button, MenuItem, TextBlock, TextBox,
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
            let text = if let Some(t) = visual.cast::<TextBlock>() {
                t.text()
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
            Some(
                json!({"type":visual.get_type().name(),"name":name,"text":text,
            "automationName":AutomationProperties::get_name(&visual),
            "focused":visual.cast::<InputElement>().is_some_and(|i|i.is_focused()),
            "x":p.x,"y":p.y,"width":b.width,"height":b.height,"hit":hit}),
            )
        })
        .collect::<Vec<_>>();
    json!({"width":viewport.width,"height":viewport.height,"visualCount":visual_count,
        "articleContainers":article_containers,"elements":elements})
    .to_string()
}
