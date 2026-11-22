use crate::{plain_text, safe_url};
use roxmltree::{Document, Node};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ParsedArticle {
    pub guid: String,
    pub title: String,
    pub url: String,
    pub author: String,
    pub content: String,
    pub summary: String,
    pub published: i64,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ParsedFeed {
    pub title: String,
    pub site_url: String,
    pub description: String,
    pub articles: Vec<ParsedArticle>,
}

pub fn parse_feed(source: &str, feed_url: &str, now: i64) -> Result<ParsedFeed, String> {
    if source.len() > 8 * 1024 * 1024 {
        return Err("The feed exceeds the 8 MB import limit.".into());
    }
    if safe_url(feed_url, None).is_none() {
        return Err("Enter an http or https feed URL.".into());
    }
    if source.trim_start().starts_with('{') {
        return parse_json_feed(source, feed_url, now);
    }
    let document =
        Document::parse(source).map_err(|error| format!("The feed is not valid XML: {error}"))?;
    let root = document.root_element();
    let atom = root.tag_name().name() == "feed";
    let channel = if atom {
        root
    } else {
        root.children()
            .find(|n| named(*n, "channel"))
            .unwrap_or(root)
    };
    if !atom && !matches!(root.tag_name().name(), "rss" | "RDF") {
        return Err("This address returned a web page, not an RSS or Atom feed.".into());
    }
    let site_url = if atom {
        atom_link(channel, feed_url)
    } else {
        text(channel, "link")
    };
    let mut feed = ParsedFeed {
        title: plain_text(&text(channel, "title")),
        site_url: safe_url(&site_url, Some(feed_url)).unwrap_or_default(),
        description: plain_text(&text(
            channel,
            if atom { "subtitle" } else { "description" },
        )),
        articles: Vec::new(),
    };
    let items = if atom || root.tag_name().name() == "rss" {
        channel
    } else {
        root
    };
    for item in items
        .children()
        .filter(|n| named(*n, if atom { "entry" } else { "item" }))
        .take(5000)
    {
        let url = if atom {
            atom_link(item, feed_url)
        } else {
            text(item, "link")
        };
        let url = safe_url(&url, Some(feed_url)).unwrap_or_default();
        let title = plain_text(&text(item, "title"));
        let content = if atom {
            atom_content(item, source).unwrap_or_else(|| text(item, "summary"))
        } else {
            let encoded = text(item, "encoded");
            if encoded.is_empty() {
                text(item, "description")
            } else {
                encoded
            }
        };
        let summary = plain_text(&content).chars().take(400).collect();
        let author = if atom {
            item.children()
                .find(|n| named(*n, "author"))
                .map(|n| text(n, "name"))
                .unwrap_or_default()
        } else {
            let author = text(item, "creator");
            if author.is_empty() {
                text(item, "author")
            } else {
                author
            }
        };
        let date = ["published", "pubDate", "updated", "date"]
            .into_iter()
            .find_map(|name| parse_date(&text(item, name)))
            .unwrap_or(now);
        let mut guid = text(item, if atom { "id" } else { "guid" });
        if guid.is_empty() {
            guid = if url.is_empty() {
                format!("{title}:{date}")
            } else {
                url.clone()
            };
        }
        feed.articles.push(ParsedArticle {
            guid,
            title: if title.is_empty() {
                "(untitled)".into()
            } else {
                title
            },
            url,
            author: plain_text(&author),
            content,
            summary,
            published: date,
        });
    }
    if feed.title.is_empty() {
        feed.title = feed_url.to_owned();
    }
    Ok(feed)
}

fn named(node: Node<'_, '_>, name: &str) -> bool {
    node.is_element() && node.tag_name().name() == name
}

fn text(node: Node<'_, '_>, name: &str) -> String {
    node.children()
        .find(|n| named(*n, name))
        .map(|n| {
            n.descendants()
                .filter(|n| n.is_text())
                .filter_map(|n| n.text())
                .collect()
        })
        .unwrap_or_default()
}

fn atom_link(node: Node<'_, '_>, base: &str) -> String {
    node.children()
        .filter(|n| named(*n, "link"))
        .find(|n| n.attribute("rel").unwrap_or("alternate") == "alternate")
        .and_then(|n| n.attribute("href"))
        .and_then(|url| safe_url(url, Some(base)))
        .unwrap_or_default()
}

fn atom_content(node: Node<'_, '_>, source: &str) -> Option<String> {
    let content = node.children().find(|n| named(*n, "content"))?;
    if content.attribute("type") == Some("xhtml") {
        let div = content.children().find(|n| n.is_element())?;
        Some(
            div.children()
                .map(|n| source.get(n.range()).unwrap_or_default())
                .collect(),
        )
    } else {
        Some(
            content
                .descendants()
                .filter(|n| n.is_text())
                .filter_map(|n| n.text())
                .collect(),
        )
    }
}

fn parse_date(value: &str) -> Option<i64> {
    chrono::DateTime::parse_from_rfc3339(value)
        .ok()
        .map(|d| d.timestamp())
        .or_else(|| {
            chrono::DateTime::parse_from_rfc2822(value)
                .ok()
                .map(|d| d.timestamp())
        })
        .or_else(|| {
            chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d")
                .ok()
                .and_then(|d| d.and_hms_opt(0, 0, 0))
                .map(|d| d.and_utc().timestamp())
        })
}

fn parse_json_feed(source: &str, feed_url: &str, now: i64) -> Result<ParsedFeed, String> {
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
            published: date,
        });
    }
    Ok(feed)
}
