//! A non-executing HTML projection. It preserves text, formatting, links and images;
//! scripts, embeds and remote style sheets never become controls or browser DOM.
use super::html::{attr, tag_end};
#[derive(Clone, Default)]
pub struct Span {
    pub text: String,
    pub bold: bool,
    pub italic: bool,
    pub code: bool,
    pub href: Option<String>,
}
#[derive(Clone, Default)]
pub struct Block {
    pub spans: Vec<Span>,
    pub heading: u8,
    pub quote: bool,
    pub pre: bool,
    pub image: Option<(String, String)>,
}
fn flush(block: &mut Block, output: &mut Vec<Block>) {
    if !block.spans.is_empty() || block.image.is_some() {
        output.push(std::mem::take(block));
    }
}
pub fn parse(html: &str, base: &str) -> Vec<Block> {
    let mut output = Vec::new();
    let mut block = Block::default();
    let mut pos = 0;
    let mut bold = 0u32;
    let mut italic = 0u32;
    let mut code = 0u32;
    let mut href = None;
    let mut lists: Vec<Option<i64>> = Vec::new();
    let mut pre_depth = 0u32;
    let mut quote_depth = 0u32;
    let mut suppressed = String::new();
    while pos < html.len() {
        let rest = &html[pos..];
        if rest.starts_with('<') {
            let Some(end) = tag_end(rest) else { break };
            let raw = &rest[1..end];
            pos += end + 1;
            let closing = raw.trim_start().starts_with('/');
            let name = raw
                .trim()
                .trim_start_matches('/')
                .split(|c: char| c.is_ascii_whitespace() || c == '/')
                .next()
                .unwrap_or("")
                .to_ascii_lowercase();
            if !suppressed.is_empty() {
                if closing && name == suppressed {
                    suppressed.clear();
                }
                continue;
            }
            if matches!(
                name.as_str(),
                "script" | "style" | "iframe" | "object" | "svg" | "form"
            ) && !closing
            {
                suppressed = name;
                continue;
            }
            match name.as_str() {
                "b" | "strong" => {
                    bold = if closing {
                        bold.saturating_sub(1)
                    } else {
                        bold + 1
                    }
                }
                "i" | "em" => {
                    italic = if closing {
                        italic.saturating_sub(1)
                    } else {
                        italic + 1
                    }
                }
                "code" | "tt" => {
                    code = if closing {
                        code.saturating_sub(1)
                    } else {
                        code + 1
                    }
                }
                "a" => {
                    href = if closing {
                        None
                    } else {
                        attr(raw, "href").and_then(|v| reader_domain::safe_url(&v, Some(base)))
                    }
                }
                "br" => block.spans.push(Span {
                    text: "\n".into(),
                    ..Default::default()
                }),
                "img" if !closing => {
                    if let Some(url) =
                        attr(raw, "src").and_then(|v| reader_domain::safe_url(&v, Some(base)))
                    {
                        flush(&mut block, &mut output);
                        block.image = Some((url, attr(raw, "alt").unwrap_or_default()));
                        flush(&mut block, &mut output);
                    }
                }
                "ol" | "ul" => {
                    flush(&mut block, &mut output);
                    if closing {
                        lists.pop();
                    } else {
                        lists.push(if name == "ol" {
                            Some(
                                attr(raw, "start")
                                    .and_then(|value| value.parse().ok())
                                    .unwrap_or(1),
                            )
                        } else {
                            None
                        });
                    }
                }
                "li" => {
                    flush(&mut block, &mut output);
                    if !closing {
                        let marker = if let Some(Some(number)) = lists.last_mut() {
                            if let Some(value) =
                                attr(raw, "value").and_then(|value| value.parse().ok())
                            {
                                *number = value;
                            }
                            let marker = format!("{number}.  ");
                            *number = number.saturating_add(1);
                            marker
                        } else {
                            "•  ".to_owned()
                        };
                        block.spans.push(Span {
                            text: marker,
                            ..Default::default()
                        });
                    }
                }
                "blockquote" | "pre" => {
                    flush(&mut block, &mut output);
                    let depth = if name == "pre" {
                        &mut pre_depth
                    } else {
                        &mut quote_depth
                    };
                    *depth = if closing {
                        depth.saturating_sub(1)
                    } else {
                        depth.saturating_add(1)
                    };
                }
                "td" | "th" => {
                    if name == "th" {
                        bold = if closing {
                            bold.saturating_sub(1)
                        } else {
                            bold + 1
                        };
                    }
                    if closing {
                        block.spans.push(Span {
                            text: "    ".into(),
                            ..Default::default()
                        });
                    }
                }
                "p" | "div" | "section" | "article" | "hr" | "table" | "tr" | "h1" | "h2"
                | "h3" | "h4" | "h5" | "h6" => {
                    flush(&mut block, &mut output);
                    if !closing && name.starts_with('h') && name.len() == 2 {
                        block.heading = name.as_bytes()[1].saturating_sub(b'0');
                    }
                }
                _ => {}
            }
        } else {
            let end = rest.find('<').unwrap_or(rest.len());
            let raw = &rest[..end];
            pos += end;
            if !suppressed.is_empty() {
                continue;
            }
            block.pre = pre_depth > 0;
            block.quote = quote_depth > 0;
            let decoded = super::html::text(raw, block.pre);
            if decoded.is_empty() {
                if !block.spans.is_empty() {
                    block.spans.push(Span {
                        text: " ".into(),
                        ..Default::default()
                    });
                }
                continue;
            }
            let mut text = String::new();
            if !block.pre && raw.starts_with(char::is_whitespace) {
                text.push(' ');
            }
            text.push_str(&decoded);
            if !block.pre && raw.ends_with(char::is_whitespace) {
                text.push(' ');
            }
            block.spans.push(Span {
                text,
                bold: bold > 0,
                italic: italic > 0,
                code: code > 0,
                href: href.clone(),
            });
        }
    }
    flush(&mut block, &mut output);
    output
}
#[cfg(test)]
mod tests;
