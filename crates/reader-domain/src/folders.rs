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
    pub(crate) fn set_feed_folders(
        &mut self,
        id: Id,
        folders: Vec<String>,
    ) -> Result<bool, String> {
        let folders = crate::document::clean_names(folders);
        let feed = self
            .feed_mut(id)
            .ok_or("This subscription no longer exists.")?;
        let changed = feed.folder_names() != folders || feed.folders != folders;
        feed.folder = folders.first().cloned().unwrap_or_default();
        feed.folders = folders.clone();
        for folder in folders {
            if !self.folders.contains(&folder) {
                self.folders.push(folder);
            }
        }
        Ok(changed)
    }
}
