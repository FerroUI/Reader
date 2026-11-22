use crate::{AppState, Id};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Default)]
pub struct FeedTrend {
    pub feed_id: Id,
    pub title: String,
    pub total: usize,
    pub read: usize,
    pub unread: usize,
    pub percent: f64,
}

#[derive(Clone, Debug, Default)]
pub struct Trends {
    pub total: usize,
    pub read: usize,
    pub unread: usize,
    pub starred: usize,
    pub subscriptions: usize,
    pub read_last_30_days: usize,
    pub days: Vec<(i64, usize)>,
    pub feeds: Vec<FeedTrend>,
}

impl AppState {
    pub fn trends(&self, now: i64) -> Trends {
        let today = now.div_euclid(86400);
        let mut days = ((today - 29)..=today)
            .map(|d| (d * 86400, 0))
            .collect::<BTreeMap<_, _>>();
        let mut by_feed = self
            .feeds
            .iter()
            .map(|f| {
                (
                    f.id,
                    FeedTrend {
                        feed_id: f.id,
                        title: f.title.clone(),
                        ..FeedTrend::default()
                    },
                )
            })
            .collect::<BTreeMap<_, _>>();
        let mut result = Trends {
            total: self.articles.len(),
            subscriptions: self.feeds.len(),
            ..Trends::default()
        };
        for article in &self.articles {
            result.read += usize::from(article.read);
            result.starred += usize::from(article.starred);
            if let Some(at) = article.read_at {
                if let Some(day) = days.get_mut(&(at.div_euclid(86400) * 86400)) {
                    *day += 1;
                    result.read_last_30_days += 1;
                }
            }
            if let Some(feed) = by_feed.get_mut(&article.feed_id) {
                feed.total += 1;
                feed.read += usize::from(article.read);
            }
        }
        result.unread = result.total - result.read;
        result.days = days.into_iter().collect();
        result.feeds = by_feed
            .into_values()
            .map(|mut f| {
                f.unread = f.total - f.read;
                f.percent = if f.total == 0 {
                    0.0
                } else {
                    f.read as f64 * 100.0 / f.total as f64
                };
                f
            })
            .collect();
        result
            .feeds
            .sort_by(|a, b| b.read.cmp(&a.read).then(a.title.cmp(&b.title)));
        result
    }
}
