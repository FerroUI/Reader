use reader_domain::{Feed, Id};
use serde::Serialize;
use std::rc::Rc;

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum HostRequest {
    Refresh {
        feeds: Vec<Feed>,
        proxy: String,
    },
    Subscribe {
        url: String,
        proxy: String,
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
