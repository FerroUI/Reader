use crate::AppViewModel;
use reader_domain::{validate_feed_proxy, FeedConnection};

impl AppViewModel {
    /// Commit connection policy and its validated endpoint as one settings edit.
    pub fn set_feed_connection(
        &self,
        connection: FeedConnection,
        proxy: &str,
    ) -> Result<(), String> {
        let proxy = validate_feed_proxy(proxy).map_err(str::to_owned)?;
        if connection == FeedConnection::Custom && proxy.is_empty() {
            return Err("Enter a custom proxy URL containing {url}, or choose Automatic.".into());
        }
        let mut settings = self.state().settings.clone();
        settings.feed_connection = connection;
        settings.feed_proxy = proxy;
        self.set_settings(settings);
        self.set_status("Feed connection saved.".into());
        Ok(())
    }
}
