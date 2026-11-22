use super::{named, xml_base};
use crate::{html::escape, safe_url};
use roxmltree::Node;

pub(super) fn parse(
    item: Node<'_, '_>,
    source: &str,
    atom: bool,
    document_url: &str,
) -> (String, String) {
    let child = |name| item.children().find(|n| named(*n, name));
    let node = if atom {
        child("content")
            .filter(|n| n.attribute("src").is_none())
            .or_else(|| child("summary"))
    } else {
        child("encoded")
            .filter(|n| !text(*n).is_empty())
            .or_else(|| child("description"))
    };
    let Some(node) = node else {
        return (String::new(), base(item, document_url));
    };
    if atom && node.attribute("type") == Some("xhtml") {
        if let Some(div) = node.children().find(|n| n.is_element()) {
            let content = div
                .children()
                .map(|n| source.get(n.range()).unwrap_or_default())
                .collect();
            // The required XHTML div is stripped from the body; preserve its inherited base.
            return (content, base(div, document_url));
        }
    }
    let value = text(node);
    let content = if atom && node.attribute("type").unwrap_or("text") == "text" {
        escape(&value)
    } else {
        value
    };
    (content, base(node, document_url))
}

fn text(node: Node<'_, '_>) -> String {
    node.descendants()
        .filter(|n| n.is_text())
        .filter_map(|n| n.text())
        .collect()
}

fn base(node: Node<'_, '_>, document_url: &str) -> String {
    xml_base(node, document_url)
        .and_then(|base| safe_url(base.as_str(), None))
        .unwrap_or_default()
}
