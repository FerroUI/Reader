//! Portable Send to destinations. A template describes an explicit user action;
//! it never runs while articles or settings are being displayed.
mod template;

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const MAX_SEND_TO_DESTINATIONS: usize = 32;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct SendToDestination {
    pub id: u64,
    pub name: String,
    pub url_template: String,
    pub enabled: bool,
}

impl SendToDestination {
    pub fn resolve(&self, article_url: &str, title: &str, source: &str) -> Result<String, String> {
        if !self.enabled {
            return Err("This Send to destination is disabled.".into());
        }
        template::resolve(&self.url_template, article_url, title, source)
    }
}

pub fn validate_send_to(destinations: &mut [SendToDestination]) -> Result<(), String> {
    if destinations.len() > MAX_SEND_TO_DESTINATIONS {
        return Err(format!(
            "You can configure up to {MAX_SEND_TO_DESTINATIONS} Send to destinations."
        ));
    }
    let mut ids = BTreeSet::new();
    let mut names = BTreeSet::new();
    for destination in destinations {
        if destination.id == 0 || !ids.insert(destination.id) {
            return Err("A Send to destination has a duplicate or invalid identifier.".into());
        }
        destination.name = destination.name.trim().to_string();
        if destination.name.is_empty()
            || destination.name.chars().count() > 80
            || destination.name.chars().any(char::is_control)
        {
            return Err("Give the Send to destination a name of 1–80 characters.".into());
        }
        if !names.insert(destination.name.to_lowercase()) {
            return Err("Choose a different name; that Send to destination already exists.".into());
        }
        destination.url_template = template::validate(&destination.url_template)?;
    }
    Ok(())
}
