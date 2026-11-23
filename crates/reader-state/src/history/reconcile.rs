//! Restore a removed subscription without discarding subsequent downloads or
//! creating duplicate URL/GUID identities. Current live identities stay stable.
use super::aliases::Aliases;
use reader_domain::{AppState, Article, Feed, Id, View};
use std::collections::{BTreeMap, BTreeSet};

pub(super) struct Index {
    by_feed: BTreeMap<Id, BTreeMap<String, usize>>,
    used: BTreeSet<Id>,
}

impl Index {
    pub fn new(state: &AppState) -> Self {
        let mut by_feed = BTreeMap::<Id, BTreeMap<String, usize>>::new();
        let mut used = state
            .feeds
            .iter()
            .map(|feed| feed.id)
            .collect::<BTreeSet<_>>();
        for (index, article) in state.articles.iter().enumerate() {
            used.insert(article.id);
            by_feed
                .entry(article.feed_id)
                .or_default()
                .insert(article.guid.clone(), index);
        }
        Self { by_feed, used }
    }
}

pub(super) fn restore_feed(
    state: &mut AppState,
    saved: &Feed,
    position: usize,
    articles: &[(usize, Article)],
    pending: &mut Vec<(usize, Article)>,
    aliases: &mut Aliases,
    index: &mut Index,
) -> Result<(), String> {
    let saved_identity = state.feeds.iter().find(|feed| feed.id == saved.id);
    if saved_identity.map_or(index.used.contains(&saved.id), |feed| feed.url != saved.url) {
        return Err("A restored subscription identifier belongs to another item.".into());
    }
    let id = if let Some(current) = state.feeds.iter_mut().find(|feed| feed.url == saved.url) {
        let folders: BTreeSet<_> = current
            .folder_names()
            .into_iter()
            .chain(saved.folder_names())
            .collect();
        current.folders = folders.into_iter().collect();
        current.folder = current.folders.first().cloned().unwrap_or_default();
        // Restore the user's subscription name. Publisher metadata and current
        // downloads remain newer than the removal snapshot.
        current.title = saved.title.clone();
        current.archived = false;
        current.id
    } else {
        if !index.used.insert(saved.id) {
            return Err("A restored subscription identifier belongs to another item.".into());
        }
        let mut feed = saved.clone();
        feed.archived = false;
        state.feeds.insert(position.min(state.feeds.len()), feed);
        saved.id
    };
    if saved.id != id {
        aliases.feeds.insert(saved.id, id);
        let source = View::Feed(saved.id).storage_key();
        if let Some(display) = state.stream_settings.remove(&source) {
            state
                .stream_settings
                .entry(View::Feed(id).storage_key())
                .or_insert(display);
        }
        aliases.settings(&mut state.settings);
    }
    for (position, saved_article) in articles {
        if saved_article.feed_id != saved.id {
            return Err("A restored article refers to a different subscription.".into());
        }
        if let Some(&position) = index
            .by_feed
            .get(&id)
            .and_then(|articles| articles.get(&saved_article.guid))
        {
            let current = &mut state.articles[position];
            if saved_article.id != current.id {
                if !index.used.insert(saved_article.id) {
                    return Err("A restored article identifier belongs to another item.".into());
                }
                aliases.articles.insert(saved_article.id, current.id);
            }
            merge_annotations(current, saved_article);
        } else {
            if !index.used.insert(saved_article.id) {
                return Err("A restored article identifier belongs to another item.".into());
            }
            let mut article = saved_article.clone();
            article.feed_id = id;
            pending.push((*position, article));
        }
    }
    Ok(())
}

fn merge_annotations(current: &mut Article, saved: &Article) {
    // Restoring a removal recovers saved annotations without erasing annotations
    // on the replacement. Newer publisher fields and downloaded copies win.
    if !current.read && saved.read {
        current.read = true;
        current.read_at = saved.read_at;
    }
    current.starred |= saved.starred;
    current.tags = current
        .tags
        .iter()
        .chain(&saved.tags)
        .cloned()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    if current.note.is_empty() {
        current.note = saved.note.clone();
    } else if !saved.note.is_empty() && current.note != saved.note {
        current.note.push_str("\n\n");
        current.note.push_str(&saved.note);
    }
    if current.full_content.is_none() && current.url == saved.url && saved.has_full_content() {
        current.full_content = saved.full_content.clone();
        current.show_full_content = saved.show_full_content;
    }
    current.rebuild_search();
}
