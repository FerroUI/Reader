use reader_domain::{Feed, FeedConnection, Id};
use serde::Serialize;
use std::rc::Rc;

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum HostRequest {
    FetchArticle {
        id: Id,
        request_id: u64,
        generation: u64,
        url: String,
        proxy: String,
        connection: FeedConnection,
    },
    CancelArticle {
        request_id: u64,
    },
    Refresh {
        feeds: Vec<Feed>,
        proxy: String,
        connection: FeedConnection,
    },
    Subscribe {
        url: String,
        proxy: String,
        connection: FeedConnection,
    },
    OpenUrl {
        url: String,
    },
    Export {
        filename: String,
        contents: String,
        mime_type: String,
    },
    Import {
        format: String,
    },
    Share {
        title: String,
        url: String,
    },
    Email {
        title: String,
        url: String,
    },
    Copy {
        contents: String,
    },
    Print {
        title: String,
        html: String,
        base_url: String,
    },
    FetchImage {
        url: String,
    },
    FocusSearch,
    ScrollToArticle {
        id: Id,
    },
    ScheduleSave,
    ApplyUpdate,
}

#[derive(Clone)]
pub struct HostServices {
    pub request: Rc<dyn Fn(HostRequest)>,
}
impl Default for HostServices {
    fn default() -> Self {
        Self {
            request: Rc::new(|_| {}),
        }
    }
}
pub type SaveCallback = Rc<dyn Fn(&str) -> Result<(), String>>;
