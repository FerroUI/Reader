//! First use and legacy migration are separate from restoring an explicit backup.
use reader_domain::AppState;

pub struct Loaded {
    pub state: AppState,
    pub error: Option<String>,
    pub changed: bool,
}

pub fn read_state() -> Loaded {
    match crate::host::load() {
        Ok(json) if json.trim().is_empty() => Loaded {
            state: AppState::with_default_feeds(),
            error: None,
            changed: true,
        },
        Ok(json) => match AppState::from_json(&json) {
            Ok(mut state) => {
                let changed = state.migrate_legacy_samples();
                Loaded {
                    state,
                    error: None,
                    changed,
                }
            }
            Err(error) => {
                let recovery = if crate::host::preserve_recovery(&json) {
                    "A recovery copy was preserved."
                } else {
                    "The original saved data is protected from overwriting. Export a backup before reloading."
                };
                Loaded {
                    state: AppState::new(),
                    error: Some(format!(
                        "Your saved library could not be read. {recovery} {error}"
                    )),
                    changed: false,
                }
            }
        },
        Err(error) => Loaded {
            state: AppState::new(),
            error: Some(crate::host::error_message(error)),
            changed: false,
        },
    }
}
