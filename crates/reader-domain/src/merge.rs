use crate::{AppState, Article, Changes, Feed, Id, ParsedFeed};
use std::collections::BTreeMap;

impl AppState {
    pub(crate) fn add_subscription(
        &mut self,
        title: String,
        url: String,
        site_url: String,
        folder: String,
    ) -> Result<(Id, bool), String> {
        let url = crate::safe_url(&url, None).ok_or("Enter an http or https feed URL.")?;
        if let Some(feed) = self.feeds.iter().find(|f| f.url == url) {
            return Ok((feed.id, false));
        }
        let id = self.allocate();
        let title = if title.trim().is_empty() {
            url.clone()
        } else {
            title.trim().into()
        };
        let folder = folder.trim().to_string();
        if !folder.is_empty() && !self.folders.contains(&folder) {
            self.folders.push(folder.clone());
        }
        self.feeds.push(Feed {
            id,
            title,
            url,
            site_url: crate::safe_url(&site_url, None).unwrap_or_default(),
            folders: if folder.is_empty() {
                Vec::new()
            } else {
                vec![folder.clone()]
            },
            folder,
            ..Feed::default()
        });
        self.feed_index.insert(id, self.feeds.len() - 1);
        Ok((id, true))
    }

    pub(crate) fn merge_feed(
        &mut self,
        url: String,
        parsed: ParsedFeed,
        now: i64,
    ) -> Result<Changes, String> {
        let (id, added) = self.add_subscription(
            parsed.title.clone(),
            url,
            parsed.site_url.clone(),
            String::new(),
        )?;
        let mut changes = Changes {
            changed: true,
            feeds: true,
            created_feed: added.then_some(id),
            ..Changes::default()
        };
        let was_sample = self.feed(id).is_some_and(|f| f.sample);
        if let Some(source) = self.feed(id).cloned() {
            // A legacy backup can still contain synthetic rows. Match their
            // exact fixture identity/content, never arbitrary publisher data.
            self.articles.retain(|a| {
                let remove = a.feed_id == id && crate::legacy_samples::removable_sample(a, &source);
                if remove {
                    changes.removed.push(a.id);
                }
                !remove
            });
            self.reindex();
        }
        let existing = self
            .articles
            .iter()
            .filter(|a| a.feed_id == id)
            .map(|a| (a.guid.clone(), a.id))
            .collect::<BTreeMap<_, _>>();
        let mut known = existing;
        for item in parsed.articles {
            if let Some(&article_id) = known.get(&item.guid) {
                let article = self.article_mut(article_id).unwrap();
                if article.title != item.title
                    || article.content != item.content
                    || article.content_base != item.content_base
                    || article.published != item.published
                    || article.url != item.url
                    || article.author != item.author
                {
                    article.title = item.title;
                    article.content = item.content;
                    article.content_base = item.content_base;
                    article.summary = item.summary;
                    article.published = item.published;
                    if article.url != item.url {
                        article.full_content = None;
                        article.show_full_content = false;
                    }
                    article.url = item.url;
                    article.author = item.author;
                    article.rebuild_search();
                    changes.articles.push(article_id);
                }
            } else {
                let article_id = self.allocate();
                let mut article = Article {
                    id: article_id,
                    feed_id: id,
                    guid: item.guid.clone(),
                    title: item.title,
                    url: item.url,
                    author: item.author,
                    content: item.content,
                    content_base: item.content_base,
                    summary: item.summary,
                    published: item.published,
                    ..Article::default()
                };
                article.rebuild_search();
                self.articles.push(article);
                self.article_index
                    .insert(article_id, self.articles.len() - 1);
                known.insert(item.guid, article_id);
                changes.articles.push(article_id);
            }
        }
        let feed = self.feed_mut(id).unwrap();
        // Preserve user-renamed titles after the first fetch.
        if added || was_sample || feed.title == feed.url {
            feed.title = parsed.title;
        }
        feed.site_url = parsed.site_url;
        feed.description = parsed.description;
        feed.fetched_at = now;
        feed.error = None;
        feed.sample = false;
        Ok(changes)
    }
}
