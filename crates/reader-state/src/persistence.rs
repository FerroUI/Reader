use crate::{history::UndoEntry, AppViewModel, HostRequest};
use reader_domain::{Action, AppState, Changes};

impl AppViewModel {
    /// Asynchronous hosts call this when a queued write fails after the synchronous save callback returned.
    pub fn persistence_failed(&self, message: String) {
        self.dirty.set(true);
        self.report_error(format!(
            "Could not save your library: {message}. Export a backup before closing this page."
        ));
    }
    pub(crate) fn schedule_save(&self) {
        self.dirty.set(true);
        if self.has_services.get() {
            self.request(HostRequest::ScheduleSave);
        } else {
            let _ = self.flush();
        }
    }
    /// The host coalesces this operation; all updates preceding this call persist atomically.
    pub fn flush(&self) -> Result<(), String> {
        if !self.dirty.replace(false) {
            return Ok(());
        }
        let result = self.export_json().and_then(|json| (self.save)(&json));
        if let Err(error) = &result {
            self.dirty.set(true);
            self.report_error(format!(
                "Could not save your library: {error}. Export a backup before closing this page."
            ));
        }
        result
    }
    pub fn export_json(&self) -> Result<String, String> {
        self.state().to_json()
    }
    pub fn export_opml(&self) -> String {
        self.state().export_opml()
    }
    pub fn import_json(&self, json: &str) -> Result<(), String> {
        let state = AppState::from_json(json)?;
        let previous = self.document.replace(state);
        self.restore_stream_settings();
        let changes = Changes {
            changed: true,
            removed: previous
                .articles
                .iter()
                .filter(|a| self.state().article(a.id).is_none())
                .map(|a| a.id)
                .collect(),
            articles: self.state().articles.iter().map(|a| a.id).collect(),
            feeds: true,
            settings: true,
            created_feed: None,
        };
        self.undo.borrow_mut().clear();
        self.undo
            .borrow_mut()
            .push_back(UndoEntry::snapshot(previous));
        self.selected.set(None);
        self.publish(&changes);
        self.schedule_save();
        self.set_status("Reader backup restored. You can undo this change.".into());
        Ok(())
    }
    pub fn import_opml(&self, source: &str) -> Result<usize, String> {
        let subscriptions = reader_domain::parse_opml(source)?;
        let folders = self.state().folders.clone();
        let mut added = Vec::new();
        let mut memberships = Vec::new();
        let mut modified = false;
        {
            let mut state = self.document.borrow_mut();
            for sub in subscriptions {
                let changes = state.apply(
                    Action::AddSubscription {
                        title: sub.title,
                        url: sub.url,
                        site_url: sub.site_url,
                        folder: sub.folder,
                    },
                    self.now(),
                )?;
                if changes.changed {
                    added.extend(changes.created_feed);
                }
                modified |= changes.changed;
                if let Some(id) = changes.created_feed {
                    let before = state.feed(id).unwrap().folder_names();
                    let mut after = before.clone();
                    after.extend(sub.folders);
                    let changed =
                        state.apply(Action::SetFeedFolders { id, folders: after }, self.now())?;
                    if changed.changed {
                        modified = true;
                        if !added.contains(&id) {
                            memberships.push((id, before));
                        }
                    }
                }
            }
        }
        let count = added.len();
        if modified {
            let mut undo = self.undo.borrow_mut();
            undo.push_back(UndoEntry::subscriptions(added, folders, memberships));
            if undo.len() > 30 {
                undo.pop_front();
            }
            drop(undo);
            self.publish(&Changes {
                changed: true,
                feeds: true,
                ..Changes::default()
            });
            self.schedule_save();
        }
        self.set_status(format!(
            "Imported {count} new subscriptions. Refresh to download their articles."
        ));
        Ok(count)
    }
    pub fn request_export_opml(&self) {
        self.request(HostRequest::Export {
            filename: "reader-subscriptions.opml".into(),
            contents: self.export_opml(),
            mime_type: "text/x-opml+xml".into(),
        });
    }
    pub fn request_export_json(&self) {
        match self.export_json() {
            Ok(contents) => self.request(HostRequest::Export {
                filename: "reader-backup.json".into(),
                contents,
                mime_type: "application/json".into(),
            }),
            Err(error) => self.report_error(error),
        }
    }
    pub fn request_import_opml(&self) {
        self.request(HostRequest::Import {
            format: "opml".into(),
        });
    }
    pub fn request_import_json(&self) {
        self.request(HostRequest::Import {
            format: "json".into(),
        });
    }
}
