use dom_smoothie::{Config, Readability};

pub const MAX_DOCUMENT_BYTES: usize = 4 * 1024 * 1024;
const MAX_ELEMENTS: usize = 50_000;

/// Owned strings can cross native worker threads and browser message boundaries.
#[derive(Debug)]
pub struct ExtractedArticle {
    pub html: String,
    pub base_url: String,
}

/// No network, script execution, cookies, or page embedding. The resulting HTML
/// is projected into Reader controls; it is never assigned to the app's DOM.
pub fn extract(html: &str, document_url: &str) -> Result<ExtractedArticle, String> {
    let valid_url = reader_domain::subscription_url(document_url)
        .ok_or("The publisher returned an invalid article address.")?;
    if html.len() > MAX_DOCUMENT_BYTES {
        return Err("The publisher page exceeds the 4 MB reading limit.".into());
    }
    if html.trim().is_empty() || !html.contains('<') || html.contains('\0') {
        return Err(
            "The publisher did not return an HTML article. Use View original to open it.".into(),
        );
    }
    // Bound allocations before constructing the HTML5 DOM, then enforce the
    // actual element count in Readability as well.
    if html.bytes().filter(|byte| *byte == b'<').count() > MAX_ELEMENTS * 2 {
        return Err(
            "This page is too complex for reading mode. Use View original to open it.".into(),
        );
    }
    let config = Config {
        max_elements_to_parse: MAX_ELEMENTS,
        char_threshold: 140,
        ..Config::default()
    };
    let mut reader = Readability::new(html, Some(&valid_url), Some(config))
        .map_err(|_| "The publisher page could not be parsed.")?;
    let title = reader.doc.select("title").text().to_lowercase();
    if [
        "just a moment",
        "access denied",
        "attention required",
        "verify you are human",
        "robot check",
    ]
    .iter()
    .any(|blocked| title.trim().starts_with(blocked))
    {
        return Err("The publisher requires a browser verification or sign-in. Use View original to continue.".into());
    }
    let base_url = reader
        .doc
        .select("base[href]")
        .attr("href")
        .and_then(|base| reader_domain::safe_url(&base, Some(&valid_url)))
        .filter(|base| reader_domain::subscription_url(base).is_some())
        .unwrap_or(valid_url);
    reader.doc_url = Some(base_url.clone());
    reader
        .doc
        .select("script,style,iframe,object,embed,form,input,button,select,textarea,dialog")
        .remove();
    let article = reader.parse().map_err(|_| {
        "No readable article was found. The publisher may require JavaScript or a subscription; use View original.".to_owned()
    })?;
    if article.text_content.trim().chars().count() < 80 {
        return Err(
            "This page contains too little article text. Use View original to read it.".into(),
        );
    }
    let html = article.content.to_string();
    if html.len() > reader_domain::MAX_ARTICLE_HTML_BYTES {
        return Err("This article exceeds the 2 MB reading limit.".into());
    }
    Ok(ExtractedArticle { html, base_url })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn page() -> String {
        let paragraph = "A carefully written technical article explains the behavior in enough detail to be read independently. It includes useful examples, explains the design choices, and gives readers the context needed to understand the result. ";
        format!("<!doctype html><html><head><title>Reading a technical article</title><base href='/journal/'></head><body><nav>Unrelated navigation <a href='/ads'>Advertisements</a></nav><article><h1>Reading a technical article</h1><p>{}</p><h2>Implementation</h2><p>{}</p><p><a href='details'>Details</a></p><img src='diagram.png' width='600' height='320'><script>throw new Error('never executed')</script><form>Sign in <input></form></article><footer>Unrelated footer</footer></body></html>", paragraph.repeat(4), paragraph.repeat(5))
    }

    #[test]
    fn extracts_article_resolves_links_and_removes_active_content() {
        let article = extract(&page(), "https://example.com/news/story").unwrap();
        assert_eq!(article.base_url, "https://example.com/journal/");
        assert!(article.html.contains("Implementation"));
        assert!(article.html.contains("https://example.com/journal/details"));
        assert!(article
            .html
            .contains("https://example.com/journal/diagram.png"));
        assert!(!article.html.contains("<script"));
        assert!(!article.html.contains("<form"));
        assert!(!article.html.contains("Unrelated navigation"));
    }

    #[test]
    fn html5_parser_accepts_unclosed_paragraphs() {
        let html = page().replace("</p>", "").replace("</html>", "");
        assert!(extract(&html, "https://example.com/story")
            .unwrap()
            .html
            .contains("Implementation"));
    }

    #[test]
    fn rejects_invalid_oversized_empty_and_verification_pages() {
        assert!(extract(&page(), "file:///private").is_err());
        assert!(extract(&"x".repeat(MAX_DOCUMENT_BYTES + 1), "https://example.com").is_err());
        assert!(extract("<html><body>Empty</body></html>", "https://example.com").is_err());
        assert!(extract(
            "<title>Just a moment...</title><p>Check your browser</p>",
            "https://example.com"
        )
        .unwrap_err()
        .contains("verification"));
    }
}
