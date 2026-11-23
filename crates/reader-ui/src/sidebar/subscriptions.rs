use crate::{
    menus,
    session::{Panel, Session},
};
use ferroui_base::layout::{HorizontalAlignment as H, VerticalAlignment as V};
use ferroui_base::media::FontWeight;
use ferroui_view::prelude::*;
use reader_controls::{icon, Icon};
use reader_domain::{SubscriptionSort, View};
use reader_state::AppViewModel;
use std::rc::Rc;

pub(super) fn header(vm: Rc<AppViewModel>, ui: Rc<Session>, inset: f64) -> Element {
    let drop_vm = vm.clone();
    let touch = crate::responsive::touch(&vm);
    menus::attach(
        &vm,
        button()
            .key("subscriptions-heading")
            .name("SubscriptionSettings")
            .class("reader-flat")
            .automation_properties_name("Subscriptions")
            .tool_tip_tip("Subscription settings")
            .height(if touch { 44.0 } else { 32.0 })
            .padding((inset, 0.0, 12.0, 0.0))
            .horizontal_content_alignment(H::Stretch)
            .with(menus::attach_context_keys)
            .with(move |heading| crate::drag::attach_target(
                &heading.clone().upcast(), drop_vm.clone(), String::new(),
            ))
            .child(rsx! {
                Grid {column_definitions:"*,12",column_spacing:8.0,
                    TextBlock {text:"Subscriptions",font_size:13.0,font_weight:FontWeight::Bold,vertical_alignment:V::Center}
                    Border {Grid.column:1,vertical_alignment:V::Center,{icon(Icon::ChevronDown,8.0,"#777777")}}
                }
            }),
        items(vm.clone(), ui),
    )
}

fn items(vm: Rc<AppViewModel>, ui: Rc<Session>) -> Vec<Element> {
    let generation = vm.library_generation();
    let refresh = vm.clone();
    let alphabetical = vm.clone();
    let manual = vm.clone();
    let all = vm.clone();
    let updated = vm.clone();
    let add = ui.clone();
    let show_all = vm.state().settings.show_all_subscriptions;
    let sort = vm.subscription_sort();
    vec![
        menus::item_enabled(
            "Refresh",
            !vm.is_refreshing() && !vm.feeds().is_empty(),
            move || {
                if refresh.library_generation() == generation {
                    refresh.refresh_view(View::All);
                }
            },
        ),
        menus::separator(),
        menus::choice(
            "Sort alphabetically",
            sort == SubscriptionSort::Alphabetical,
            Some("subscription-sort"),
            move || {
                if alphabetical.library_generation() == generation {
                    alphabetical.set_subscription_sort(SubscriptionSort::Alphabetical);
                }
            },
        ),
        menus::choice(
            "Sort by drag and drop",
            sort == SubscriptionSort::Manual,
            Some("subscription-sort"),
            move || {
                if manual.library_generation() == generation {
                    manual.set_subscription_sort(SubscriptionSort::Manual);
                }
            },
        ),
        menus::separator(),
        menus::choice(
            "Show all",
            show_all,
            Some("subscription-visibility"),
            move || visibility(&all, generation, true),
        ),
        menus::choice(
            "Show updated",
            !show_all,
            Some("subscription-visibility"),
            move || visibility(&updated, generation, false),
        ),
        menus::separator(),
        menus::item("Add subscription...", move || add.show(Panel::Subscribe)),
        menus::item("Manage subscriptions...", move || {
            ui.show(Panel::Subscriptions)
        }),
    ]
}

fn visibility(vm: &AppViewModel, generation: u64, show_all: bool) {
    if vm.library_generation() != generation {
        return;
    }
    let mut settings = vm.state().settings.clone();
    if settings.show_all_subscriptions != show_all {
        settings.show_all_subscriptions = show_all;
        vm.set_settings(settings);
    }
}
