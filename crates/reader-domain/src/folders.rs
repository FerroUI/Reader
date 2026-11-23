use crate::{AppState, Feed, Id};

impl Feed {
    pub fn in_folder(&self, name: &str) -> bool {
        self.folder == name || self.folders.iter().any(|folder| folder == name)
    }
    pub fn folder_names(&self) -> Vec<String> {
        let mut names = self.folders.clone();
        if !self.folder.is_empty() {
            names.push(self.folder.clone());
        }
        crate::document::clean_names(names)
    }
}

impl AppState {
    /// Validate explicit create/rename operations without silently merging
    /// distinct folder identities or mutating a draft on failure.
    pub fn validate_folder_name(
        &self,
        current: Option<&str>,
        input: &str,
    ) -> Result<String, String> {
        let folders = self.folders();
        if current.is_some_and(|name| !folders.iter().any(|folder| folder == name)) {
            return Err("This folder no longer exists.".into());
        }
        let name = input.trim();
        if name.is_empty() {
            return Err("Enter a folder name.".into());
        }
        if name.chars().any(char::is_control) {
            return Err("A folder name cannot contain control characters.".into());
        }
        if current != Some(name) && folders.iter().any(|folder| folder == name) {
            return Err("A folder with this name already exists.".into());
        }
        Ok(name.to_owned())
    }

    pub(crate) fn set_feed_folders(
        &mut self,
        id: Id,
        folders: Vec<String>,
    ) -> Result<bool, String> {
        let folders = crate::document::clean_names(folders);
        let feed = self
            .feed_mut(id)
            .filter(|feed| !feed.archived)
            .ok_or("This subscription no longer exists.")?;
        let changed = feed.folder_names() != folders || feed.folders != folders;
        feed.folder = folders.first().cloned().unwrap_or_default();
        feed.folders = folders.clone();
        let mut known: std::collections::BTreeSet<_> = self.folders.iter().cloned().collect();
        for folder in folders {
            if known.insert(folder.clone()) {
                self.folders.push(folder);
            }
        }
        Ok(changed)
    }
}
