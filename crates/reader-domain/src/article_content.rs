//! Bounded, portable publisher copies. Feed bodies and user annotations remain independent.
use crate::{AppState, Article, Changes, Id};
use serde::{Deserialize, Serialize};
use std::rc::Rc;

pub const MAX_ARTICLE_HTML_BYTES: usize = 2 * 1024 * 1024;
pub const MAX_ARTICLE_CACHE_BYTES: usize = 24 * 1024 * 1024;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FullArticle {
    pub html: String,
    /// The original article link, distinct from its redirected document base.
    pub source_url: String,
    pub base_url: String,
    pub fetched_at: i64,
}

impl FullArticle {
    pub fn new(
        html: String,
        source_url: &str,
        base_url: &str,
        fetched_at: i64,
    ) -> Result<Self, String> {
        let content = Self {
            html,
            source_url: crate::safe_url(source_url, None).ok_or("Invalid article URL.")?,
            base_url: crate::safe_url(base_url, None).ok_or("Invalid article document URL.")?,
            fetched_at,
        };
        content.validate()?;
        Ok(content)
    }

    fn validate(&self) -> Result<(), String> {
        if self.html.len() > MAX_ARTICLE_HTML_BYTES {
            return Err("This article exceeds the 2 MB reading limit.".into());
        }
        if self.html.trim().is_empty() || crate::plain_text(&self.html).is_empty() {
            return Err("The publisher did not provide readable article content.".into());
        }
        if !valid_url(&self.source_url) || !valid_url(&self.base_url) {
            return Err("Invalid article document URL.".into());
        }
        Ok(())
    }
}

fn valid_url(value: &str) -> bool {
    // Subscription validation rejects credentials; preserve article fragments
    // because they are part of the saved link identity and original navigation.
    crate::subscription_url(value).is_some()
        && crate::safe_url(value, None).as_deref() == Some(value)
}

pub(crate) fn is_false(value: &bool) -> bool {
    !value
}

impl Article {
    pub fn has_full_content(&self) -> bool {
        self.full_content
            .as_ref()
            .is_some_and(|full| full.source_url == self.url)
    }

    pub fn reading_content(&self) -> &str {
        self.full_content
            .as_ref()
            .filter(|full| self.show_full_content && full.source_url == self.url)
            .map_or(self.content.as_str(), |full| full.html.as_str())
    }

    pub fn reading_base_url(&self) -> &str {
        self.full_content
            .as_ref()
            .filter(|full| self.show_full_content && full.source_url == self.url)
            .map_or_else(|| self.content_base_url(), |full| full.base_url.as_str())
    }

    pub(crate) fn normalize_full_content(&mut self) {
        if self
            .full_content
            .as_ref()
            .is_some_and(|full| full.source_url != self.url || full.validate().is_err())
        {
            self.full_content = None;
        }
        self.show_full_content &= self.full_content.is_some();
    }
}

impl AppState {
    pub(crate) fn cache_full_article(
        &mut self,
        id: Id,
        content: FullArticle,
    ) -> Result<Changes, String> {
        content.validate()?;
        let article = self
            .article_mut(id)
            .ok_or("This article no longer exists.")?;
        if article.url != content.source_url {
            return Err("The article address changed while its content was loading.".into());
        }
        article.full_content = Some(Rc::new(content));
        article.show_full_content = true;
        article.rebuild_search();
        let mut articles = self.trim_article_cache(Some(id));
        articles.push(id);
        Ok(Changes {
            changed: true,
            articles,
            ..Changes::default()
        })
    }

    /// Keep the newly opened article and retire only downloaded copies, never
    /// feed content, read state or user annotations. Import applies the same cap.
    pub(crate) fn trim_article_cache(&mut self, preserve: Option<Id>) -> Vec<Id> {
        self.trim_article_cache_to(preserve, MAX_ARTICLE_CACHE_BYTES)
    }

    fn trim_article_cache_to(&mut self, preserve: Option<Id>, limit: usize) -> Vec<Id> {
        let mut total: usize = self
            .articles
            .iter()
            .filter_map(|article| article.full_content.as_ref().map(|full| full.html.len()))
            .sum();
        if total <= limit {
            return Vec::new();
        }
        let mut oldest: Vec<_> = self
            .articles
            .iter()
            .enumerate()
            .filter(|(_, article)| Some(article.id) != preserve)
            .filter_map(|(index, article)| {
                article
                    .full_content
                    .as_ref()
                    .map(|full| (full.fetched_at, article.id, index))
            })
            .collect();
        oldest.sort_unstable();
        let mut removed = Vec::new();
        for (_, _, index) in oldest {
            if total <= limit {
                break;
            }
            let article = &mut self.articles[index];
            if let Some(full) = article.full_content.take() {
                total -= full.html.len();
                article.show_full_content = false;
                article.rebuild_search();
                removed.push(article.id);
            }
        }
        removed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bounded_eviction_keeps_requested_copy_and_every_annotation() {
        let mut state = AppState::new();
        for (id, fetched_at) in [(1, 50), (2, 10), (3, 30)] {
            state.articles.push(Article {
                id,
                url: format!("https://example.com/{id}"),
                content: "Feed body".into(),
                note: "My note".into(),
                starred: true,
                read: true,
                tags: vec!["Saved".into()],
                full_content: Some(Rc::new(FullArticle {
                    html: "<p>Full copy</p>".into(),
                    source_url: format!("https://example.com/{id}"),
                    base_url: format!("https://example.com/{id}"),
                    fetched_at,
                })),
                show_full_content: true,
                ..Article::default()
            });
        }
        let removed = state.trim_article_cache_to(Some(2), 32);
        assert_eq!(removed, [3]);
        assert!(
            state.articles[1].has_full_content(),
            "the requested copy is retained even with an older timestamp"
        );
        let evicted = &state.articles[2];
        assert!(!evicted.show_full_content);
        assert_eq!(evicted.reading_content(), "Feed body");
        assert!(evicted.read && evicted.starred);
        assert_eq!(evicted.note, "My note");
        assert_eq!(evicted.tags, ["Saved"]);
    }
}
