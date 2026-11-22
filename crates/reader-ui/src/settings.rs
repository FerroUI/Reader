use crate::session::{Panel, Session};
use ferroui_base::layout::{HorizontalAlignment as H, VerticalAlignment as V};
use ferroui_base::media::{FontWeight, TextWrapping};
use ferroui_view::prelude::*;
use reader_controls::{action, action_row, link, paragraph, separator};
use reader_domain::{Density, View};
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
            "Confirm before marking a stream with more than 50 unread items as read",
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
                .class("reader-check")
                .content(label)
                .horizontal_alignment(H::Stretch)
                .horizontal_content_alignment(H::Stretch)
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
    let mut starts = vec![
        ("Home".to_owned(), View::Home),
        ("All items".to_owned(), View::All),
        ("Starred items".to_owned(), View::Starred),
        ("Trends".to_owned(), View::Trends),
        ("Explore".to_owned(), View::Explore),
    ];
    starts.extend(
        vm.folders()
            .into_iter()
            .map(|name| (name.clone(), View::Folder(name))),
    );
    starts.extend(
        vm.feeds()
            .into_iter()
            .map(|feed| (feed.title, View::Feed(feed.id))),
    );
    let start_label = starts
        .iter()
        .find(|(_, view)| view == &settings.start_page)
        .map(|(label, _)| label.clone())
        .unwrap_or_else(|| "Home".into());
    let mut start_items = Vec::new();
    for (label, view) in starts {
        let model = vm.clone();
        let selected = settings.start_page == view;
        start_items.push(crate::menus::checked(&label, selected, move || {
            let mut settings = model.state().settings.clone();
            settings.start_page = view.clone();
            model.set_settings(settings);
        }));
    }
    form = form
        .child(
            text_block()
                .text("Start page")
                .font_weight(FontWeight::Bold),
        )
        .child(crate::menus::button(&vm, &start_label, start_items));
    form = form.child(separator()).child(
        text_block()
            .text("Display density")
            .font_weight(FontWeight::Bold),
    );
    let mut density_row = Vec::new();
    for (label, density) in [
        ("Comfortable", Density::Comfortable),
        ("Cozy", Density::Cozy),
        ("Compact", Density::Compact),
    ] {
        let model = vm.clone();
        density_row.push(choice(label, settings.density == density, move || {
            model.set_density(density)
        }));
    }
    form = form.child(action_row(density_row, 8.0)).child(
        text_block()
            .text("Article text size")
            .font_weight(FontWeight::Bold),
    );
    let mut fonts = Vec::new();
    for (label, size) in [
        ("Small", 13.0),
        ("Medium", 14.0),
        ("Large", 16.0),
        ("Extra large", 18.0),
    ] {
        let model = vm.clone();
        fonts.push(choice(
            label,
            (settings.font_size - size).abs() < 0.1,
            move || {
                let mut settings = model.state().settings.clone();
                settings.font_size = size;
                model.set_settings(settings);
            },
        ));
    }
    form = form.child(action_row(fonts, 8.0)).child(separator()).child(
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
        {action_row([action("Import subscriptions...",move||import.request_import_opml()),action("Export subscriptions",move||export.request_export_opml())],10.0)}
        {separator()}
        TextBlock {text:"Your complete library",font_weight:FontWeight::Bold,font_size:14.0}
        {paragraph("A complete backup includes cached articles, read state, stars, tags, notes, subscriptions and preferences.")}
        {action_row([action("Download backup",move||backup.request_export_json()),action("Restore backup...",move||restore.request_import_json())],10.0)}
    }}
}
pub fn subscriptions(vm: Rc<AppViewModel>, ui: Rc<Session>) -> Element {
    let import = ui.clone();
    let add = ui.clone();
    let folder = ui.clone();
    let mut page = stack_panel()
        .spacing(14.0)
        .child(action_row(
            [
                action("Add a subscription", move || add.show(Panel::Subscribe)),
                action("New folder", move || folder.show(Panel::NewFolder)),
                action("Import/Export", move || import.show(Panel::ImportExport)),
            ],
            10.0,
        ))
        .child(
            reader_controls::input("Filter subscriptions")
                .placeholder_text("Filter subscriptions")
                .text(ui.subscription_filter.two_way()),
        );
    let filter = ui.subscription_filter.get().to_lowercase();
    let feeds = vm.feeds();
    let mut matches = 0;
    for feed in feeds.iter().filter(|feed| {
        format!(
            "{} {} {}",
            feed.title,
            feed.url,
            feed.folder_names().join(" ")
        )
        .to_lowercase()
        .contains(&filter)
    }) {
        matches += 1;
        let id = feed.id;
        let rename = ui.clone();
        let folders = ui.clone();
        let remove = ui.clone();
        page=page.child(border().border_brush("#DDDDDD").border_thickness((0.0,0.0,0.0,1.0)).padding((0.0,8.0)).child(rsx!{
            StackPanel {spacing:5.0,
                TextBlock {text:feed.title.clone(),font_weight:FontWeight::Bold,font_size:13.0,text_wrapping:TextWrapping::Wrap}
                TextBlock {text:feed.url.clone(),font_size:11.0,foreground:"#777777",text_wrapping:TextWrapping::Wrap}
                if !feed.folder_names().is_empty(){TextBlock{text:format!("Folders: {}",feed.folder_names().join(", ")),font_size:12.0,foreground:"#777777",text_wrapping:TextWrapping::Wrap}}
                {action_row([
                    link("Rename",move||rename.show(Panel::Rename(id))),
                    link("Change folders",move||folders.show(Panel::Move(id))),
                    link("Unsubscribe",move||remove.show(Panel::Unsubscribe(id)))
                ],16.0)}
                if let Some(error)=feed.error.clone(){TextBlock{text:error,foreground:"#B3261E",text_wrapping:TextWrapping::Wrap,font_size:12.0}}
            }
        }));
    }
    if feeds.is_empty() {
        page = page.child(paragraph("You haven't subscribed to any feeds yet."));
    } else if matches == 0 {
        page = page.child(paragraph("No subscriptions match your filter."));
    }
    let folders = vm.folders();
    if !folders.is_empty() {
        page = page
            .child(separator())
            .child(text_block().text("Folders").font_weight(FontWeight::Bold));
        for folder in folders {
            let model = vm.clone();
            let name = folder.clone();
            let rename_name = folder.clone();
            let rename_ui = ui.clone();
            let narrow = crate::responsive::narrow(&vm);
            page=page.child(rsx!{Grid{column_definitions:if narrow{"*"}else{"*,Auto"},row_definitions:if narrow{"Auto,Auto"}else{"Auto"},column_spacing:16.0,
            TextBlock{text:folder,vertical_alignment:V::Center,text_wrapping:TextWrapping::Wrap}
            Border{Grid.column:if narrow{0}else{1},Grid.row:if narrow{1}else{0},{action_row([
                link("Rename",move||rename_ui.show(Panel::RenameFolder(rename_name.clone()))),
                link("Remove folder",move||model.delete_folder(name.clone()))
            ],16.0)}}
        }});
        }
    }
    page.into()
}

fn choice(label: &str, selected: bool, activate: impl Fn() + 'static) -> Element {
    button()
        .class(if selected {
            "reader-button selected"
        } else {
            "reader-button"
        })
        .content(label)
        .automation_properties_name(label)
        .on_click(move |event| {
            event.set_handled(true);
            activate();
        })
        .into()
}
