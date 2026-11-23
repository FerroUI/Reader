//! Transient host-update presentation, independent of the saved feed document.
use crate::{AppViewModel, HostRequest};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UpdateNotice {
    pub message: String,
    pub applying: bool,
}

impl AppViewModel {
    pub fn update_notice(&self) -> Option<UpdateNotice> {
        self.update_notice.borrow().clone()
    }

    pub fn set_update_notice(&self, message: String, applying: bool) {
        let next = (!message.is_empty()).then_some(UpdateNotice { message, applying });
        if *self.update_notice.borrow() != next {
            *self.update_notice.borrow_mut() = next;
            self.notify();
        }
    }

    pub fn dismiss_update(&self) {
        if self.update_notice().is_some_and(|notice| !notice.applying) {
            self.set_update_notice(String::new(), false);
        }
    }

    pub fn apply_update(&self) {
        if !self.update_notice().is_some_and(|notice| !notice.applying) {
            return;
        }
        self.set_update_notice("Saving your library and updating Reader…".into(), true);
        self.request(HostRequest::ApplyUpdate);
    }
}
