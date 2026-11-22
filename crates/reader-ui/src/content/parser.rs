//! A non-executing HTML projection. It preserves text, formatting, links and images;
//! scripts, embeds and remote style sheets never become controls or browser DOM.
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
                "p" | "div" | "section" | "article" | "hr" | "ul" | "ol" | "li" | "blockquote"
                | "pre" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                    flush(&mut block, &mut output);
                    if !closing {
                        if name.starts_with('h') && name.len() == 2 {
                            block.heading = name.as_bytes()[1].saturating_sub(b'0');
                        }
                        block.quote = name == "blockquote";
                        block.pre = name == "pre";
                        if name == "li" {
                            block.spans.push(Span {
                                text: "•  ".into(),
                                ..Default::default()
                            });
                        }
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
            let decoded = reader_domain::plain_text(raw);
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
            if raw.starts_with(char::is_whitespace) {
                text.push(' ');
            }
            text.push_str(&decoded);
            if raw.ends_with(char::is_whitespace) {
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
fn tag_end(text: &str) -> Option<usize> {
    let mut quote = None;
    for (i, c) in text.char_indices().skip(1) {
        if let Some(q) = quote {
            if c == q {
                quote = None;
            }
        } else if c == '\'' || c == '"' {
            quote = Some(c);
        } else if c == '>' {
            return Some(i);
        }
    }
    None
}
fn attr(tag: &str, wanted: &str) -> Option<String> {
    let mut chars = tag.char_indices().peekable();
    while let Some((_, c)) = chars.peek() {
        if c.is_ascii_whitespace() {
            break;
        }
        chars.next();
    }
    while let Some((start, c)) = chars.next() {
        if c.is_ascii_whitespace() || c == '/' {
            continue;
        }
        let mut end = start + c.len_utf8();
        while let Some((i, c)) = chars.peek() {
            if *c == '=' || c.is_ascii_whitespace() {
                break;
            }
            end = *i + c.len_utf8();
            chars.next();
        }
        let name = &tag[start..end];
        while chars.peek().is_some_and(|(_, c)| c.is_ascii_whitespace()) {
            chars.next();
        }
        if !chars.peek().is_some_and(|(_, c)| *c == '=') {
            continue;
        }
        chars.next();
        while chars.peek().is_some_and(|(_, c)| c.is_ascii_whitespace()) {
            chars.next();
        }
        let Some((start, first)) = chars.next() else {
            break;
        };
        let (value_start, mut value_end) = if first == '\'' || first == '"' {
            (start + 1, start + 1)
        } else {
            (start, start + first.len_utf8())
        };
        while let Some((i, c)) = chars.peek() {
            if (first == '\'' || first == '"') && *c == first {
                chars.next();
                break;
            }
            if first != '\'' && first != '"' && c.is_ascii_whitespace() {
                break;
            }
            value_end = *i + c.len_utf8();
            chars.next();
        }
        if name.eq_ignore_ascii_case(wanted) {
            return Some(reader_domain::plain_text(&tag[value_start..value_end]));
        }
    }
    None
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn projects_safe_content_and_formatting() {
        let blocks=parse("<p>Hello <strong>world</strong> <a href='/more'>read more</a></p><script>bad()</script><img src='/photo.jpg' alt='Photo'>","https://example.org/feed");
        assert_eq!(blocks.len(), 2);
        assert!(blocks[0].spans.iter().any(|s| s.bold && s.text == "world"));
        assert!(blocks[0]
            .spans
            .iter()
            .any(|s| s.href.as_deref() == Some("https://example.org/more")));
        assert_eq!(
            blocks[1].image.as_ref().unwrap().0,
            "https://example.org/photo.jpg"
        );
    }
    #[test]
    fn discards_active_urls() {
        let blocks = parse(
            "<p><a href='javascript:bad()'>safe text</a></p><img src='data:x'>",
            "https://example.org",
        );
        assert_eq!(blocks.len(), 1);
        assert!(blocks[0].spans.iter().all(|s| s.href.is_none()));
    }
}
