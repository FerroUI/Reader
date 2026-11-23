use crate::{Action, AppState, Changes};

impl AppState {
    pub fn apply(&mut self, action: Action, now: i64) -> Result<Changes, String> {
        let mut changes = Changes::default();
        match action {
            Action::CacheFullArticle { id, content } => {
                return self.cache_full_article(id, content);
            }
            Action::ShowFullArticle { id, show } => {
                let article = self
                    .article_mut(id)
                    .ok_or("This article no longer exists.")?;
                let show = show && article.has_full_content();
                if article.show_full_content != show {
                    article.show_full_content = show;
                    changes.articles.push(id);
                }
            }
            Action::SetRead { id, read } => {
                let article = self
                    .article_mut(id)
                    .ok_or("This article no longer exists.")?;
                if article.read != read {
                    article.read = read;
                    article.read_at = read.then_some(now);
                    changes.articles.push(id);
                }
            }
            Action::ToggleStar(id) => {
                let article = self
                    .article_mut(id)
                    .ok_or("This article no longer exists.")?;
                article.starred = !article.starred;
                changes.articles.push(id);
            }
            Action::SetTags { id, tags } => {
                let article = self
                    .article_mut(id)
                    .ok_or("This article no longer exists.")?;
                let tags = crate::document::clean_names(tags);
                if article.tags != tags {
                    article.tags = tags;
                    article.rebuild_search();
                    changes.articles.push(id);
                }
            }
            Action::SetNote { id, note } => {
                let article = self
                    .article_mut(id)
                    .ok_or("This article no longer exists.")?;
                if article.note != note {
                    article.note = note;
                    article.rebuild_search();
                    changes.articles.push(id);
                }
            }
            Action::MarkRead { ids } => {
                for id in ids {
                    if let Some(article) = self.article_mut(id) {
                        if !article.read {
                            article.read = true;
                            article.read_at = Some(now);
                            changes.articles.push(id);
                        }
                    }
                }
            }
            Action::RenameFeed { id, title } => {
                let title = title.trim();
                if title.is_empty() {
                    return Err("A subscription name cannot be empty.".into());
                }
                let feed = self
                    .feed_mut(id)
                    .ok_or("This subscription no longer exists.")?;
                changes.feeds = feed.title != title;
                feed.title = title.into();
            }
            Action::MoveFeed { id, folder } => {
                changes.feeds = self.set_feed_folders(id, vec![folder])?;
            }
            Action::PositionFeed {
                id,
                target,
                before,
                folder,
            } => {
                changes.feeds = self.position_feed(id, target, before, folder)?;
            }
            Action::SetFeedFolders { id, folders } => {
                changes.feeds = self.set_feed_folders(id, folders)?;
            }
            Action::RemoveFeed(id) => {
                changes.removed = self
                    .articles
                    .iter()
                    .filter(|a| a.feed_id == id)
                    .map(|a| a.id)
                    .collect();
                changes.feeds = self.feeds.iter().any(|f| f.id == id);
                self.feeds.retain(|f| f.id != id);
                self.articles.retain(|a| a.feed_id != id);
                self.reindex();
            }
            Action::CreateFolder(name) => {
                let name = name.trim().to_string();
                if name.is_empty() {
                    return Err("Enter a folder name.".into());
                }
                if !self.folders().contains(&name) {
                    self.folders.push(name);
                    changes.feeds = true;
                }
            }
            Action::RenameFolder { from, to } => {
                let to = to.trim().to_string();
                if to.is_empty() {
                    return Err("Enter a folder name.".into());
                }
                if from != to {
                    for folder in &mut self.folders {
                        if *folder == from {
                            *folder = to.clone();
                            changes.feeds = true;
                        }
                    }
                    for feed in &mut self.feeds {
                        if feed.in_folder(&from) {
                            feed.folders = crate::document::clean_names(
                                feed.folder_names()
                                    .into_iter()
                                    .map(|f| if f == from { to.clone() } else { f })
                                    .collect(),
                            );
                            feed.folder = feed.folders.first().cloned().unwrap_or_default();
                            changes.feeds = true;
                        }
                    }
                    if let Some(display) = self.stream_settings.remove(&format!("folder:{from}")) {
                        self.stream_settings
                            .entry(format!("folder:{to}"))
                            .or_insert(display);
                    }
                    self.folders = crate::document::clean_names(std::mem::take(&mut self.folders));
                }
            }
            Action::DeleteFolder(name) => {
                changes.feeds = self.folders().contains(&name);
                self.folders.retain(|f| *f != name);
                for feed in &mut self.feeds {
                    feed.folders = feed
                        .folder_names()
                        .into_iter()
                        .filter(|f| f != &name)
                        .collect();
                    feed.folder = feed.folders.first().cloned().unwrap_or_default();
                }
            }
            Action::SetSettings(mut settings) => {
                crate::validate_send_to(&mut settings.send_to)?;
                if !settings.font_size.is_finite() {
                    return Err("Text size must be a finite number.".into());
                }
                settings.font_size = settings.font_size.clamp(11.0, 26.0);
                changes.settings = self.settings != settings;
                self.settings = settings;
            }
            Action::SetStreamSettings { stream, settings } => {
                let previous = self.stream_settings.get(&stream).copied();
                let display = crate::StreamSettings::from_settings(&settings);
                let mut changes = self.apply(Action::SetSettings(settings), now)?;
                self.stream_settings.insert(stream, display);
                changes.settings |= previous != Some(display);
                changes.changed |= changes.settings;
                return Ok(changes);
            }
            Action::AddSubscription {
                title,
                url,
                site_url,
                folder,
            } => {
                let (id, added) = self.add_subscription(title, url, site_url, folder)?;
                changes.feeds = added;
                changes.created_feed = Some(id);
            }
            Action::MergeFeed { url, feed } => return self.merge_feed(url, feed, now),
            Action::FeedError { url, message } => {
                if let Some(feed) = self.feeds.iter_mut().find(|f| f.url == url) {
                    changes.feeds = feed.error.as_ref() != Some(&message);
                    feed.error = Some(message);
                }
            }
        }
        changes.articles.sort_unstable();
        changes.articles.dedup();
        changes.changed = !changes.articles.is_empty()
            || !changes.removed.is_empty()
            || changes.feeds
            || changes.settings;
        Ok(changes)
    }
}
