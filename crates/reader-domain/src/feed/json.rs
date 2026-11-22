use super::{parse_date, ParsedArticle, ParsedFeed};
use crate::{plain_text, safe_url};

pub(super) fn parse(source: &str, feed_url: &str, now: i64) -> Result<ParsedFeed, String> {
    let data: serde_json::Value = serde_json::from_str(source).map_err(|e| e.to_string())?;
    if !data["version"]
        .as_str()
        .unwrap_or_default()
        .starts_with("https://jsonfeed.org/version/")
    {
        return Err("This JSON document is not a JSON Feed.".into());
    }
    let string =
        |value: &serde_json::Value, key: &str| value[key].as_str().unwrap_or_default().to_string();
    let mut feed = ParsedFeed {
        title: string(&data, "title"),
        site_url: safe_url(&string(&data, "home_page_url"), Some(feed_url)).unwrap_or_default(),
        description: string(&data, "description"),
        articles: Vec::new(),
    };
    for item in data["items"].as_array().into_iter().flatten().take(5000) {
        let html = string(item, "content_html");
        let content = if html.is_empty() {
            crate::html::escape(&string(item, "content_text"))
        } else {
            html
        };
        let url = safe_url(&string(item, "url"), Some(feed_url)).unwrap_or_default();
        let date = parse_date(&string(item, "date_published")).unwrap_or(now);
        let title = string(item, "title");
        let guid = string(item, "id");
        feed.articles.push(ParsedArticle {
            guid: if guid.is_empty() {
                format!("{url}:{title}:{date}")
            } else {
                guid
            },
            title: if title.is_empty() {
                "(untitled)".into()
            } else {
                title
            },
            url,
            author: item["authors"]
                .as_array()
                .and_then(|a| a.first())
                .map(|a| string(a, "name"))
                .unwrap_or_else(|| string(&item["author"], "name")),
            summary: plain_text(&content).chars().take(400).collect(),
            content,
            content_base: feed_url.to_string(),
            published: date,
        });
    }
    Ok(feed)
}
