//! Home distinguishes a first download from an actually caught-up library.
use ferroui_base::media::TextWrapping;
use ferroui_view::prelude::*;
use reader_state::AppViewModel;
use std::rc::Rc;

pub(super) fn view(vm: Rc<AppViewModel>) -> Element {
    let loading = vm.is_refreshing();
    let state = vm.state();
    let pending = state.feeds.iter().any(|feed| feed.fetched_at == 0);
    let failed = state.feeds.iter().any(|feed| feed.error.is_some());
    let (title, message) = if loading {
        (
            "Loading your feeds…",
            "Latest articles will appear here as they arrive.",
        )
    } else if state.articles.is_empty() && failed {
        (
            "Your feeds are ready to try again",
            "Check your connection, then refresh to download the latest articles.",
        )
    } else if state.articles.is_empty() && pending {
        (
            "Your subscriptions are ready",
            "Refresh to load the latest articles from your feeds.",
        )
    } else {
        (
            "You're all caught up!",
            "There are no unread items in your subscriptions. Refresh to check for new stories.",
        )
    };
    drop(state);
    rsx! { StackPanel {name:"HomeFeedStatus",spacing:12.0,
        TextBlock {text:title,font_size:20.0,text_wrapping:TextWrapping::Wrap}
        {reader_controls::paragraph(message)}
        if !loading {
            {reader_controls::action("Refresh feeds",move ||vm.refresh())}
        }
    }}
}
