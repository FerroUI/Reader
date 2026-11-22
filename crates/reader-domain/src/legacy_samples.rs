//! Conservatively retire the old sample library without carrying its stories
//! into the application binary or changing a user's real subscription choices.
use crate::{AppState, Article, Feed, DEFAULT_FEEDS};

const DESCRIPTION: &str = "Starter subscription with illustrative sample articles. Refresh to fetch the publisher's live feed.";
const LEGACY: &[(&str, &str, &str, &str)] = &[
    (
        "The Rust Blog",
        "https://blog.rust-lang.org/feed.xml",
        "https://blog.rust-lang.org/",
        "Technology",
    ),
    (
        "NASA",
        "https://www.nasa.gov/feed/",
        "https://www.nasa.gov/",
        "Science",
    ),
    (
        "Smashing Magazine",
        "https://www.smashingmagazine.com/feed/",
        "https://www.smashingmagazine.com/",
        "Design",
    ),
    (
        "Mozilla Hacks",
        "https://hacks.mozilla.org/feed/",
        "https://hacks.mozilla.org/",
        "Technology",
    ),
    (
        "A List Apart",
        "https://alistapart.com/main/feed/",
        "https://alistapart.com/",
        "Design",
    ),
    (
        "Hacker News",
        "https://hnrss.org/frontpage",
        "https://news.ycombinator.com/",
        "Technology",
    ),
];
// Stable content fingerprints identify exactly the nine shipped fixture rows.
// GUIDs, author, source URL and expected subscription must also match.
const STORIES: &[(usize, u64)] = &[
    (0, 0xc2ddccb2c0a98975),
    (1, 0x1acf726cfe847459),
    (2, 0xf17ea499278c1dec),
    (3, 0x52bdd5ad7fa3786e),
    (4, 0xa0ecbcbf20e1ca38),
    (5, 0x69d9b81b472db176),
    (0, 0x87e4129b8fc2dd50),
    (2, 0x74b55da9c1d7f047),
    (1, 0x4c5898df2cb80e40),
];

fn sample_index(article: &Article, feed: &Feed) -> Option<usize> {
    let index = article
        .guid
        .strip_prefix("reader-sample-")?
        .parse::<usize>()
        .ok()?;
    let &(source, fingerprint) = STORIES.get(index)?;
    if article.guid != format!("reader-sample-{index}")
        || article.author != "Reader sample library"
        || feed.url != LEGACY[source].1
        || article.url != LEGACY[source].2
        || !article.content_base.is_empty()
    {
        return None;
    }
    let mut hash = 0xcbf29ce484222325_u64;
    for field in [&article.title, &article.summary, &article.content] {
        for byte in field.bytes().chain(std::iter::once(0)) {
            hash = (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3);
        }
    }
    (hash == fingerprint).then_some(index)
}

pub(crate) fn removable_sample(article: &Article, feed: &Feed) -> bool {
    let Some(index) = sample_index(article, feed) else {
        return false;
    };
    // The original fixture already starred/tagged its third row. Those defaults
    // are not user annotations; an added star, tag or note is preserved.
    article.note.is_empty()
        && (!article.starred || index == 2)
        && (article.tags.is_empty() || (index == 2 && article.tags == ["Inspiration"]))
}

fn untouched(state: &AppState) -> bool {
    state.feeds.len() == LEGACY.len()
        && state.articles.len() == STORIES.len()
        && state.folders == ["Design", "Science", "Technology"]
        && state.feeds.iter().zip(LEGACY).all(|(feed, source)| {
            feed.sample
                && feed.fetched_at == 0
                && feed.title == source.0
                && feed.url == source.1
                && feed.site_url == source.2
                && feed.folder == source.3
                && feed.folders == [source.3]
                && feed.description == DESCRIPTION
        })
        && state.articles.iter().all(|article| {
            state
                .feed(article.feed_id)
                .is_some_and(|feed| removable_sample(article, feed))
        })
}

impl AppState {
    /// Run once on normal saved-library loading, not on explicit backup restore.
    /// Only an intact old demo gets the new catalogue; customized libraries keep
    /// their chosen feeds and any user annotations on recognizable sample rows.
    pub fn migrate_legacy_samples(&mut self) -> bool {
        let replace_catalogue = untouched(self)
            && self
                .next_id
                .checked_add(DEFAULT_FEEDS.len() as u64)
                .is_some_and(|next| next < u64::MAX);
        let removed: std::collections::BTreeSet<_> = self
            .articles
            .iter()
            .filter(|article| {
                self.feed(article.feed_id)
                    .is_some_and(|feed| removable_sample(article, feed))
            })
            .map(|article| article.id)
            .collect();
        let mut changed = !removed.is_empty();
        self.articles
            .retain(|article| !removed.contains(&article.id));
        if replace_catalogue {
            let old = std::mem::take(&mut self.feeds);
            for source in DEFAULT_FEEDS {
                let id = old
                    .iter()
                    .find(|feed| {
                        feed.url == source.url
                            || (source.title == "Hacker News" && feed.url == LEGACY[5].1)
                    })
                    .map(|feed| feed.id)
                    .unwrap_or_else(|| self.allocate());
                self.feeds.push(source.subscription(id));
            }
            self.folders = crate::document::clean_names(
                DEFAULT_FEEDS
                    .iter()
                    .map(|source| source.folder.into())
                    .collect(),
            );
            changed = true;
        } else {
            for feed in &mut self.feeds {
                if feed.sample {
                    feed.sample = false;
                    changed = true;
                }
                if feed.description == DESCRIPTION {
                    feed.description.clear();
                    changed = true;
                }
            }
        }
        if changed {
            self.reindex();
        }
        changed
    }
}
