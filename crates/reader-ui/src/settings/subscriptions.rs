use super::controls::{heading, help};
use crate::session::{FolderTarget, Panel, Session};
use ferroui_base::layout::{HorizontalAlignment as H, VerticalAlignment as V};
use ferroui_base::media::{FontWeight, TextWrapping};
use ferroui_view::prelude::*;
use reader_controls::{action, action_row, paragraph, separator};
use reader_domain::{Feed, View};
use reader_state::AppViewModel;
use std::rc::Rc;

pub fn view(vm: Rc<AppViewModel>, ui: Rc<Session>) -> Element {
    let import = ui.clone();
    let add = ui.clone();
    let folder = ui.clone();
    let filter = ui.subscription_filter.get().trim().to_lowercase();
    let clear = ui.subscription_filter;
    let touch = crate::responsive::touch(&vm);
    let height = if touch { 44.0 } else { 29.0 };
    let feeds = vm.feeds();
    let matches = feeds
        .iter()
        .filter(|feed| matches_filter(feed, &filter))
        .collect::<Vec<_>>();
    let mut page=stack_panel().name("SubscriptionSettings").spacing(12.0)
        .child(action_row([
            action("Add a subscription",move||add.show(Panel::Subscribe)),
            action("New folder",move||folder.show(Panel::NewFolder)),
            action("Import/Export",move||import.show(Panel::ImportExport)),
        ],10.0))
        .child(rsx! {StackPanel {spacing:6.0,
            {heading("Filter subscriptions")}
            Grid {column_definitions:"*,Auto",column_spacing:8.0,
                {reader_controls::input("Filter subscriptions").name("SubscriptionFilter").height(height)
                    .placeholder_text("Title, feed URL or folder").text(ui.subscription_filter.two_way())}
                Button {Grid.column:1,name:"ClearSubscriptionFilter",class:"reader-button",content:"Clear",height:height,
                    automation_properties_name:"Clear subscription filter",is_enabled:!filter.is_empty(),on_click:move |_|clear.set_if_changed(String::new())}
            }
            TextBlock {name:"SubscriptionResultCount",text:if filter.is_empty(){format!("{} subscriptions",feeds.len())}else{format!("{} of {} subscriptions",matches.len(),feeds.len())},font_size:12.0,foreground:"#666666",text_wrapping:TextWrapping::Wrap}
        }});
    for feed in &matches {
        page = page.child(row(feed, &vm, &ui, height));
    }
    if feeds.is_empty() {
        page = page.child(paragraph("You haven't subscribed to any feeds yet."));
    } else if matches.is_empty() {
        page = page.child(paragraph(
            "No subscriptions match your filter. Try a title, feed URL or folder name.",
        ));
    }
    let folders = vm
        .folders()
        .into_iter()
        .filter(|folder| filter.is_empty() || folder.to_lowercase().contains(&filter))
        .collect::<Vec<_>>();
    if !folders.is_empty() {
        page = page.child(separator()).child(heading("Folders"));
        for folder in folders {
            let delete_ui = ui.clone();
            let delete_target = FolderTarget::capture(&vm, &folder);
            let rename_name = folder.clone();
            let rename_ui = ui.clone();
            page=page.child(rsx! {StackPanel {key:format!("settings-folder:{folder}"),spacing:4.0,
                TextBlock {text:folder.clone(),font_weight:FontWeight::Bold,text_wrapping:TextWrapping::Wrap}
                {action_row([
                    row_action("Rename",&format!("Rename folder {folder}"),height,move||rename_ui.show(Panel::RenameFolder(rename_name.clone()))),
                    row_action("Delete folder",&format!("Delete folder {folder}"),height,move||delete_ui.show(Panel::DeleteFolder(delete_target.clone())))
                ],16.0)}
            }});
        }
    }
    page.into()
}

fn matches_filter(feed: &Feed, filter: &str) -> bool {
    filter.is_empty()
        || feed.title.to_lowercase().contains(filter)
        || feed.url.to_lowercase().contains(filter)
        || feed
            .folder_names()
            .iter()
            .any(|folder| folder.to_lowercase().contains(filter))
}

fn row(feed: &Feed, vm: &AppViewModel, ui: &Rc<Session>, height: f64) -> Element {
    let id = feed.id;
    let rename = ui.clone();
    let folders = ui.clone();
    let remove = ui.clone();
    let unread = vm.count(&View::Feed(id));
    rsx! {Border {key:format!("settings-feed:{id}"),name:format!("Subscription-{id}"),border_brush:"#DDDDDD",border_thickness:(0.0,0.0,0.0,1.0),padding:(0.0,8.0),
        StackPanel {spacing:5.0,
            Grid {column_definitions:"*,Auto",column_spacing:12.0,
                TextBlock {text:feed.title.clone(),font_weight:FontWeight::Bold,font_size:13.0,text_wrapping:TextWrapping::Wrap,vertical_alignment:V::Center}
                if unread>0 {TextBlock {Grid.column:1,text:format!("{unread} unread"),font_size:12.0,foreground:"#666666",vertical_alignment:V::Center}}
            }
            {help(feed.url.clone())}
            if !feed.folder_names().is_empty() {{help(format!("Folders: {}",feed.folder_names().join(", ")))}}
            if let Some(error)=feed.error.clone(){TextBlock{text:error,foreground:"#B3261E",text_wrapping:TextWrapping::Wrap,font_size:12.0}}
            {action_row([
                row_action("Rename",&format!("Rename subscription {}",feed.title),height,move||rename.show(Panel::Rename(id))),
                row_action("Change folders",&format!("Change folders for {}",feed.title),height,move||folders.show(Panel::Move(id))),
                row_action("Unsubscribe",&format!("Unsubscribe from {}",feed.title),height,move||remove.show(Panel::Unsubscribe(id)))
            ],16.0)}
        }
    }}
}

fn row_action(label: &str, name: &str, height: f64, activate: impl Fn() + 'static) -> Element {
    button()
        .class("reader-link")
        .content(label)
        .automation_properties_name(name)
        .min_height(height)
        .horizontal_content_alignment(H::Left)
        .on_click(move |event| {
            event.set_handled(true);
            activate();
        })
        .into()
}
