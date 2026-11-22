use std::cell::Cell;

#[derive(Clone, Copy)]
pub struct Ticket {
    pub generation: u64,
    subscription: u64,
}

#[derive(Default)]
pub struct FeedOperations {
    generation: Cell<u64>,
    pending: Cell<usize>,
    latest_subscription: Cell<u64>,
}
impl FeedOperations {
    pub fn begin(&self, generation: u64, subscribe: bool) -> Ticket {
        if self.generation.replace(generation) != generation {
            self.pending.set(0);
        }
        self.pending.set(self.pending.get() + 1);
        let subscription = if subscribe {
            let next = self.latest_subscription.get().wrapping_add(1).max(1);
            self.latest_subscription.set(next);
            next
        } else {
            0
        };
        Ticket {
            generation,
            subscription,
        }
    }
    pub fn is_latest_subscription(&self, ticket: Ticket) -> bool {
        ticket.subscription != 0 && ticket.subscription == self.latest_subscription.get()
    }
    /// Returns true only when the final operation in this library has finished.
    pub fn complete(&self, ticket: Ticket) -> bool {
        if ticket.generation != self.generation.get() || self.pending.get() == 0 {
            return false;
        }
        self.pending.set(self.pending.get() - 1);
        self.pending.get() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn concurrent_subscriptions_complete_independently_and_only_latest_can_navigate() {
        let operations = FeedOperations::default();
        let first = operations.begin(0, true);
        let second = operations.begin(0, true);
        assert!(!operations.is_latest_subscription(first));
        assert!(operations.is_latest_subscription(second));
        assert!(!operations.complete(second));
        assert!(operations.complete(first));
    }
    #[test]
    fn a_restored_library_does_not_wait_for_or_finish_an_old_request() {
        let operations = FeedOperations::default();
        let old = operations.begin(0, true);
        let refresh = operations.begin(1, false);
        assert!(!operations.complete(old));
        assert!(operations.complete(refresh));
    }
}
