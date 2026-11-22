use crate::{html::escape, safe_url, AppState};
use roxmltree::Document;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OpmlSubscription {
    pub title: String,
    pub url: String,
    pub site_url: String,
    pub folder: String,
    pub folders: Vec<String>,
}

pub fn parse_opml(source: &str) -> Result<Vec<OpmlSubscription>, String> {
    if source.len() > 8 * 1024 * 1024 {
        return Err("OPML exceeds the 8 MB import limit.".into());
    }
    let doc = Document::parse(source).map_err(|e| format!("Invalid OPML: {e}"))?;
    if doc.root_element().tag_name().name() != "opml" {
        return Err("Select an OPML subscription file.".into());
    }
    let mut subscriptions: Vec<OpmlSubscription> = Vec::new();
    let mut seen = std::collections::BTreeMap::<String, usize>::new();
    for node in doc.descendants().filter(|n| n.has_tag_name("outline")) {
        let Some(raw_url) = node
            .attribute("xmlUrl")
            .or_else(|| node.attribute("xmlurl"))
        else {
            continue;
        };
        let Some(url) = safe_url(raw_url, None) else {
            continue;
        };
        let mut folders = node
            .ancestors()
            .skip(1)
            .take_while(|n| n.has_tag_name("outline"))
            .filter_map(|n| n.attribute("text").or_else(|| n.attribute("title")))
            .filter(|s| !s.trim().is_empty())
            .map(str::to_string)
            .collect::<Vec<_>>();
        folders.reverse();
        let folder = folders.join(" / ");
        if let Some(&index) = seen.get(&url) {
            if !folder.is_empty() && !subscriptions[index].folders.contains(&folder) {
                subscriptions[index].folders.push(folder);
            }
            continue;
        }
        seen.insert(url.clone(), subscriptions.len());
        subscriptions.push(OpmlSubscription {
            title: node
                .attribute("title")
                .or_else(|| node.attribute("text"))
                .unwrap_or(&url)
                .to_owned(),
            site_url: node
                .attribute("htmlUrl")
                .and_then(|u| safe_url(u, None))
                .unwrap_or_default(),
            url,
            folders: if folder.is_empty() {
                Vec::new()
            } else {
                vec![folder.clone()]
            },
            folder,
        });
    }
    if subscriptions.is_empty() {
        return Err("No usable RSS or Atom subscriptions were found.".into());
    }
    for subscription in &mut subscriptions {
        subscription.folders.sort();
        subscription.folder = subscription.folders.first().cloned().unwrap_or_default();
    }
    Ok(subscriptions)
}

impl AppState {
    pub fn export_opml(&self) -> String {
        let mut output = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<opml version=\"2.0\">\n<head><title>FerroUI Reader subscriptions</title></head>\n<body>\n");
        for feed in self.feeds.iter().filter(|f| f.folder_names().is_empty()) {
            outline(&mut output, feed);
        }
        for folder in self.folders() {
            output.push_str(&format!(
                "<outline text=\"{}\" title=\"{}\">\n",
                escape(&folder),
                escape(&folder)
            ));
            for feed in self.feeds.iter().filter(|f| f.in_folder(&folder)) {
                outline(&mut output, feed);
            }
            output.push_str("</outline>\n");
        }
        output.push_str("</body>\n</opml>\n");
        output
    }
}

fn outline(output: &mut String, feed: &crate::Feed) {
    output.push_str(&format!(
        "<outline type=\"rss\" text=\"{}\" title=\"{}\" xmlUrl=\"{}\" htmlUrl=\"{}\" />\n",
        escape(&feed.title),
        escape(&feed.title),
        escape(&feed.url),
        escape(&feed.site_url)
    ));
}
