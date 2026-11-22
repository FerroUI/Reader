pub(super) fn tag_end(text: &str) -> Option<usize> {
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
pub(super) fn attr(tag: &str, wanted: &str) -> Option<String> {
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

/// Decode character references without changing indentation or line endings.
pub(super) fn text(raw: &str, preserve: bool) -> String {
    if !preserve {
        return reader_domain::plain_text(raw);
    }
    let mut result = String::with_capacity(raw.len());
    let mut remaining = raw;
    while let Some(index) = remaining.find('&') {
        result.push_str(&remaining[..index]);
        remaining = &remaining[index..];
        if let Some(end) = remaining.find(';').filter(|end| *end <= 16) {
            let entity = &remaining[..=end];
            if entity == "&nbsp;" {
                result.push('\u{a0}');
            } else if let Some(character) = entity
                .strip_prefix("&#")
                .and_then(|entity| entity.strip_suffix(';'))
                .and_then(|number| {
                    number
                        .strip_prefix('x')
                        .or_else(|| number.strip_prefix('X'))
                        .and_then(|number| u32::from_str_radix(number, 16).ok())
                        .or_else(|| number.parse::<u32>().ok())
                })
                .and_then(char::from_u32)
            {
                result.push(character);
            } else {
                let decoded = reader_domain::plain_text(entity);
                if decoded.is_empty() {
                    result.push(' ');
                } else {
                    result.push_str(&decoded);
                }
            }
            remaining = &remaining[end + 1..];
        } else {
            result.push('&');
            remaining = &remaining[1..];
        }
    }
    result.push_str(remaining);
    result
}
