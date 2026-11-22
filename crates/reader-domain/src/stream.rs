use crate::{Settings, SortOrder, View, ViewMode};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct StreamSettings {
    pub view_mode: ViewMode,
    pub sort: SortOrder,
    pub unread_only: bool,
}
impl StreamSettings {
    pub fn from_settings(settings: &Settings) -> Self {
        Self {
            view_mode: settings.view_mode,
            sort: settings.sort,
            unread_only: settings.unread_only,
        }
    }
    pub fn apply_to(self, settings: &mut Settings) {
        settings.view_mode = self.view_mode;
        settings.sort = self.sort;
        settings.unread_only = self.unread_only;
    }
}
impl View {
    pub fn storage_key(&self) -> String {
        match self {
            Self::Home => "home".into(),
            Self::All => "all".into(),
            Self::Starred => "starred".into(),
            Self::Trends => "trends".into(),
            Self::Explore => "explore".into(),
            Self::Settings => "settings".into(),
            Self::Feed(id) => format!("feed:{id}"),
            Self::Folder(name) => format!("folder:{name}"),
            Self::Tag(name) => format!("tag:{name}"),
            Self::Search(_) => "search".into(),
        }
    }
}
