use std::collections::BTreeMap;

pub(crate) const TAP_SLOP: f64 = 8.0;

/// Maximum excursion is sticky: moving back to the press point is not a tap.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Motion {
    origin: (f64, f64),
    distance: f64,
}
impl Motion {
    pub fn new(x: f64, y: f64) -> Self {
        Self {
            origin: (x, y),
            distance: if x.is_finite() && y.is_finite() {
                0.0
            } else {
                f64::INFINITY
            },
        }
    }
    pub fn update(&mut self, x: f64, y: f64) {
        let distance = (x - self.origin.0).hypot(y - self.origin.1);
        self.distance = if distance.is_finite() {
            self.distance.max(distance)
        } else {
            f64::INFINITY
        };
    }
    pub fn within_tap(&self) -> bool {
        self.distance < TAP_SLOP
    }
    pub fn traveled(&self, distance: f64) -> bool {
        self.distance >= distance
    }
}

struct Contact {
    motion: Motion,
    canceled: bool,
}

/// Platform-independent state used by the routed-event adapter. Capturing a
/// ScrollGestureRecognizer cancels this sequence through PointerCaptureLost.
#[derive(Default)]
pub(crate) struct Contacts(BTreeMap<i32, Contact>);
impl Contacts {
    pub fn press(&mut self, id: i32, primary: bool, x: f64, y: f64) -> bool {
        let single = self.0.is_empty() && primary;
        for contact in self.0.values_mut() {
            contact.canceled = true;
        }
        let motion = Motion::new(x, y);
        let eligible = single && motion.within_tap();
        self.0.insert(
            id,
            Contact {
                motion,
                canceled: !eligible,
            },
        );
        eligible
    }
    pub fn moved(&mut self, id: i32, x: f64, y: f64) {
        if let Some(contact) = self.0.get_mut(&id) {
            contact.motion.update(x, y);
        }
    }
    pub fn hold(&mut self, id: i32) {
        if let Some(contact) = self.0.get_mut(&id) {
            contact.canceled = true;
        }
    }
    pub fn release(&mut self, id: i32, x: f64, y: f64) -> bool {
        let Some(mut contact) = self.0.remove(&id) else {
            return false;
        };
        contact.motion.update(x, y);
        !contact.canceled && contact.motion.within_tap()
    }
    pub fn lost(&mut self, id: i32) {
        self.0.remove(&id);
    }
    pub fn clear(&mut self) {
        self.0.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::Contacts;
    #[test]
    fn taps_allow_jitter_but_a_pan_returning_to_origin_stays_canceled() {
        let mut contacts = Contacts::default();
        assert!(contacts.press(1, true, 20.0, 30.0));
        contacts.moved(1, 23.0, 31.0);
        assert!(contacts.release(1, 20.0, 31.0));
        contacts.press(2, true, 20.0, 30.0);
        contacts.moved(2, 20.0, 38.0);
        contacts.moved(2, 20.0, 30.0);
        assert!(!contacts.release(2, 20.0, 30.0));
    }
    #[test]
    fn multitouch_cancels_both_commands_and_next_single_touch_recovers() {
        let mut contacts = Contacts::default();
        contacts.press(1, true, 0.0, 0.0);
        assert!(!contacts.press(2, false, 10.0, 10.0));
        assert!(!contacts.release(1, 0.0, 0.0));
        assert!(!contacts.release(2, 10.0, 10.0));
        assert!(contacts.press(3, true, 0.0, 0.0));
        assert!(contacts.release(3, 0.0, 0.0));
    }
    #[test]
    fn capture_loss_hold_and_orphan_releases_never_activate() {
        let mut contacts = Contacts::default();
        contacts.press(1, true, 0.0, 0.0);
        contacts.lost(1);
        assert!(!contacts.release(1, 0.0, 0.0));
        contacts.press(2, true, 0.0, 0.0);
        contacts.hold(2);
        assert!(!contacts.release(2, 0.0, 0.0));
        contacts.press(3, true, 0.0, 0.0);
        contacts.clear();
        assert!(!contacts.release(3, 0.0, 0.0));
    }
    #[test]
    fn nonfinite_pointer_positions_are_rejected() {
        let mut contacts = Contacts::default();
        assert!(!contacts.press(1, true, f64::NAN, 0.0));
        assert!(!contacts.release(1, 0.0, 0.0));
        contacts.press(2, true, 0.0, 0.0);
        contacts.moved(2, f64::INFINITY, 0.0);
        assert!(!contacts.release(2, 0.0, 0.0));
    }
}
