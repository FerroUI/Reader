/// Only navigable network URLs cross the browser boundary.
pub fn safe_url(value: &str, base: Option<&str>) -> Option<String> {
    let value = value.trim();
    if value.is_empty() {
        return None;
    }
    let url = url::Url::parse(value).ok().or_else(|| {
        base.and_then(|base| url::Url::parse(base).ok())
            .and_then(|base| base.join(value).ok())
    })?;
    matches!(url.scheme(), "http" | "https").then(|| url.to_string())
}

/// Extract readable search/preview text without exposing markup to a renderer.
pub fn plain_text(html: &str) -> String {
    let mut output = String::with_capacity(html.len().min(4096));
    let mut chars = html.chars().peekable();
    let mut suppressed: Option<String> = None;
    while let Some(character) = chars.next() {
        if character == '<' {
            let mut tag = String::new();
            for next in chars.by_ref() {
                if next == '>' {
                    break;
                }
                if tag.len() < 512 {
                    tag.push(next);
                }
            }
            let name = tag
                .trim()
                .split_whitespace()
                .next()
                .unwrap_or("")
                .to_ascii_lowercase();
            if let Some(current) = &suppressed {
                if name == format!("/{current}") {
                    suppressed = None;
                }
            } else if matches!(name.as_str(), "script" | "style" | "iframe") {
                suppressed = Some(name);
            } else if matches!(
                name.trim_start_matches('/'),
                "p" | "div"
                    | "br"
                    | "br/"
                    | "li"
                    | "ul"
                    | "ol"
                    | "h1"
                    | "h2"
                    | "h3"
                    | "h4"
                    | "h5"
                    | "h6"
                    | "tr"
                    | "td"
                    | "th"
                    | "blockquote"
                    | "article"
                    | "section"
                    | "hr"
            ) {
                output.push(' ');
            }
        } else if suppressed.is_none() {
            if character == '&' {
                let mut entity = String::new();
                while let Some(&next) = chars.peek() {
                    if next == ';' {
                        chars.next();
                        break;
                    }
                    if !next.is_ascii_alphanumeric() && next != '#' {
                        break;
                    }
                    if entity.len() >= 12 {
                        break;
                    }
                    entity.push(chars.next().unwrap());
                }
                match entity.as_str() {
                    "amp" => output.push('&'),
                    "lt" => output.push('<'),
                    "gt" => output.push('>'),
                    "quot" => output.push('"'),
                    "apos" | "#39" => output.push('\''),
                    "nbsp" => output.push(' '),
                    "mdash" => output.push('—'),
                    "ndash" => output.push('–'),
                    "hellip" => output.push('…'),
                    "rsquo" => output.push('’'),
                    "lsquo" => output.push('‘'),
                    _ => {
                        let numeric = entity
                            .strip_prefix("#x")
                            .or_else(|| entity.strip_prefix("#X"))
                            .and_then(|s| u32::from_str_radix(s, 16).ok())
                            .or_else(|| entity.strip_prefix('#').and_then(|s| s.parse().ok()));
                        if let Some(value) = numeric.and_then(char::from_u32) {
                            output.push(value);
                        } else {
                            output.push('&');
                            output.push_str(&entity);
                        }
                    }
                }
            } else {
                output.push(character);
            }
        }
    }
    output.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub(crate) fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
