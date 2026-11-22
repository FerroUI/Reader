//! Small streaming helpers for already downloaded HTML. No browser DOM needed.
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
    let bytes = tag.as_bytes();
    let mut at = tag.find(char::is_whitespace).unwrap_or(tag.len());
    while at < bytes.len() {
        while bytes
            .get(at)
            .is_some_and(|b| b.is_ascii_whitespace() || *b == b'/')
        {
            at += 1;
        }
        let start = at;
        while bytes
            .get(at)
            .is_some_and(|b| !b.is_ascii_whitespace() && !matches!(*b, b'=' | b'/' | b'>'))
        {
            at += 1;
        }
        if start == at {
            at += 1;
            continue;
        }
        let name = &tag[start..at];
        while bytes.get(at).is_some_and(u8::is_ascii_whitespace) {
            at += 1;
        }
        if bytes.get(at) != Some(&b'=') {
            if name.eq_ignore_ascii_case(wanted) {
                return Some(String::new());
            }
            continue;
        }
        at += 1;
        while bytes.get(at).is_some_and(u8::is_ascii_whitespace) {
            at += 1;
        }
        let quote = bytes.get(at).copied().filter(|b| matches!(b, b'\'' | b'"'));
        if quote.is_some() {
            at += 1;
        }
        let start = at;
        while bytes.get(at).is_some_and(|b| {
            if let Some(q) = quote {
                *b != q
            } else {
                !b.is_ascii_whitespace() && *b != b'>'
            }
        }) {
            at += 1;
        }
        if name.eq_ignore_ascii_case(wanted) {
            return Some(decode(&tag[start..at]));
        }
        if quote.is_some() && at < bytes.len() {
            at += 1;
        }
    }
    None
}

/// Decode the HTML5 entity table, including two-codepoint mathematical symbols.
/// Search only one short reference at a time so bare '&' runs stay linear.
pub(super) fn decode(raw: &str) -> String {
    let mut output = String::with_capacity(raw.len());
    let mut rest = raw;
    while let Some(at) = rest.find('&') {
        output.push_str(&rest[..at]);
        rest = &rest[at..];
        let end = rest
            .as_bytes()
            .iter()
            .enumerate()
            .skip(1)
            .take(32)
            .find(|(_, byte)| matches!(**byte, b';' | b'&') || byte.is_ascii_whitespace())
            .map(|(at, _)| at)
            .filter(|at| rest.as_bytes()[*at] == b';');
        let Some(end) = end else {
            output.push('&');
            rest = &rest[1..];
            continue;
        };
        if let Some(&(first, second)) = web_atoms::NAMED_ENTITIES
            .get(&rest[1..=end])
            .filter(|(first, _)| *first != 0)
        {
            output.push(char::from_u32(first).unwrap_or(char::REPLACEMENT_CHARACTER));
            if second != 0 {
                output.push(char::from_u32(second).unwrap_or(char::REPLACEMENT_CHARACTER));
            }
        } else if let Some(character) = numeric_reference(&rest[1..end]) {
            output.push(character);
        } else {
            output.push_str(&rest[..=end]);
        }
        rest = &rest[end + 1..];
    }
    output.push_str(rest);
    output
}

fn numeric_reference(entity: &str) -> Option<char> {
    let number = entity.strip_prefix('#')?;
    let (digits, radix) = number
        .strip_prefix('x')
        .or_else(|| number.strip_prefix('X'))
        .map_or((number, 10), |hex| (hex, 16));
    if digits.is_empty() || !digits.chars().all(|c| c.is_digit(radix)) {
        return None;
    }
    let mut number = u32::from_str_radix(digits, radix).unwrap_or(0xfffd);
    // HTML retains the legacy Windows-1252 numeric-reference interpretation.
    const C1: [u32; 32] = [
        0x20ac, 0x81, 0x201a, 0x192, 0x201e, 0x2026, 0x2020, 0x2021, 0x2c6, 0x2030, 0x160, 0x2039,
        0x152, 0x8d, 0x17d, 0x8f, 0x90, 0x2018, 0x2019, 0x201c, 0x201d, 0x2022, 0x2013, 0x2014,
        0x2dc, 0x2122, 0x161, 0x203a, 0x153, 0x9d, 0x17e, 0x178,
    ];
    if (0x80..=0x9f).contains(&number) {
        number = C1[(number - 0x80) as usize];
    }
    Some(
        char::from_u32(if number == 0 { 0xfffd } else { number })
            .unwrap_or(char::REPLACEMENT_CHARACTER),
    )
}

pub(super) fn dimension(tag: &str, name: &str) -> Option<f64> {
    attr(tag, name)?
        .trim()
        .trim_end_matches("px")
        .parse::<f64>()
        .ok()
        .filter(|n| n.is_finite() && *n > 0.0 && *n <= 16384.0)
}

pub(super) fn skip_element(text: &str, name: &str) -> usize {
    // Raw-text elements may contain '<' comparisons or quoted pseudo-tags.
    // Only an actual closing tag terminates them, as in the HTML tokenizer.
    if matches!(name, "script" | "style" | "iframe") {
        let closing = format!("</{name}");
        for (at, candidate) in text.as_bytes().windows(closing.len()).enumerate() {
            if candidate.eq_ignore_ascii_case(closing.as_bytes())
                && text
                    .as_bytes()
                    .get(at + closing.len())
                    .is_some_and(|c| c.is_ascii_whitespace() || matches!(*c, b'>' | b'/'))
            {
                if let Some(end) = tag_end(&text[at..]) {
                    return at + end + 1;
                }
            }
        }
        return text.len();
    }
    let mut at = 0;
    let mut depth = 1usize;
    while let Some(start) = text[at..].find('<') {
        at += start;
        let Some(end) = tag_end(&text[at..]) else {
            break;
        };
        let raw = text[at + 1..at + end].trim();
        let closing = raw.starts_with('/');
        let tag = raw
            .trim_start_matches('/')
            .split(|c: char| c.is_ascii_whitespace() || c == '/')
            .next()
            .unwrap_or("");
        at += end + 1;
        if tag.eq_ignore_ascii_case(name) {
            if closing {
                depth -= 1;
                if depth == 0 {
                    return at;
                }
            } else if !raw.ends_with('/') {
                depth += 1;
            }
        }
    }
    text.len()
}
