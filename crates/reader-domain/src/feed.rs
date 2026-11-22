mod content;
mod json;

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
    #[serde(default)]
    pub content_base: String,
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
        return json::parse(source, feed_url, now);
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
        rss_link(channel, feed_url)
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
            rss_link(item, feed_url)
        };
        let url = safe_url(&url, Some(feed_url)).unwrap_or_default();
        let title = plain_text(&text(item, "title"));
        let (content, content_base) = content::parse(item, source, atom, feed_url);
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
            content_base,
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
        .and_then(|n| resolve_link(n, n.attribute("href")?, base))
        .unwrap_or_default()
}

fn rss_link(node: Node<'_, '_>, base: &str) -> String {
    node.children()
        .find(|n| named(*n, "link"))
        .and_then(|n| resolve_link(n, n.text().unwrap_or_default(), base))
        .unwrap_or_default()
}

/// XML Base values compose from outermost ancestor to the element carrying the URI.
fn resolve_link(node: Node<'_, '_>, link: &str, document_url: &str) -> Option<String> {
    let base = xml_base(node, document_url)?;
    safe_url(link, Some(base.as_str()))
}

fn xml_base(node: Node<'_, '_>, document_url: &str) -> Option<url::Url> {
    let mut base = url::Url::parse(document_url).ok()?;
    let ancestors: Vec<_> = node.ancestors().filter(|n| n.is_element()).collect();
    for ancestor in ancestors.into_iter().rev() {
        if let Some(value) = ancestor.attribute(("http://www.w3.org/XML/1998/namespace", "base")) {
            base = base.join(value.trim()).ok()?;
        }
    }
    Some(base)
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
