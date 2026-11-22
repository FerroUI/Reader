//! Extract feed autodiscovery links without executing or rendering publisher HTML.
use std::collections::BTreeMap;

pub fn feed_link(html: &str, page_url: &str) -> Option<String> {
    let mut rest = html.get(..html.len().min(256 * 1024)).unwrap_or(html);
    let mut base = page_url.to_owned();
    while let Some(start) = rest.find('<') {
        rest = &rest[start + 1..];
        if rest.starts_with("!--") {
            rest = rest.split_once("-->")?.1;
            continue;
        }
        let mut quote = None;
        let end = rest.char_indices().find_map(|(i, c)| {
            if let Some(q) = quote {
                if c == q {
                    quote = None;
                }
            } else if c == '\'' || c == '"' {
                quote = Some(c);
            } else if c == '>' {
                return Some(i);
            }
            None
        })?;
        let tag = &rest[..end];
        rest = &rest[end + 1..];
        let name_end = tag
            .find(|c: char| c.is_whitespace() || c == '/')
            .unwrap_or(tag.len());
        let name = &tag[..name_end];
        if name.eq_ignore_ascii_case("script") || name.eq_ignore_ascii_case("style") {
            let lower = rest.to_ascii_lowercase();
            let close = lower.find(&format!("</{}", name.to_ascii_lowercase()))?;
            rest = &rest[close..];
            continue;
        }
        if !name.eq_ignore_ascii_case("link") && !name.eq_ignore_ascii_case("base") {
            continue;
        }
        let attributes = attributes(&tag[name_end..]);
        let Some(href) = attributes.get("href") else {
            continue;
        };
        let url = reader_domain::safe_url(&reader_domain::plain_text(href), Some(&base));
        if name.eq_ignore_ascii_case("base") {
            if let Some(url) = url {
                base = url;
            }
            continue;
        }
        let alternate = attributes.get("rel").is_some_and(|rel| {
            rel.split_whitespace()
                .any(|s| s.eq_ignore_ascii_case("alternate"))
        });
        let mime = attributes
            .get("type")
            .map(|s| s.to_ascii_lowercase())
            .unwrap_or_default();
        if alternate
            && matches!(
                mime.split(';').next().unwrap_or("").trim(),
                "application/rss+xml"
                    | "application/atom+xml"
                    | "application/feed+json"
                    | "application/json"
            )
        {
            if url.is_some() {
                return url;
            }
        }
    }
    None
}

fn attributes(mut source: &str) -> BTreeMap<String, String> {
    let mut values = BTreeMap::new();
    while !source.trim_start().is_empty() {
        source = source.trim_start();
        let end = source
            .find(|c: char| c.is_whitespace() || c == '=' || c == '/')
            .unwrap_or(source.len());
        if end == 0 {
            source = &source[1..];
            continue;
        }
        let name = source[..end].to_ascii_lowercase();
        source = source[end..].trim_start();
        if !source.starts_with('=') {
            continue;
        }
        source = source[1..].trim_start();
        let Some(first) = source.chars().next() else {
            break;
        };
        let value;
        if first == '\'' || first == '"' {
            source = &source[1..];
            let end = source.find(first).unwrap_or(source.len());
            value = source[..end].to_owned();
            source = &source[(end + 1).min(source.len())..];
        } else {
            let end = source.find(char::is_whitespace).unwrap_or(source.len());
            value = source[..end].to_owned();
            source = &source[end..];
        }
        values.insert(name, value);
    }
    values
}
