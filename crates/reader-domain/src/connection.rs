//! Browser feed policy and validated user-entered connection addresses.
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum FeedConnection {
    /// Try the publisher, then a saved custom proxy or the public feed relay.
    #[default]
    Automatic,
    /// Request the publisher directly; never forward the address to a relay.
    Direct,
    /// Try the publisher, then the explicitly configured endpoint.
    Custom,
}

/// Accept the same website addresses advertised by Subscribe's example.
/// Explicit unsupported schemes and embedded credentials are never rewritten.
pub fn subscription_url(value: &str) -> Option<String> {
    let value = value.trim();
    if value.is_empty() || value.chars().any(char::is_whitespace) || value.contains('\\') {
        return None;
    }
    let explicit_scheme = value.find("://").is_some_and(|end| {
        value[..end]
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, b'+' | b'-' | b'.'))
    });
    let candidate = if value.starts_with("//") {
        format!("https:{value}")
    } else if explicit_scheme {
        value.to_owned()
    } else {
        let authority = value.split(['/', '?', '#']).next()?;
        if authority.contains('@')
            || !(authority.contains('.')
                || authority == "localhost"
                || authority.starts_with("localhost:")
                || authority.starts_with('['))
        {
            return None;
        }
        if !authority.starts_with('[') {
            if let Some((host, port)) = authority.rsplit_once(':') {
                if host.is_empty() || port.is_empty() || !port.bytes().all(|c| c.is_ascii_digit()) {
                    return None;
                }
            }
        }
        format!("https://{value}")
    };
    let mut url = Url::parse(&candidate).ok()?;
    if !matches!(url.scheme(), "http" | "https")
        || !url.has_host()
        || !url.username().is_empty()
        || url.password().is_some()
    {
        return None;
    }
    url.set_fragment(None);
    Some(url.into())
}

/// Validate a proxy template before saving it. The placeholder remains literal
/// in the document and is percent-encoded exactly once by the browser host.
pub fn validate_feed_proxy(value: &str) -> Result<String, &'static str> {
    let value = value.trim();
    if value.is_empty() {
        return Ok(String::new());
    }
    if !value.contains("{url}") {
        return Err("The feed proxy address must contain {url}.");
    }
    let probe = value.replace("{url}", "https%3A%2F%2Fexample.com%2Ffeed.xml");
    let url = Url::parse(&probe).map_err(|_| "Enter a valid HTTP or HTTPS feed proxy URL.")?;
    if !matches!(url.scheme(), "http" | "https")
        || !url.has_host()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.fragment().is_some()
        || value.chars().any(char::is_whitespace)
        || value.contains('\\')
    {
        return Err("Use an HTTP or HTTPS proxy URL without embedded credentials or a fragment.");
    }
    Ok(value.to_owned())
}
