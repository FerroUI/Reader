use crate::{AppState, Article, View};

pub fn matches_view(article: &Article, state: &AppState, view: &View) -> bool {
    match view {
        View::Starred => article.starred,
        View::Feed(id) => article.feed_id == *id,
        View::Folder(name) => state
            .feed(article.feed_id)
            .is_some_and(|f| f.in_folder(name)),
        View::Tag(tag) => article.tags.contains(tag),
        View::Search(query) => search_matches(article, state, query),
        View::Home | View::All | View::Trends => true,
        View::Explore | View::Settings => false,
    }
}

/// Terms are ANDed; quotes group a phrase and `-` excludes a term or operator.
pub fn search_matches(article: &Article, state: &AppState, query: &str) -> bool {
    tokens(query).into_iter().all(|term| {
        let (exclude, term) = term
            .strip_prefix('-')
            .map_or((false, term.as_str()), |s| (true, s));
        let lower = term.to_lowercase();
        let matched = if let Some(value) = lower.strip_prefix("is:") {
            match value {
                "read" => article.read,
                "unread" => !article.read,
                "starred" | "star" => article.starred,
                _ => false,
            }
        } else if let Some(value) = lower.strip_prefix("feed-id:") {
            value.parse::<crate::Id>().ok() == Some(article.feed_id)
        } else if let Some(value) = lower.strip_prefix("folder:") {
            state.feed(article.feed_id).is_some_and(|feed| {
                feed.folder_names()
                    .iter()
                    .any(|folder| folder.to_lowercase() == value)
            })
        } else if let Some(value) = lower
            .strip_prefix("tag:")
            .or_else(|| lower.strip_prefix("label:"))
        {
            article.tags.iter().any(|t| t.to_lowercase() == value)
        } else if let Some(value) = lower
            .strip_prefix("from:")
            .or_else(|| lower.strip_prefix("feed:"))
        {
            state.feed(article.feed_id).is_some_and(|f| {
                f.title.to_lowercase().contains(value) || f.url.to_lowercase().contains(value)
            })
        } else if let Some(value) = lower.strip_prefix("author:") {
            article.author.to_lowercase().contains(value)
        } else if let Some(value) = lower.strip_prefix("title:") {
            article.title.to_lowercase().contains(value)
        } else if let Some(value) = lower.strip_prefix("before:") {
            date(value).is_some_and(|date| article.published < date)
        } else if let Some(value) = lower.strip_prefix("after:") {
            date(value).is_some_and(|date| article.published >= date)
        } else {
            article.search_text.contains(&lower)
        };
        matched != exclude
    })
}

fn date(value: &str) -> Option<i64> {
    chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .ok()
        .and_then(|d| d.and_hms_opt(0, 0, 0))
        .map(|d| d.and_utc().timestamp())
}

fn tokens(query: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut token = String::new();
    let mut quoted = false;
    for character in query.chars() {
        match character {
            '"' => quoted = !quoted,
            c if c.is_whitespace() && !quoted => {
                if !token.is_empty() {
                    result.push(std::mem::take(&mut token));
                }
            }
            c => token.push(c),
        }
    }
    if !token.is_empty() {
        result.push(token);
    }
    result
}
