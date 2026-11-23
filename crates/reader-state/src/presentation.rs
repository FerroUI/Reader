use crate::{AppViewModel, HostRequest};
use reader_domain::{Article, Feed, Id, Trends, View};

impl AppViewModel {
    pub fn title(&self) -> String {
        match self.view() {
            View::Home => "Home".into(),
            View::All => "All items".into(),
            View::Starred => "Starred items".into(),
            View::Trends => "Trends".into(),
            View::Explore => "Browse for stuff".into(),
            View::Settings => "Reader settings".into(),
            View::Feed(id) => self
                .state()
                .feed(id)
                .map(|f| f.title.clone())
                .unwrap_or_else(|| "Subscription".into()),
            View::Folder(name) | View::Tag(name) => name,
            View::Search(query) => format!("Search results for “{query}”"),
        }
    }
    pub fn article(&self, id: Id) -> Option<Article> {
        self.state().article(id).cloned()
    }
    pub fn feed(&self, id: Id) -> Option<Feed> {
        self.state().feed(id).filter(|feed| !feed.archived).cloned()
    }
    pub fn feeds(&self) -> Vec<Feed> {
        self.state()
            .feeds
            .iter()
            .filter(|feed| !feed.archived)
            .cloned()
            .collect()
    }
    pub fn folders(&self) -> Vec<String> {
        self.state().folders()
    }
    pub fn tags(&self) -> Vec<String> {
        self.state().tags()
    }
    pub fn count(&self, view: &View) -> usize {
        self.counts.borrow().count(view, &self.state(), true)
    }
    pub fn total_count(&self, view: &View) -> usize {
        self.counts.borrow().count(view, &self.state(), false)
    }
    pub fn trends(&self) -> Trends {
        self.state().trends(self.now())
    }
    pub fn date_label(&self, timestamp: i64) -> String {
        let elapsed = self.now().saturating_sub(timestamp);
        if elapsed < 60 {
            "just now".into()
        } else if elapsed < 3600 {
            format!("{}m", elapsed / 60)
        } else if elapsed < 86400 {
            format!("{}h", elapsed / 3600)
        } else {
            chrono::DateTime::from_timestamp(timestamp, 0)
                .map(|d| d.format("%b %-d, %Y").to_string())
                .unwrap_or_default()
        }
    }
    pub fn open_article(&self, id: Id) {
        if let Some(article) = self.article(id) {
            if reader_domain::safe_url(&article.url, None).is_some() {
                self.request(HostRequest::OpenUrl { url: article.url });
            }
        }
    }
    pub fn share_article(&self, id: Id) {
        if let Some(article) = self.article(id) {
            self.request(HostRequest::Share {
                title: article.title,
                url: article.url,
            });
        }
    }
    pub fn email_article(&self, id: Id) {
        if let Some(article) = self.article(id) {
            self.request(HostRequest::Email {
                title: article.title,
                url: article.url,
            });
        }
    }
    pub fn print_article(&self, id: Id) {
        if let Some(article) = self.article(id) {
            let base_url = article.reading_base_url().to_string();
            let html = article.reading_content().to_string();
            self.request(HostRequest::Print {
                title: article.title,
                html,
                base_url,
            });
        }
    }
}
