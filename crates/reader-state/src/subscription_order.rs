use crate::AppViewModel;
use reader_domain::{Action, Id, SubscriptionNode, SubscriptionOrder, SubscriptionSort};

impl AppViewModel {
    pub fn subscription_sort(&self) -> SubscriptionSort {
        self.state().subscription_order.sort
    }

    pub fn subscription_tree(&self) -> SubscriptionOrder {
        self.state().subscription_tree()
    }

    pub fn set_subscription_sort(&self, sort: SubscriptionSort) -> bool {
        self.order_command(Action::SetSubscriptionSort(sort), None)
    }

    pub fn sort_subscriptions(&self, folder: Option<String>) -> bool {
        self.order_command(Action::SortSubscriptions(folder), None)
    }

    pub fn position_root(
        &self,
        item: SubscriptionNode,
        target: SubscriptionNode,
        before: bool,
    ) -> bool {
        self.order_command(
            Action::PositionRoot {
                item,
                target,
                before,
            },
            None,
        )
    }

    pub fn position_feed_occurrence(
        &self,
        id: Id,
        from_folder: String,
        to_folder: String,
        target: Option<SubscriptionNode>,
        before: bool,
    ) -> bool {
        self.order_command(
            Action::PositionFeedOccurrence {
                id,
                from_folder,
                to_folder,
                target,
                before,
            },
            Some(id),
        )
    }

    fn order_command(&self, action: Action, source: Option<Id>) -> bool {
        // Compare only the small tree metadata and affected labels. Cloning an
        // article document just to validate a pointer/menu action is unnecessary.
        let snapshot = || {
            let state = self.state();
            (
                state.subscription_order.clone(),
                source.and_then(|id| state.feed(id).map(|feed| feed.folder_names())),
            )
        };
        let before = snapshot();
        self.dispatch(action);
        before != snapshot()
    }
}
