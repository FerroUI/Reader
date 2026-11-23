use url::Url;

const MAX_TEMPLATE_BYTES: usize = 8192;
const MAX_DESTINATION_BYTES: usize = 32768;

/// Reader's GET destinations used encodeURIComponent for each substitution.
/// Expanding in one pass prevents article text containing `${...}` from being
/// interpreted as another template expression.
fn expand(
    value: &str,
    mut token: impl FnMut(&str) -> Result<String, String>,
) -> Result<String, String> {
    let mut output = String::with_capacity(value.len());
    let mut remaining = value;
    while let Some(start) = remaining.find("${") {
        output.push_str(&remaining[..start]);
        let after = &remaining[start + 2..];
        let end = after
            .find('}')
            .ok_or("Close each Send to placeholder with }.")?;
        let name = &after[..end];
        if !matches!(name, "url" | "title" | "source") {
            return Err(format!(
                "Unsupported placeholder ${{{name}}}. Use ${{url}}, ${{title}} or ${{source}}."
            ));
        }
        output.push_str(&token(name)?);
        if output.len() > MAX_DESTINATION_BYTES {
            return Err("The expanded Send to address is too long.".into());
        }
        remaining = &after[end + 1..];
    }
    output.push_str(remaining);
    if output.contains(['{', '}']) {
        return Err("Use ${url}, ${title} or ${source} for Send to placeholders.".into());
    }
    if output.len() > MAX_DESTINATION_BYTES {
        return Err("The expanded Send to address is too long.".into());
    }
    Ok(output)
}

fn endpoint(value: &str) -> Result<Url, String> {
    let url = Url::parse(value)
        .map_err(|_| "Enter a complete HTTPS address for the Send to destination.")?;
    if url.scheme() != "https"
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
    {
        return Err("Use an HTTPS destination without an embedded username or password.".into());
    }
    Ok(url)
}

pub(super) fn validate(value: &str) -> Result<String, String> {
    let value = value.trim();
    if value.is_empty()
        || value.len() > MAX_TEMPLATE_BYTES
        || value.chars().any(char::is_whitespace)
        || value.chars().any(char::is_control)
    {
        return Err(
            "Enter a Send to URL template of at most 8 KB without spaces or control characters."
                .into(),
        );
    }
    let authority = value
        .split_once("://")
        .map(|(_, rest)| rest.split(['/', '?', '#']).next().unwrap_or(rest));
    if authority.is_some_and(|part| part.contains("${")) {
        return Err(
            "Keep the destination host fixed; put placeholders in its path, query or fragment."
                .into(),
        );
    }
    let mut substitutions = 0;
    let probe = expand(value, |_| {
        substitutions += 1;
        Ok("reader-value".into())
    })?;
    endpoint(&probe)?;
    if substitutions == 0 {
        return Err("Include ${url}, ${title} or ${source} in the Send to URL template.".into());
    }
    Ok(value.to_string())
}

pub(super) fn resolve(
    template: &str,
    article_url: &str,
    title: &str,
    source: &str,
) -> Result<String, String> {
    let template = validate(template)?;
    let article =
        crate::safe_url(article_url, None).ok_or("This item has no article address to send.")?;
    let expanded = expand(&template, |name| {
        encode(match name {
            "url" => &article,
            "title" => title,
            "source" => source,
            _ => unreachable!(),
        })
    })?;
    Ok(endpoint(&expanded)?.to_string())
}

fn encode(value: &str) -> Result<String, String> {
    const HEX: &[u8] = b"0123456789ABCDEF";
    let mut result = String::with_capacity(value.len().min(MAX_DESTINATION_BYTES));
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || b"-_.!~*'()".contains(&byte) {
            result.push(char::from(byte));
        } else {
            result.push('%');
            result.push(char::from(HEX[usize::from(byte >> 4)]));
            result.push(char::from(HEX[usize::from(byte & 15)]));
        }
        if result.len() > MAX_DESTINATION_BYTES {
            return Err("The expanded Send to address is too long.".into());
        }
    }
    Ok(result)
}
