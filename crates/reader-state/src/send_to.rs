use crate::{AppViewModel, HostRequest};
use reader_domain::{validate_send_to, Action, Id, SendToDestination};

impl AppViewModel {
    /// Commit a complete validated destination. An invalid draft never replaces
    /// saved preferences, including when editing an existing destination.
    pub fn save_send_to(
        &self,
        id: Option<u64>,
        name: &str,
        template: &str,
        enabled: bool,
    ) -> Result<u64, String> {
        let mut settings = self.state().settings.clone();
        let id = match id {
            Some(id) if settings.send_to.iter().any(|item| item.id == id) => id,
            Some(_) => return Err("That Send to destination no longer exists.".into()),
            None => settings
                .send_to
                .iter()
                .map(|item| item.id)
                .max()
                .unwrap_or(0)
                .checked_add(1)
                .ok_or("Send to destination identifiers are exhausted.")?,
        };
        let destination = SendToDestination {
            id,
            name: name.into(),
            url_template: template.into(),
            enabled,
        };
        if let Some(existing) = settings.send_to.iter_mut().find(|item| item.id == id) {
            *existing = destination;
        } else {
            settings.send_to.push(destination);
        }
        validate_send_to(&mut settings.send_to)?;
        self.dispatch(Action::SetSettings(settings));
        Ok(id)
    }

    pub fn set_send_to_enabled(&self, id: u64, enabled: bool) -> Result<(), String> {
        let mut settings = self.state().settings.clone();
        let destination = settings
            .send_to
            .iter_mut()
            .find(|item| item.id == id)
            .ok_or("That Send to destination no longer exists.")?;
        destination.enabled = enabled;
        self.dispatch(Action::SetSettings(settings));
        Ok(())
    }

    pub fn remove_send_to(&self, id: u64) {
        let mut settings = self.state().settings.clone();
        settings.send_to.retain(|destination| destination.id != id);
        self.dispatch(Action::SetSettings(settings));
    }

    /// A popup can outlive an article update or a recycled row. Bind it to the
    /// source URL, document generation and exact destination it displayed.
    pub fn send_article_to(
        &self,
        id: Id,
        source_url: &str,
        generation: u64,
        destination: &SendToDestination,
    ) -> Result<(), String> {
        if generation != self.library_generation() {
            return Err("The library changed. Open Send to again for the current item.".into());
        }
        let url = {
            let state = self.state();
            let article = state
                .article(id)
                .filter(|article| article.url == source_url)
                .ok_or("This item changed or is no longer in your library.")?;
            let saved = state
                .settings
                .send_to
                .iter()
                .find(|item| *item == destination && item.enabled)
                .ok_or("The Send to destination changed. Open the menu again.")?;
            let source = state
                .feed(article.feed_id)
                .map(|feed| feed.title.as_str())
                .unwrap_or("");
            saved.resolve(&article.url, &article.title, source)?
        };
        self.request(HostRequest::OpenUrl { url });
        Ok(())
    }
}
