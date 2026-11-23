use crate::{
    menus,
    session::{Panel, Session},
};
use ferroui_view::prelude::*;
use reader_controls::Icon;
use reader_domain::{Density, SortOrder, View, ViewMode};
use reader_state::AppViewModel;
use std::rc::Rc;

pub(super) fn filter_menu(vm: Rc<AppViewModel>) -> Element {
    let unread = vm.state().settings.unread_only;
    let all = vm.clone();
    let fresh = vm.clone();
    let touch = crate::responsive::touch(&vm);
    let label = if unread { "New items" } else { "All items" };
    let filter = if touch {
        reader_controls::dropdown(label)
    } else {
        reader_controls::compact_dropdown(label)
    };
    menus::attach(
        &vm,
        filter
            .name("StreamFilter")
            .width(if touch { 96.0 } else { 72.0 })
            .padding((if touch { 6.0 } else { 2.0 }, 5.0)),
        vec![
            menus::checked("All items", !unread, move || all.set_unread_only(false)),
            menus::checked("New items", unread, move || fresh.set_unread_only(true)),
        ],
    )
}
pub(super) fn mark_menu(vm: Rc<AppViewModel>, ui: Rc<Session>) -> Element {
    let mark = vm.clone();
    let confirm = ui.clone();
    let touch = crate::responsive::touch(&vm);
    rsx! {Grid{column_definitions:if touch{"99,44"}else{"99,20"},height:if touch{44.0}else{29.0},
        Button{class:"reader-button",padding:(6.0,5.0),content:"Mark all as read",automation_properties_name:"Mark all as read",on_click:move |_|{
            request_mark_all(&mark,&confirm,None);
        }}
        Border{Grid.column:1,{menus::attach(&vm,reader_controls::toolbar_icon(Icon::ChevronDown,"Mark all read options").width(if touch{44.0}else{20.0}).padding(5.0),mark_items(vm.clone(),ui,false))}}
    }}
}
fn mark_items(vm: Rc<AppViewModel>, ui: Rc<Session>, standalone: bool) -> Vec<Element> {
    let all = vm.clone();
    let confirm_all = ui.clone();
    let day = vm.clone();
    let confirm_day = ui.clone();
    let week = vm.clone();
    let confirm_week = ui.clone();
    let fortnight = vm.clone();
    vec![
        menus::item(
            if standalone {
                "Mark all as read"
            } else {
                "All items"
            },
            move || {
                request_mark_all(&all, &confirm_all, None);
            },
        ),
        menus::item(
            if standalone {
                "Mark older than a day as read"
            } else {
                "Items older than a day"
            },
            move || request_mark_all(&day, &confirm_day, Some(day.now() - 86400)),
        ),
        menus::item(
            if standalone {
                "Mark older than a week as read"
            } else {
                "Items older than a week"
            },
            move || request_mark_all(&week, &confirm_week, Some(week.now() - 7 * 86400)),
        ),
        menus::item(
            if standalone {
                "Mark older than two weeks as read"
            } else {
                "Items older than two weeks"
            },
            move || request_mark_all(&fortnight, &ui, Some(fortnight.now() - 14 * 86400)),
        ),
    ]
}
fn request_mark_all(vm: &AppViewModel, ui: &Session, cutoff: Option<i64>) {
    if vm.should_confirm_mark_all() {
        ui.show(Panel::MarkAll(cutoff));
    } else {
        vm.mark_all_read(cutoff);
    }
}
pub(super) fn sort_menu(vm: Rc<AppViewModel>, ui: Rc<Session>) -> Element {
    let label = if matches!(vm.view(), View::Feed(_)) {
        "Feed settings"
    } else {
        "View settings"
    };
    menus::attach(
        &vm,
        reader_controls::dropdown(label).width(115.0),
        sort_items(vm.clone(), ui),
    )
}
fn sort_items(vm: Rc<AppViewModel>, ui: Rc<Session>) -> Vec<Element> {
    let order = vm.state().settings.sort;
    let newest = vm.clone();
    let oldest = vm.clone();
    let larger = ui.clone();
    let smaller = ui.clone();
    let scale = ui.article_scale.get();
    let current = vm.cursor().or(vm.selected()).is_some();
    let mut items = vec![
        menus::checked("Sort by newest", order == SortOrder::Newest, move || {
            newest.set_sort(SortOrder::Newest)
        }),
        menus::checked("Sort by oldest", order == SortOrder::Oldest, move || {
            oldest.set_sort(SortOrder::Oldest)
        }),
        menus::separator(),
        menus::item_enabled(
            "Larger article text (+)",
            current && scale.can_change(true),
            move || {
                crate::reading::scale::change(&larger, true);
            },
        ),
        menus::item_enabled(
            "Smaller article text (-)",
            current && scale.can_change(false),
            move || {
                crate::reading::scale::change(&smaller, false);
            },
        ),
    ];
    if let View::Feed(id) = vm.view() {
        let rename = ui.clone();
        let folders = ui.clone();
        let unsubscribe = ui.clone();
        items.push(menus::separator());
        items.push(menus::item("Rename subscription...", move || {
            rename.show(Panel::Rename(id))
        }));
        items.push(menus::item("Change folders...", move || {
            folders.show(Panel::Move(id))
        }));
        items.push(menus::item("Unsubscribe...", move || {
            unsubscribe.show(Panel::Unsubscribe(id))
        }));
    }
    items
}
pub(super) fn more_items(vm: Rc<AppViewModel>, ui: Rc<Session>) -> Vec<Element> {
    let list = vm.clone();
    let expanded = vm.clone();
    let previous = vm.clone();
    let next = vm.clone();
    let mode = vm.state().settings.view_mode;
    let has_items = vm.visible_len() > 0;
    let mut items = vec![
        menus::choice(
            "List view",
            mode == ViewMode::List,
            Some("view-mode"),
            move || list.set_view_mode(ViewMode::List),
        ),
        menus::choice(
            "Expanded view",
            mode == ViewMode::Expanded,
            Some("view-mode"),
            move || expanded.set_view_mode(ViewMode::Expanded),
        ),
        menus::item_enabled("Previous item", has_items, move || {
            previous.next_article(-1)
        }),
        menus::item_enabled("Next item", has_items, move || next.next_article(1)),
        menus::separator(),
    ];
    items.extend(mark_items(vm.clone(), ui.clone(), true));
    items.push(menus::separator());
    items.extend(sort_items(vm, ui));
    items
}

pub(super) fn settings_items(vm: Rc<AppViewModel>, ui: Rc<Session>) -> Vec<Element> {
    let pref = ui.clone();
    let sub = ui.clone();
    let imports = ui.clone();
    let keys = ui.clone();
    let about = ui.clone();
    let comfortable = vm.clone();
    let cozy = vm.clone();
    let compact = vm.clone();
    let density = vm.state().settings.density;
    vec![
        menus::item("Reader settings", move || pref.show(Panel::Preferences)),
        menus::item("Subscriptions", move || sub.show(Panel::Subscriptions)),
        menus::item("Import/Export", move || imports.show(Panel::ImportExport)),
        menus::separator(),
        menus::checked("Comfortable", density == Density::Comfortable, move || {
            comfortable.set_density(Density::Comfortable)
        }),
        menus::checked("Cozy", density == Density::Cozy, move || {
            cozy.set_density(Density::Cozy)
        }),
        menus::checked("Compact", density == Density::Compact, move || {
            compact.set_density(Density::Compact)
        }),
        menus::separator(),
        menus::item("Keyboard shortcuts", move || keys.show(Panel::Shortcuts)),
        menus::item("About Reader", move || about.show(Panel::About)),
    ]
}
