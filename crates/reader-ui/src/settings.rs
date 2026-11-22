use crate::session::{Panel, Session};
use ferroui_base::layout::{Orientation, VerticalAlignment as V};
use ferroui_base::media::{FontWeight, TextWrapping};
use ferroui_view::prelude::*;
use reader_controls::{action, link, paragraph, separator};
use reader_domain::Density;
use reader_state::AppViewModel;
use std::rc::Rc;

pub fn preferences(vm: Rc<AppViewModel>, ui: Rc<Session>) -> Element {
    let settings = vm.state().settings.clone();
    let mut form = stack_panel().spacing(16.0).child(paragraph(
        "Choose how you read and organize your subscriptions.",
    ));
    for (label, current, field) in [
        (
            "Mark items as read when scrolling in expanded view",
            settings.mark_read_on_scroll,
            0,
        ),
        (
            "Confirm when marking all items as read",
            settings.confirm_mark_all,
            1,
        ),
        (
            "Show all subscriptions, including those with no unread items",
            settings.show_all_subscriptions,
            2,
        ),
    ] {
        let model = vm.clone();
        form = form.child(
            check_box()
                .content(label)
                .font_size(13.0)
                .min_height(28.0)
                .padding(0.0)
                .is_checked(Some(current))
                .on_click(move |_| {
                    let mut settings = model.state().settings.clone();
                    match field {
                        0 => settings.mark_read_on_scroll = !current,
                        1 => settings.confirm_mark_all = !current,
                        _ => settings.show_all_subscriptions = !current,
                    };
                    model.set_settings(settings);
                }),
        );
    }
    form = form.child(separator()).child(
        text_block()
            .text("Display density")
            .font_weight(FontWeight::Bold),
    );
    let mut density_row = stack_panel()
        .orientation(Orientation::Horizontal)
        .spacing(8.0);
    for (label, density) in [
        ("Comfortable", Density::Comfortable),
        ("Cozy", Density::Cozy),
        ("Compact", Density::Compact),
    ] {
        let model = vm.clone();
        density_row = density_row.child(action(label, move || model.set_density(density)));
    }
    form = form.child(density_row).child(
        text_block()
            .text("Article text size")
            .font_weight(FontWeight::Bold),
    );
    let mut fonts = stack_panel()
        .orientation(Orientation::Horizontal)
        .spacing(8.0);
    for (label, size) in [
        ("Small", 13.0),
        ("Medium", 14.0),
        ("Large", 16.0),
        ("Extra large", 18.0),
    ] {
        let model = vm.clone();
        fonts = fonts.child(action(label, move || {
            let mut settings = model.state().settings.clone();
            settings.font_size = size;
            model.set_settings(settings);
        }));
    }
    form = form.child(fonts).child(separator()).child(
        text_block()
            .text("Feed connection")
            .font_weight(FontWeight::Bold),
    );
    if ui.form_for.borrow().as_ref() != Some(&Panel::Preferences) {
        *ui.form_for.borrow_mut() = Some(Panel::Preferences);
        ui.entry.set(settings.feed_proxy);
    }
    let save = vm.clone();
    let input = ui.entry;
    form=form.child(paragraph("Feeds that allow browser requests work directly. For other feeds, you can use a trusted proxy. Enter its URL with {url} where the encoded feed address should go."))
        .child(reader_controls::input("Feed proxy URL").name("FeedProxy").text(ui.entry.two_way()).placeholder_text("https://your-service.example/feed?url={url}"))
        .child(action("Save feed connection",move||{let mut settings=save.state().settings.clone();settings.feed_proxy=input.get_untracked().trim().to_owned();save.set_settings(settings);save.set_status("Feed connection saved.".into());}));
    form.into()
}
pub fn import_export(vm: Rc<AppViewModel>) -> Element {
    let import = vm.clone();
    let export = vm.clone();
    let backup = vm.clone();
    let restore = vm.clone();
    rsx! {StackPanel {spacing:18.0,
        TextBlock {text:"Your subscriptions",font_weight:FontWeight::Bold,font_size:14.0}
        {paragraph("Import subscriptions from an OPML file, including an export from Google Reader or another feed reader. Folder names are preserved.")}
        StackPanel {orientation:Orientation::Horizontal,spacing:10.0,
            {action("Import subscriptions...",move||import.request_import_opml())}
            {action("Export subscriptions",move||export.request_export_opml())}
        }
        {separator()}
        TextBlock {text:"Your complete library",font_weight:FontWeight::Bold,font_size:14.0}
        {paragraph("A complete backup includes cached articles, read state, stars, tags, notes, subscriptions and preferences.")}
        StackPanel {orientation:Orientation::Horizontal,spacing:10.0,
            {action("Download backup",move||backup.request_export_json())}
            {action("Restore backup...",move||restore.request_import_json())}
        }
    }}
}
pub fn subscriptions(vm: Rc<AppViewModel>, ui: Rc<Session>) -> Element {
    let import = ui.clone();
    let add = ui.clone();
    let folder = ui.clone();
    let mut page = stack_panel()
        .spacing(14.0)
        .child(rsx! {
            StackPanel {orientation:Orientation::Horizontal,spacing:10.0,
                {action("Add a subscription",move||add.show(Panel::Subscribe))}
                {action("New folder",move||folder.show(Panel::NewFolder))}
                {action("Import/Export",move||import.show(Panel::ImportExport))}
            }
        })
        .child(
            reader_controls::input("Filter subscriptions")
                .placeholder_text("Filter subscriptions")
                .text(ui.subscription_filter.two_way()),
        );
    let filter = ui.subscription_filter.get().to_lowercase();
    let feeds = vm.feeds();
    for feed in feeds.iter().filter(|feed| {
        format!("{} {} {}", feed.title, feed.url, feed.folder)
            .to_lowercase()
            .contains(&filter)
    }) {
        let id = feed.id;
        let rename = ui.clone();
        let folders = ui.clone();
        let remove = ui.clone();
        page=page.child(border().border_brush("#DDDDDD").border_thickness((0.0,0.0,0.0,1.0)).padding((0.0,8.0)).child(rsx!{
            StackPanel {spacing:5.0,
                TextBlock {text:feed.title.clone(),font_weight:FontWeight::Bold,font_size:13.0}
                TextBlock {text:feed.url.clone(),font_size:11.0,foreground:"#777777",text_wrapping:TextWrapping::Wrap}
                StackPanel {orientation:Orientation::Horizontal,spacing:16.0,
                    {link("Rename",move||rename.show(Panel::Rename(id)))}
                    {link(&if feed.folder.is_empty(){"Change folders".to_string()}else{format!("Folder: {}",feed.folder)},move||folders.show(Panel::Move(id)))}
                    {link("Unsubscribe",move||remove.show(Panel::Unsubscribe(id)))}
                }
                if let Some(error)=feed.error.clone(){TextBlock{text:error,foreground:"#B3261E",text_wrapping:TextWrapping::Wrap,font_size:12.0}}
            }
        }));
    }
    if feeds.is_empty() {
        page = page.child(paragraph("You haven't subscribed to any feeds yet."));
    }
    let folders = vm.folders();
    if !folders.is_empty() {
        page = page
            .child(separator())
            .child(text_block().text("Folders").font_weight(FontWeight::Bold));
        for folder in folders {
            let model = vm.clone();
            let name = folder.clone();
            page=page.child(rsx!{Grid{column_definitions:"*,Auto",
            TextBlock{text:folder,vertical_alignment:V::Center}
            Border{Grid.column:1,{link("Remove folder",move||model.delete_folder(name.clone()))}}
        }});
        }
    }
    page.into()
}
